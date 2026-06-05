use std::{
    net::{SocketAddr, SocketAddrV4, TcpListener as StdTcpListener},
    thread::{self, JoinHandle},
};

use embedder_traits::{EventLoopWaker, webdriver_bidi::WebDriverBidiToEmbedderMsg};
use log::info;
use tokio::net::TcpListener;

use crate::{
    dispatcher::{DispatchMessage, Dispatcher},
    handler::{Handler, WebDriverBidiHandler},
    server::serve,
};

pub mod connection;
pub mod dispatcher;
pub mod error;
pub mod handler;
pub mod model;
pub mod server;
pub mod session;

pub fn start_server(
    port: u16,
    embedder_tx: crossbeam_channel::Sender<WebDriverBidiToEmbedderMsg>,
    event_loop_waker: Box<dyn EventLoopWaker>,
) {
    let handler = Handler::new(event_loop_waker, embedder_tx);

    thread::Builder::new()
        .name("WebDriverBiDiServer".to_owned())
        .spawn(move || {
            let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), port);
            match start(SocketAddr::V4(address), handler) {
                Ok(listening) => {
                    info!("WebDriver BiDi server listening on {}", listening.socket)
                },
                Err(e) => {
                    panic!("Unable to start WebDriver BiDi server {e:?}");
                },
            }
        })
        .expect("Thread spawning failed");
}

/// A WebSocket Listener.
///
/// See <https://www.w3.org/TR/webdriver-bidi/#websocket-listener>.
pub struct Listener {
    guard: Option<thread::JoinHandle<()>>,
    /// Host and port
    pub socket: SocketAddr,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.guard.take().map(JoinHandle::join);
    }
}

/// To start listening for a WebSocket connection.
///
/// See <https://www.w3.org/TR/webdriver-bidi/#start-listening-for-a-websocket-connection>.
///
/// ## NOTE
///
/// Currently this implementation only supports [BiDi-only sesion](https://www.w3.org/TR/webdriver-bidi/#supports-bidi-only-sessions)
/// and does not support upgrade from WebDriver classic. So the only WebSocket resource is `/session`.
///
/// ## NOTE
///
/// WebDriver Bidi allows implementation to reuse existing listener, and there is no reason to
/// have multiple active listeners for non-intermediary node like servo, thus step 4 is ignored.
pub fn start<T>(
    mut address: SocketAddr,
    handler: T,
    // TODO: implementation defined check like allow_hosts
) -> ::std::io::Result<Listener>
where
    T: 'static + WebDriverBidiHandler,
{
    let listener = StdTcpListener::bind(address)?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    if address.port() == 0 {
        address.set_port(addr.port());
    }

    let (dispatch_tx, dispatch_rx) = crossbeam_channel::unbounded::<DispatchMessage>();

    let builder = thread::Builder::new().name("webdriver bidi server".to_string());
    let handle = builder.spawn({
        let dispatch_tx = dispatch_tx.clone();
        {
            move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .build()
                    .expect("fail to create tokio runtime");
                rt.block_on(async {
                    // from_std must be called in IO-enabled runtime
                    let listener = TcpListener::from_std(listener)
                        .expect("fail to convert TcpListener to tokio");
                    serve(listener, dispatch_tx).await
                });
            }
        }
    })?;

    let builder = thread::Builder::new().name("webdriver dispatcher".to_string());
    builder.spawn(move || {
        Dispatcher::new(handler, dispatch_tx, dispatch_rx).run();
    })?;

    Ok(Listener {
        guard: Some(handle),
        socket: addr,
    })
}
