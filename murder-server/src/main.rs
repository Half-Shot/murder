#[macro_use]
extern crate slog;
extern crate slog_term;
#[macro_use]
extern crate slog_scope;
extern crate protobuf;
extern crate uuid;
extern crate murder;

use slog::*;
use std::net::{TcpListener};
use std::time::{Duration};
use std::thread::spawn;

mod gameserver;
mod packet;
mod pb_murder;
mod packetphase;

fn setup_logger() -> Logger {
    let drain = slog_term::streamer().async().full().build();
    Logger::root(LevelFilter::new(drain, Level::Debug).fuse(), o!(
        "version" => env!("CARGO_PKG_VERSION"),
    ))
}

fn main() {
    let root = setup_logger();
    let log = root.new(o!{});
    slog_scope::set_global_logger(log);
    let listener = TcpListener::bind("127.0.0.1:1337").unwrap();
    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("Got connection, spawning thread.");
                spawn(move || {
                    let mut gs = gameserver::GameServer::new();
                    stream.set_read_timeout(Some(Duration::from_millis(100000))).unwrap();
                    stream.set_nonblocking(false).unwrap();
                    gs.run(stream);
                });
            }
            Err(e) => { error!("Failed to attach stream: {:?}", e) /* connection failed */ }
        }
    }
}
