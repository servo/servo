use std::{net::SocketAddrV4, thread};

use embedder_traits::EventLoopWaker;

use crate::handler::Handler;

pub mod dispatcher;
pub mod error;
pub mod handler;
pub mod model;
pub mod server;
pub mod transport;

pub fn start_server(
    port: u16,
    event_loop_waker: Box<dyn EventLoopWaker>,
    embedder_tx: crossbeam_channel::Sender<()>,
    embedder_rx: crossbeam_channel::Receiver<()>,
) {
    let handler = Handler::new(event_loop_waker, embedder_tx, embedder_rx);

    thread::Builder::new()
        .name("WebDriverBiDiServer".to_owned())
        .spawn(move || {
            let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
            // TODO:
            // match server::start(SocketAddr::V4(address), handler) {
            //     Ok(_listening) => {
            //         // TODO: info
            //     },
            //     Err(e) => {
            //         panic!("Unable to start WebDriver BiDi server {e:?}");
            //     },
            // }
        })
        .expect("Thread spawning failed");
}
