use serde::{
    de::DeserializeOwned,
    ser::Serialize
};

use std::any::TypeId;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::net::SocketAddr;
use std::rc::Rc;

use laminar::{Packet, Socket, SocketEvent};
use bottles::{Dispatcher, Queue};

#[cfg(test)]
use mockall::automock;

pub trait Protocol {
    fn register<T: 'static + DeserializeOwned>(&mut self);
    fn send_reliable_unordered<S: Sender<Packet>, T: 'static + Serialize>(&mut self, sender: &S, addr: SocketAddr, value: T);
    fn receive(&mut self, raw: &[u8]);
}

#[cfg_attr(test, automock)]
pub trait Sender<T: Send + 'static> {
    fn send(&self, msg: T) -> Result<(), Box<dyn std::error::Error>>;
}

type Decoder = Box<dyn Fn(&mut Dispatcher, &mut dyn Read)>;

pub struct SimpleProtocol {
    dispatcher: Dispatcher,
    decoders: Vec<Decoder>,
    message_ids: HashMap<TypeId, usize>,
}

impl SimpleProtocol {
    pub fn new() -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            decoders: Vec::new(),
            message_ids: HashMap::new(),
        }
    }

    fn prepare_send_buffer<T: 'static + Serialize>(&self, message: T) -> Vec<u8> {
        let mut buffer = Vec::new();
        bincode::serialize_into(&mut buffer, &self.message_ids.get(&TypeId::of::<T>()).unwrap()).unwrap();
        bincode::serialize_into(&mut buffer, &message).unwrap();

        buffer
    }

    pub fn dispatcher_mut(&mut self) -> &mut Dispatcher {
        &mut self.dispatcher
    }
}

impl Protocol for SimpleProtocol {
    fn register<T: 'static + DeserializeOwned>(&mut self) {
        self.dispatcher.register::<T>();

        let decoder = |dispatcher: &mut Dispatcher, read: &mut dyn Read| {
            let message: Rc<T> = Rc::new(bincode::deserialize_from(read).unwrap());
            dispatcher.dispatch(message);
        };
        let id = self.decoders.len();

        self.message_ids.insert(TypeId::of::<T>(), id);
        self.decoders.push(Box::new(decoder));
    }

    fn receive(&mut self, bytes: &[u8]) {
        let mut bytes = Cursor::new(bytes);

        let discriminant: usize = bincode::deserialize_from(&mut bytes).unwrap();
        (self.decoders[discriminant])(&mut self.dispatcher, &mut bytes);
    }

    fn send_reliable_unordered<S: Sender<Packet>, T: 'static + Serialize>(&mut self, sender: &S, addr: SocketAddr, message: T)
    {
        sender.send(
            Packet::reliable_unordered(addr, self.prepare_send_buffer(message))
        ).unwrap();
    }
}

pub fn register_messages(protocol: &mut impl Protocol) {
    use crate::message::{
        from_client,
        from_server,
        both
    };

    protocol.register::<from_client::Greeting>();
    protocol.register::<from_server::GreetingResponse>();
    protocol.register::<both::PlayerStatus>();
}

impl<T: Send+'static> Sender<T> for crossbeam_channel::Sender<T> {
    fn send(&self, data: T) -> Result<(), Box<dyn std::error::Error>> {
        self.send(data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MockSender, SimpleProtocol, Protocol, Packet};
    use std::net::{SocketAddrV4, Ipv4Addr};
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    struct Msg {
        a: i32
    }

    #[test]
    fn test_basic() {
        let mut sender = MockSender::<Packet>::new();
        let mut protocol = SimpleProtocol::new();

        sender.expect_send()
            .times(1)
            .returning(|packet|
                Ok(())
            );
        
        protocol.register::<Msg>();

        protocol.send_reliable_unordered(&mut sender, SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1600).into(), Msg { a: 42 });

    }

    #[test]
    #[should_panic]
    fn test_send_unregistered_panics() {
        let mut sender = MockSender::<Packet>::new();
        sender.expect_send()
            .returning(|_| Ok(()));

        let mut protocol = SimpleProtocol::new();

        protocol.send_reliable_unordered(&mut sender, "127.0.0.1:8080".parse().unwrap(), Msg { a: 42 });
    }
}

