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

use std::{env, io};
use std::net::SocketAddr;

use tokio::net::{TcpStream, TcpListener};
use tokio::prelude::*;
use tokio_codec::Decoder;

use p2p_aion::{ChannelBuffer};

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    let listener = TcpListener::bind(&addr).expect("failed to bind");
    println!("Listening on: {}", addr);

    tokio::run({
        listener.incoming()
            .map_err(|e| println!("failed to accept socket; error = {:?}", e))
            .for_each(|socket| {
                process(socket);
                Ok(())
            })
    });
}

fn process(socket: TcpStream) {
    let (tx, rx) =
        p2p::P2p.framed(socket)
            .split();

    let task = tx.send_all(rx.and_then(respond))
        .then(|res| {
            if let Err(e) = res {
                println!("failed to process connection; error = {:?}", e);
            }

            Ok(())
        });

    // Spawn the task that handles the connection.
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