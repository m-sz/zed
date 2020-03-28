pub mod util;

mod zed;

use laminar::{Socket, SocketEvent, Packet};
use std::time::{Instant, Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::sleep;
use std::net::SocketAddr;

const ADDR: &str = "127.0.0.1:10995";

fn socket_poller(should_run: Arc<AtomicBool>, mut socket: Socket) -> Result<(), laminar::ErrorKind> {
    while should_run.load(Ordering::Relaxed) {
        socket.manual_poll(Instant::now());
        sleep(Duration::from_micros(100));
    }

    Ok(())
}

fn main() -> Result<(), laminar::ErrorKind> {
    use zed::app::{Main, Net};
    use zed_shared::protocol::SimpleProtocol;
    use zed_shared::protocol::register_messages;

    let addr: String;
    let local_addr: Option<String>;

    match std::env::args().skip(1).nth(0) {
        Some(a) => addr = a.clone(),
        None => addr = ADDR.to_string()
    };

    let mut socket = match std::env::args().skip(2).nth(0) {
        Some(a) => Socket::bind(a),
        None => Socket::bind_any()
    }?;

    println!("Client on on {}", socket.local_addr().unwrap());
    let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());

    let mut protocol = SimpleProtocol::new();
    register_messages(&mut protocol);

    println!("Server address: {}", addr);
    let net = Net::new(&mut socket, addr.parse().unwrap(), protocol);


    let should_run = Arc::new(AtomicBool::new(true));
    let net_thread = {
        let should_run = should_run.clone();
        std::thread::spawn(move || socket_poller(should_run, socket))
    };


    app::run(move |ctx| {
            zed::app::Main::new(ctx, net)
        },
        should_run.clone()
    );

    net_thread.join();
    Ok(())
}