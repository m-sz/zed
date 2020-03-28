use std::time::{Instant, Duration};
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use laminar::{ErrorKind, SocketEvent, Socket, Packet};
use simple_signal::{self, Signal};
use crossbeam_channel::{Sender};

use zed_shared::protocol::{Protocol, SimpleProtocol, register_messages};
use zed_shared::message::{both, from_client, from_server};
use std::collections::HashMap;

const ADDR: &str = "127.0.0.1:10995";

fn main() -> Result<(), ErrorKind> {
    let addr: String;

    match std::env::args().skip(1).nth(0) {
        Some(a) => addr = a.clone(),
        None => addr = ADDR.to_string()
    };

    let mut clients = Vec::new();

    let mut socket = Socket::bind(addr)?;
    println!("Server on {}", socket.local_addr().unwrap());
    let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());

    let mut protocol = SimpleProtocol::new();
    register_messages(&mut protocol);

    let should_run = Arc::new(AtomicBool::new(true));
    let thread;
    {
        let should_run = should_run.clone();
        simple_signal::set_handler(&[Signal::Int], move |_signals| {
            should_run.store(false, Ordering::Relaxed);
        });
    }
    {
        let should_run = should_run.clone();
        thread = std::thread::spawn(move || {
            while should_run.load(Ordering::Relaxed) {
                socket.manual_poll(Instant::now());
                std::thread::sleep(Duration::from_micros(100));
            }
        });
    }

    while should_run.load(Ordering::Relaxed) {
        if let Ok(event) = receiver.recv() {
            match event {
                SocketEvent::Connect(addr) => {
                    if clients.iter().find(|&&a| a == addr).is_some() {
                        break;
                    }

                    println!("Client connected from {}", addr);
                    let client_id = clients.len();
                    clients.push(addr);

                    protocol.send_reliable_unordered(&sender, addr, from_server::GreetingResponse {
                        player_id: client_id
                    });
                },
                SocketEvent::Timeout(addr) => {
                    println!("Client timeout {}", addr);
                }
                SocketEvent::Packet(packet) => {
                    println!("Received a packet from {}", packet.addr());
                    for c in &clients {
                        sender.try_send(Packet::reliable_unordered(
                            *c,
                            packet.payload().to_vec()
                        )).unwrap();
                    }
                },
                _ => ()
            }
        }
    }

    thread.join();
    Ok(())
}