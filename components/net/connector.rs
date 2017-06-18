/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hosts::replace_host;
use hyper::client::Pool;
use hyper::error::{Result as HyperResult, Error as HyperError};
use hyper::net::{NetworkConnector, HttpsStream, HttpStream, SslClient};
use hyper_openssl::OpensslClient;
use openssl::ssl::{SSL_OP_NO_COMPRESSION, SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3};
use openssl::ssl::{SslConnectorBuilder, SslMethod};
use std::io;
use std::net::TcpStream;
use std::path::PathBuf;

pub struct HttpsConnector {
    ssl: OpensslClient,
}

impl HttpsConnector {
    fn new(ssl: OpensslClient) -> HttpsConnector {
        HttpsConnector {
            ssl: ssl,
        }
    }
}

impl NetworkConnector for HttpsConnector {
    type Stream = HttpsStream<<OpensslClient as SslClient>::Stream>;

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

pub fn create_ssl_client(ca_file: &PathBuf) -> OpensslClient {
    let mut ssl_connector_builder = SslConnectorBuilder::new(SslMethod::tls()).unwrap();
    {
        let context = ssl_connector_builder.builder_mut();
        context.set_ca_file(ca_file).expect("could not set CA file");
        context.set_cipher_list(DEFAULT_CIPHERS).expect("could not set ciphers");
        context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);
    }
    let ssl_connector = ssl_connector_builder.build();
    OpensslClient::from(ssl_connector)
}

pub fn create_http_connector(ssl_client: OpensslClient) -> Pool<Connector> {
    let https_connector = HttpsConnector::new(ssl_client);
    Pool::with_connector(Default::default(), https_connector)
}

// The basic logic here is to prefer ciphers with ECDSA certificates, Forward
// Secrecy, AES GCM ciphers, AES ciphers, and finally 3DES ciphers.
// A complete discussion of the issues involved in TLS configuration can be found here:
// https://wiki.mozilla.org/Security/Server_Side_TLS
const DEFAULT_CIPHERS: &'static str = concat!(
    "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:",
    "ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:",
    "DHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES128-SHA256:",
    "ECDHE-RSA-AES128-SHA256:ECDHE-ECDSA-AES256-SHA384:ECDHE-RSA-AES256-SHA384:",
    "ECDHE-ECDSA-AES128-SHA:ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA:",
    "ECDHE-RSA-AES256-SHA:DHE-RSA-AES128-SHA256:DHE-RSA-AES128-SHA:",
    "DHE-RSA-AES256-SHA256:DHE-RSA-AES256-SHA:ECDHE-RSA-DES-CBC3-SHA:",
    "ECDHE-ECDSA-DES-CBC3-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:",
    "AES128-SHA256:AES256-SHA256:AES128-SHA:AES256-SHA"
);
