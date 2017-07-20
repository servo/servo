/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hosts::replace_host;
use hyper::client::Pool;
use hyper::error::{Result as HyperResult, Error as HyperError};
use hyper::net::{NetworkConnector, HttpsStream, HttpStream, SslClient};
use hyper_sync_rustls::TlsClient;
use rustls;
use std::{io, fs};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

pub struct HttpsConnector {
    ssl: TlsClient,
}

impl HttpsConnector {
    fn new(ssl: TlsClient) -> HttpsConnector {
        HttpsConnector {
            ssl: ssl,
        }
    }
}

impl NetworkConnector for HttpsConnector {
    type Stream = HttpsStream<<TlsClient as SslClient>::Stream>;

    fn connect(&self, host: &str, port: u16, scheme: &str) -> HyperResult<Self::Stream> {
        if scheme != "http" && scheme != "https" {
            return Err(HyperError::Io(io::Error::new(io::ErrorKind::InvalidInput,
                                                     "Invalid scheme for Http")));
        }

        // Perform host replacement when making the actual TCP connection.
        let addr = &(&*replace_host(host), port);
        let stream = HttpStream(TcpStream::connect(addr)?);

        if scheme == "http" {
            Ok(HttpsStream::Http(stream))
        } else {
            // Do not perform host replacement on the host that is used
            // for verifying any SSL certificate encountered.
            self.ssl.wrap_client(stream, host).map(HttpsStream::Https)
        }
    }
}

pub type Connector = HttpsConnector;

pub fn create_ssl_client(ca_file: &PathBuf) -> TlsClient {
    let mut ca = {
        let f = fs::File::open(ca_file).expect("cannot open CA file");
        io::BufReader::new(f)
    };
    let mut tls = rustls::ClientConfig::new();
    tls.root_store.add_pem_file(&mut ca).unwrap();
    TlsClient { cfg: Arc::new(tls) }
}

pub fn create_http_connector(ssl_client: TlsClient ) -> Pool<Connector> {
    let https_connector = HttpsConnector::new(ssl_client);
    Pool::with_connector(Default::default(), https_connector)
}
