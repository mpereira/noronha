use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate serde_json;

fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();

    let host = "0.0.0.0";
    let port = 6500;
    let address = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&address).unwrap();

    info!("listening at {}", address);
    info!("started");

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(mut stream: TcpStream) {
    let response = json!({
        "cluster_name" : "noronha",
        "node_name" : "noronha-0",
        "noronha_version" : "0.1.0"
    });

    stream
        .write(serde_json::to_string_pretty(&response).unwrap().as_bytes())
        .unwrap();
    stream.write("\n".as_bytes()).unwrap();
    stream.flush().unwrap();
}
