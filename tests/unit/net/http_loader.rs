/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net::http_loader::{load, LoadError};
use hyper::net::{NetworkStream, NetworkConnector};
use hyper;
use url::Url;
use std::io::{self, Read, Write, Cursor};
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use ipc_channel::ipc;
use net_traits::LoadData;
use net::hsts::HSTSList;
use net::hsts::HSTSEntry;

#[derive(Clone)]
pub struct MockStream {
    pub read: Cursor<Vec<u8>>,
    pub write: Vec<u8>,
    #[cfg(feature = "timeouts")]
    pub read_timeout: Cell<Option<Duration>>,
    #[cfg(feature = "timeouts")]
    pub write_timeout: Cell<Option<Duration>>
}

impl fmt::Debug for MockStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MockStream {{ read: {:?}, write: {:?} }}", self.read.get_ref(), self.write)
    }
}

impl PartialEq for MockStream {
    fn eq(&self, other: &MockStream) -> bool {
        self.read.get_ref() == other.read.get_ref() && self.write == other.write
    }
}

impl MockStream {
    pub fn new() -> MockStream {
        MockStream::with_input(b"")
    }

    #[cfg(not(feature = "timeouts"))]
    pub fn with_input(input: &[u8]) -> MockStream {
        MockStream {
            read: Cursor::new(input.to_vec()),
            write: vec![]
        }
    }

    #[cfg(feature = "timeouts")]
    pub fn with_input(input: &[u8]) -> MockStream {
        MockStream {
            read: Cursor::new(input.to_vec()),
            write: vec![],
            read_timeout: Cell::new(None),
            write_timeout: Cell::new(None),
        }
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read.read(buf)
    }
}

impl Write for MockStream {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        Write::write(&mut self.write, msg)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl NetworkStream for MockStream {
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        Ok("127.0.0.1:1337".parse().unwrap())
    }

    #[cfg(feature = "timeouts")]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.read_timeout.set(dur);
        Ok(())
    }

    #[cfg(feature = "timeouts")]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.write_timeout.set(dur);
        Ok(())
    }
}

/// A wrapper around a `MockStream` that allows one to clone it and keep an independent copy to the
/// same underlying stream.
#[derive(Clone)]
pub struct CloneableMockStream {
    pub inner: Arc<Mutex<MockStream>>,
}

impl Write for CloneableMockStream {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().write(msg)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}

impl Read for CloneableMockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().read(buf)
    }
}

impl NetworkStream for CloneableMockStream {
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        self.inner.lock().unwrap().peer_addr()
    }

    #[cfg(feature = "timeouts")]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.inner.lock().unwrap().set_read_timeout(dur)
    }

    #[cfg(feature = "timeouts")]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.inner.lock().unwrap().set_write_timeout(dur)
    }
}

impl CloneableMockStream {
    pub fn with_stream(stream: MockStream) -> CloneableMockStream {
        CloneableMockStream {
            inner: Arc::new(Mutex::new(stream)),
        }
    }
}

pub struct MockConnector;

impl NetworkConnector for MockConnector {
    type Stream = MockStream;

    fn connect(&self, _host: &str, _port: u16, _scheme: &str) -> hyper::Result<MockStream> {
        Ok(MockStream::new())
    }
}

#[test]
fn test_load_errors_when_scheme_is_not_http_or_https() {
    let url = Url::parse("ftp://not-supported").unwrap();
    let (cookies_chan, _) = ipc::channel().unwrap();
    let load_data = LoadData::new(url, None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load(load_data, cookies_chan, None, hsts_list, &MockConnector) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}

#[test]
fn test_load_errors_when_viewing_source_and_inner_url_scheme_is_not_http_or_https() {
    let url = Url::parse("view-source:ftp://not-supported").unwrap();
    let (cookies_chan, _) = ipc::channel().unwrap();
    let load_data = LoadData::new(url, None);
    let hsts_list = Arc::new(Mutex::new(HSTSList { entries: Vec::new() }));

    match load(load_data, cookies_chan, None, hsts_list, &MockConnector) {
        Err(LoadError::UnsupportedScheme(_)) => {}
        _ => panic!("expected ftp scheme to be unsupported")
    }
}
