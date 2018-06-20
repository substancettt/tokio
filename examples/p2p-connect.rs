//! An example of hooking up stdin/stdout to either a TCP or UDP stream.
//!
//! This example will connect to a socket address specified in the argument list
//! and then forward all data read on stdin to the server, printing out all data
//! received on stdout. An optional `--udp` argument can be passed to specify
//! that the connection should be made over UDP instead of TCP, translating each
//! line entered on stdin to a UDP packet to be sent to the remote address.
//!
//! Note that this is not currently optimized for performance, especially
//! around buffer management. Rather it's intended to show an example of
//! working with a client.
//!
//! This example can be quite useful when interacting with the other examples in
//! this repository! Many of them recommend running this as a simple "hook up
//! stdin/stdout to a server" to get up and running.

#![deny(warnings)]


extern crate tokio;
extern crate tokio_codec;
extern crate tokio_current_thread;
extern crate tokio_io;
extern crate futures;
extern crate bytes;


use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;

use std::env;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();
    let tcp = TcpStream::connect(&addr)
        .map(|stream| {
            let (_reader, writer) = stream.split();
            let (tx, _rx) = futures::sync::mpsc::unbounded();
            tx.unbounded_send("hehe".to_string()).unwrap();
            let tx_buf = format!("[Aion] test string...").to_string().into_bytes();
            let amt = io::write_all(writer, tx_buf);
            let _amt = amt.map(|(writer, _)| writer);
            println!("Connected");
        })
        .map_err(|e| println!("Failed to connect: {}", e))
    ;


    tokio_current_thread::spawn(tcp);
}
