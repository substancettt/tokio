//! A chat server that broadcasts a message to all connections.
//!
//! This is a line-based server which accepts connections, reads lines from
//! those connections, and broadcasts the lines to all other connected clients.
//!
//! This example is similar to chat.rs, but uses combinators and a much more
//! functional style.
//!
//! You can test this out by running:
//!
//!     cargo run --example chat
//!
//! And then in another window run:
//!
//!     cargo run --example connect 127.0.0.1:8080
//!
//! You can run the second command in multiple windows and then chat between the
//! two, seeing the messages from the other client as they're received. For all
//! connected clients they'll all join the same room and see everyone else's
//! messages.

#![deny(warnings)]

extern crate tokio;
extern crate futures;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

use std::collections::HashMap;
use std::iter;
use std::env;
use std::io::{BufReader};
use std::sync::{Arc, Mutex};

fn main() {
    // Create the TCP listener we'll accept connections on.
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let socket = TcpListener::bind(&addr).unwrap();
    println!("Listening on: {}", addr);

    let connections = Arc::new(Mutex::new(HashMap::new()));

    let srv = socket.incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |stream| {
            // The client's socket address
            let addr = stream.peer_addr().unwrap();

            println!("New Connection: {}", addr);

            let (reader, writer) = stream.split();
            let (tx, rx) = futures::sync::mpsc::unbounded();
            connections.lock().unwrap().insert(addr, tx);


            let connections_inner = connections.clone();
            let reader = BufReader::new(reader);

            let iter = stream::iter_ok::<_, io::Error>(iter::repeat(()));

            let socket_reader = iter.fold(reader, move |reader, _| {
//                let line = io::read_to_end(reader, Vec::new());
                let line = io::read_until(reader, b'\n', Vec::new());
                let line = line.and_then(|(reader, vec)| {
                    if vec.len() == 0 {
                        Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
                    } else {
                        Ok((reader, vec))
                    }
                });

                let line = line.map(|(reader, vec)| {
                    (reader, String::from_utf8(vec))
                });

                // Move the connection state into the closure below.
                let connections = connections_inner.clone();

                line.map(move |(reader, message)| {
                    let mut conns = connections.lock().unwrap();

                    if let Ok(msg) = message {
                        println!("Inbound Nodes:  ");
                        for (add, _) in conns.iter_mut() {
                            if add.eq(&addr) {
                                println!("*  {}", add);
                            } else {
                                println!(" {}", add);
                            }
                        }
                        println!("====================");

                        let iter = conns.iter_mut()
                            .filter(|(&k, _)| k.eq(&addr) )
                            .map(|(_, v)| v);
                        for tx in iter {
                            println!("msg: {}", msg);
                            tx.unbounded_send(msg.to_string()).unwrap();
                        }
                    } else {
                        let tx = conns.get_mut(&addr).unwrap();
                        tx.unbounded_send("You didn't send valid UTF-8.".to_string()).unwrap();
                    }
                    reader
                })
            });

            // Whenever we receive a string on the Receiver, we write it to
            // `WriteHalf<TcpStream>`.
            let socket_writer = rx.fold(writer, |writer, msg| {
                let tx_buf = format!("[Aion] {}", msg).to_string().into_bytes();
                let amt = io::write_all(writer, tx_buf);
                let amt = amt.map(|(writer, _)| writer);
                amt.map_err(|_| ())
            });

            // Now that we've got futures representing each half of the socket, we
            // use the `select` combinator to wait for either half to be done to
            // tear down the other. Then we spawn off the result.
            let connections = connections.clone();
            let socket_reader = socket_reader.map_err(|_| ());
            let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));

            // Spawn a task to process the connection
            tokio::spawn(connection.then(move |_| {
                connections.lock().unwrap().remove(&addr);
                println!("Connection {} closed.", addr);
                Ok(())
            }));

            Ok(())
        });

    // execute server
    tokio::run(srv);
}
