//! A "tiny" example of HTTP request/response handling using just tokio-core
//!
//! This example is intended for *learning purposes* to see how various pieces
//! hook up together and how HTTP can get up and running. Note that this example
//! is written with the restriction that it *can't* use any "big" library other
//! than tokio-core, if you'd like a "real world" HTTP library you likely want a
//! crate like Hyper.
//!
//! Code here is based on the `echo-threads` example and implements two paths,
//! the `/plaintext` and `/json` routes to respond with some text and json,
//! respectively. By default this will run I/O on all the cores your system has
//! available, and it doesn't support HTTP request bodies.

#![deny(warnings)]

extern crate bytes;

extern crate time;
extern crate tokio;
extern crate tokio_codec;
extern crate bincode;

#[macro_use]
extern crate serde_derive;
extern crate serde;

use std::{env, io};
use std::net::SocketAddr;

use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio_codec::Decoder;

use p2p::{ChannelBuffer};

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8081".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr).expect("failed to bind");
    println!("Listening on: {}", addr);

    tokio::run({
        listener.incoming()
            .map_err(|e| println!("failed to accept socket; error = {:?}", e))
            .for_each(|socket| {
                process(socket, false);
                Ok(())
            })
    });

    let mut peer_addrs = Vec::new();
    let peer_addr1 = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let peer_addr1 = peer_addr1.parse::<SocketAddr>().unwrap();
    let peer_addr2 = env::args().nth(1).unwrap_or("127.0.0.1:8082".to_string());
    let peer_addr2 = peer_addr2.parse::<SocketAddr>().unwrap();

    peer_addrs.push(peer_addr1);
    peer_addrs.push(peer_addr2);

    for peer_addr in peer_addrs {
        let tcp = TcpStream::connect(&peer_addr)
            .map(move |socket| {
                println!("Connected on: {}", &peer_addr);
                process(socket, true);
            })
            .map_err(|e| println!("error reading stdout; error = {:?}", e));
        tokio::run(tcp);
    }
}

fn process(socket: TcpStream, is_sponsor: bool) {
    let (mut tx, rx) =
        p2p::P2p.framed(socket)
            .split();

    if is_sponsor {
        let mut req = ChannelBuffer::new();
        req.head.ver = 0x0000 as u16;
        req.head.ctrl = 0x00 as u8;
        req.head.action = 0x01 as u8;
        req.head.len = 4;
        req.body.push('a' as u8);
        req.body.push('i' as u8);
        req.body.push('o' as u8);
        req.body.push('n' as u8);

        tx.start_send(req).unwrap();
    }

    let task = tx.send_all(rx.and_then(respond))
        .then(|res| {
            if let Err(e) = res {
                println!("failed to process connection; error = {:?}", e);
            }

            Ok(())
        });

    tokio::spawn(task);
}

fn respond(req: ChannelBuffer)
           -> Box<Future<Item=ChannelBuffer, Error=io::Error> + Send>
{
    match req.head.ver {
        0 => {
            println!("Ver 0 package received.");
        }
        1 => {
            println!("Ver 1 package received.");
        }
        _ => {
            println!("Invalid Version {}.", req.head.ver);
        }
    };

    let mut res = ChannelBuffer::new();

    res.head.ver = req.head.ver;
    res.head.ctrl = req.head.ctrl;
    res.head.action = req.head.action;
    res.body = "sdfasdf".as_bytes().to_vec();
    res.head.len = res.body.len() as u32;

    println!("ver {}, ctrl {}, action {}, len {}.", res.head.ver, res.head.ctrl, res.head.action, res.head.len);

    Box::new(future::ok(res))
}

mod p2p {
    use tokio_codec::{Decoder, Encoder};
    use std::{io};
    use bytes::{BufMut, BytesMut};
    use bincode::{serialize, deserialize};

    pub struct P2p;

    #[derive(Serialize, Deserialize, PartialEq)]
    pub struct Head {
        pub ver: u16,
        pub ctrl: u8,
        pub action: u8,
        pub len: u32,
    }

    impl Head {
        pub fn new() -> Head {
            Head {
                ver: 0 as u16,
                ctrl: 0 as u8,
                action: 0 as u8,
                len: 0,
            }
        }
    }

    #[derive(Serialize, Deserialize, PartialEq)]
    pub struct ChannelBuffer {
        pub head: Head,
        pub body: Vec<u8>,
    }

    impl ChannelBuffer {
        pub fn new() -> ChannelBuffer {
            ChannelBuffer {
                head: Head::new(),
                body: Vec::new(),
            }
        }
    }

    impl Encoder for P2p {
        type Item = ChannelBuffer;
        type Error = io::Error;

        fn encode(&mut self, item: ChannelBuffer, dst: &mut BytesMut) -> io::Result<()> {
            let encoded: Vec<u8> = serialize(&item).unwrap();
            dst.put_slice(encoded.as_slice());

            return Ok(());
        }
    }

    impl Decoder for P2p {
        type Item = ChannelBuffer;
        type Error = io::Error;

        fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<ChannelBuffer>> {
            if src.len() > 0 {
                let encoded: Vec<u8> = src.to_vec();
                let decoded: ChannelBuffer = deserialize(&encoded[..]).unwrap();

                Ok(Some(decoded))
            } else {
                Ok(None)
            }
        }
    }
}
