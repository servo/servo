/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::hosts::replace_host;
use hyper::client::connect::{Connect, Destination};
use hyper::client::HttpConnector as HyperHttpConnector;
use hyper::rt::Future;
use hyper::{Body, Client};
use hyper_openssl::HttpsConnector;
use openssl::ssl::{SslConnector, SslConnectorBuilder, SslMethod, SslOptions};
use openssl::x509;
use tokio::prelude::future::Executor;

pub const BUF_SIZE: usize = 32768;

pub struct HttpConnector {
    inner: HyperHttpConnector,
}

impl HttpConnector {
    fn new() -> HttpConnector {
        let mut inner = HyperHttpConnector::new(4);
        inner.enforce_http(false);
        inner.set_happy_eyeballs_timeout(None);
        HttpConnector { inner }
    }
}

impl Connect for HttpConnector {
    type Transport = <HyperHttpConnector as Connect>::Transport;
    type Error = <HyperHttpConnector as Connect>::Error;
    type Future = <HyperHttpConnector as Connect>::Future;

    fn connect(&self, dest: Destination) -> Self::Future {
        // Perform host replacement when making the actual TCP connection.
        let mut new_dest = dest.clone();
        let addr = replace_host(dest.host());
        new_dest.set_host(&*addr).unwrap();
        self.inner.connect(new_dest)
    }
}

pub type Connector = HttpsConnector<HttpConnector>;

pub fn create_ssl_connector_builder(certs: &str) -> SslConnectorBuilder {
    // certs include multiple certificates. We could add all of them at once,
    // but if any of them were already added, openssl would fail to insert all
    // of them.
    let mut certs = certs;
    let mut ssl_connector_builder = SslConnector::builder(SslMethod::tls()).unwrap();
    loop {
        let token = "-----END CERTIFICATE-----";
        if let Some(index) = certs.find(token) {
            let (cert, rest) = certs.split_at(index + token.len());
            certs = rest;
            let cert = x509::X509::from_pem(cert.as_bytes()).unwrap();
            ssl_connector_builder
                .cert_store_mut()
                .add_cert(cert)
                .or_else(|e| {
                    let v: Option<Option<&str>> = e.errors().iter().nth(0).map(|e| e.reason());
                    if v == Some(Some("cert already in hash table")) {
                        warn!("Cert already in hash table. Ignoring.");
                        // Ignore error X509_R_CERT_ALREADY_IN_HASH_TABLE which means the
                        // certificate is already in the store.
                        Ok(())
                    } else {
                        Err(e)
                    }
                })
                .expect("could not set CA file");
        } else {
            break;
        }
    }
    ssl_connector_builder
        .set_cipher_list(DEFAULT_CIPHERS)
        .expect("could not set ciphers");
    ssl_connector_builder
        .set_options(SslOptions::NO_SSLV2 | SslOptions::NO_SSLV3 | SslOptions::NO_COMPRESSION);
    ssl_connector_builder
}

pub fn create_http_client<E>(
    ssl_connector_builder: SslConnectorBuilder,
    executor: E,
) -> Client<Connector, Body>
where
    E: Executor<Box<dyn Future<Error = (), Item = ()> + Send + 'static>> + Sync + Send + 'static,
{
    let connector =
        HttpsConnector::with_connector(HttpConnector::new(), ssl_connector_builder).unwrap();
    Client::builder()
        .http1_title_case_headers(true)
        .executor(executor)
        .build(connector)
}

// Prefer Forward Secrecy over plain RSA, AES-GCM over AES-CBC, ECDSA over RSA.
// A complete discussion of the issues involved in TLS configuration can be found here:
// https://wiki.mozilla.org/Security/Server_Side_TLS
const DEFAULT_CIPHERS: &'static str = concat!(
    "ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:",
    "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:",
    "ECDHE-RSA-AES256-SHA:ECDHE-RSA-AES128-SHA:AES256-SHA:AES128-SHA"
);
