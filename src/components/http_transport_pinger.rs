use libc;
use std::io::Error as IoError;
use std::os::raw::c_int;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{self as channel, Receiver, Sender};
use reqwest::{self, Client};
use serde_json;
use signal_hook;

use cluster::{Cluster, Pong};
use components::configuration::Configuration;
use node::{Node, UnknownNode};

use components;

fn notify(
    signals: &[c_int],
) -> Result<(Sender<c_int>, Receiver<c_int>), IoError> {
    let (s, r) = channel::bounded(100);
    let signals = signal_hook::iterator::Signals::new(signals)?;
    let sender = s.clone();
    thread::spawn(move || {
        for signal in signals.forever() {
            sender.send(signal);
        }
    });
    Ok((s, r))
}

#[derive(Debug)]
pub enum Error {
    HttpError(reqwest::Error),
    JsonError(serde_json::Error, String),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::JsonError(error, String::new())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        Error::HttpError(error)
    }
}

pub fn identify_peer(
    client: &Client,
    cluster: &mut Cluster,
    peer: &UnknownNode,
) -> Result<Node, Error> {
    let url = format!("http://{}/ping", peer.address);

    debug!("Sending {:?}", cluster.node);

    match client.post(&url).json(&cluster.node).send() {
        Ok(mut response) => {
            // Using text here because Response.json() consumes the response and
            // I want to use it multiple times.
            // https://github.com/seanmonstar/reqwest/issues/302
            let body = response.text()?;
            match serde_json::from_str::<Pong>(&body) {
                Ok(pong) => Ok(cluster.identify_peer(peer, pong.from)),
                Err(error) => Err(Error::JsonError(error, body)),
            }
        }
        Err(error) => Err(Error::HttpError(error)),
    }
}

fn ping_peer(client: &Client, node: &Node, peer: &Node) -> Result<Pong, Error> {
    let url = format!("http://{}/ping", peer.address);

    debug!("Sending {:?}", node);

    match client.post(&url).json(node).send() {
        Ok(mut response) => {
            // Using text here because Response.json() consumes the response and
            // I want to use it multiple times.
            // https://github.com/seanmonstar/reqwest/issues/302
            let body = response.text()?;
            match serde_json::from_str::<Pong>(&body) {
                Ok(pong) => Ok(pong),
                Err(error) => Err(Error::JsonError(error, body)),
            }
        }
        Err(error) => Err(Error::HttpError(error)),
    }
}

pub fn identify_peers(client: &Client) -> () {
    match components::cluster::STATE.write().unwrap().as_mut() {
        Some(cluster) => {
            let unknown_peers = cluster.unknown_peers.clone();
            for peer in unknown_peers {
                info!("Identifying {}", peer.address);
                match identify_peer(client, cluster, &peer) {
                    Ok(node) => {
                        info!(
                            "Identified peer: {} is {}",
                            node.name, peer.address
                        );
                    }
                    Err(error) => {
                        info!(
                            "Failed to identify {}: {:?}",
                            peer.address, error
                        );
                    }
                }
            }
        }
        None => (),
    }
}

fn ping_peers(client: &Client) -> () {
    match components::cluster::STATE.read().unwrap().as_ref() {
        Some(cluster) => {
            for peer in &cluster.peers {
                info!("Pinging {}", peer.name);
                match ping_peer(client, &cluster.node, &peer) {
                    Ok(pong) => {
                        info!("Got pong from {}", pong.from.name);
                    }
                    Err(error) => {
                        info!("Failed to ping {}: {:?}", peer.name, error);
                    }
                }
            }
        }
        None => (),
    }
}

pub fn start() -> () {
    let c = Configuration::read();

    let http_transport_pinger_connect_timeout =
        &c.http_transport_pinger_connect_timeout;
    let http_transport_pinger_schedule = c.http_transport_pinger_schedule;

    let duration = Duration::from_millis(http_transport_pinger_schedule);
    let ping_receiver = channel::tick(duration);
    let (_signal_sender, signal_receiver) =
        notify(&[libc::SIGINT, libc::SIGTERM]).unwrap();
    let client = Client::new();

    identify_peers(&client);

    loop {
        select! {
            recv(ping_receiver, ping) => match ping {
                Some(_ping) => {
                    debug!("ping_receiver got message");
                    identify_peers(&client);
                    ping_peers(&client);
                },
                None => error!("ping_receiver channel closed"),
            }
            recv(signal_receiver, signal) => match signal {
                Some(signal) => {
                    warn!("Received signal {:?}, exiting", signal);
                    break;
                },
                None => error!("signal_receiver channel closed"),
            }
        }
    }
}

pub fn spawn() -> JoinHandle<()> {
    thread::spawn(start)
}
