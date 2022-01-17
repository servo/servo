/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::hosts::replace_host;
use crate::http_loader::HANDLE;
use futures::{task::Context, task::Poll, Future};
use http::uri::{Authority, Uri as Destination};
use hyper::client::HttpConnector as HyperHttpConnector;
use hyper::rt::Executor;
use hyper::{service::Service, Body, Client};
use hyper_openssl::HttpsConnector;
use openssl::ex_data::Index;
use openssl::ssl::{
    Ssl, SslConnector, SslConnectorBuilder, SslContext, SslMethod, SslOptions, SslVerifyMode,
};
use openssl::x509::{self, X509StoreContext};
use std::collections::hash_map::{Entry, HashMap};
use std::sync::{Arc, Mutex};

pub const BUF_SIZE: usize = 32768;
pub const ALPN_H2_H1: &'static [u8] = b"\x02h2\x08http/1.1";
pub const ALPN_H1: &'static [u8] = b"\x08http/1.1";

// See https://wiki.mozilla.org/Security/Server_Side_TLS for orientation.
const TLS1_2_CIPHERSUITES: &'static str = concat!(
    "ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:",
    "ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES128-GCM-SHA256:",
    "ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-GCM-SHA256:",
    "ECDHE-RSA-AES256-SHA:ECDHE-RSA-AES128-SHA@SECLEVEL=2"
);
const SIGNATURE_ALGORITHMS: &'static str = concat!(
    "ed448:ed25519:",
    "ECDSA+SHA384:ECDSA+SHA256:",
    "RSA-PSS+SHA512:RSA-PSS+SHA384:RSA-PSS+SHA256:",
    "RSA+SHA512:RSA+SHA384:RSA+SHA256"
);

#[derive(Clone)]
pub struct ConnectionCerts {
    certs: Arc<Mutex<HashMap<String, (Vec<u8>, u32)>>>,
}

impl ConnectionCerts {
    pub fn new() -> Self {
        Self {
            certs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn store(&self, host: String, cert_bytes: Vec<u8>) {
        let mut certs = self.certs.lock().unwrap();
        let entry = certs.entry(host).or_insert((cert_bytes, 0));
        entry.1 += 1;
    }

    pub(crate) fn remove(&self, host: String) -> Option<Vec<u8>> {
        match self.certs.lock().unwrap().entry(host) {
            Entry::Vacant(_) => return None,
            Entry::Occupied(mut e) => {
                e.get_mut().1 -= 1;
                if e.get().1 == 0 {
                    return Some((e.remove_entry().1).0);
                }
                Some(e.get().0.clone())
            },
        }
    }
}

#[derive(Clone)]
pub struct HttpConnector {
    inner: HyperHttpConnector,
}

impl HttpConnector {
    fn new() -> HttpConnector {
        let mut inner = HyperHttpConnector::new();
        inner.enforce_http(false);
        inner.set_happy_eyeballs_timeout(None);
        HttpConnector { inner }
    }
}

impl Service<Destination> for HttpConnector {
    type Response = <HyperHttpConnector as Service<Destination>>::Response;
    type Error = <HyperHttpConnector as Service<Destination>>::Error;
    type Future = <HyperHttpConnector as Service<Destination>>::Future;

    fn call(&mut self, dest: Destination) -> Self::Future {
        // Perform host replacement when making the actual TCP connection.
        let mut new_dest = dest.clone();
        let mut parts = dest.into_parts();

        if let Some(auth) = parts.authority {
            let host = auth.host();
            let host = replace_host(host);

            let authority = if let Some(port) = auth.port() {
                format!("{}:{}", host, port.as_str())
            } else {
                format!("{}", &*host)
            };

            if let Ok(authority) = Authority::from_maybe_shared(authority) {
                parts.authority = Some(authority);
                if let Ok(dest) = Destination::from_parts(parts) {
                    new_dest = dest
                }
            }
        }

        self.inner.call(new_dest)
    }

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }
}

pub type Connector = HttpsConnector<HttpConnector>;
pub type TlsConfig = SslConnectorBuilder;

#[derive(Clone)]
pub struct ExtraCerts(Arc<Mutex<Vec<Vec<u8>>>>);

impl ExtraCerts {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(vec![])))
    }

    pub fn add(&self, bytes: Vec<u8>) {
        self.0.lock().unwrap().push(bytes);
    }
}

struct Host(String);

lazy_static! {
    static ref EXTRA_INDEX: Index<SslContext, ExtraCerts> = SslContext::new_ex_index().unwrap();
    static ref CONNECTION_INDEX: Index<SslContext, ConnectionCerts> =
        SslContext::new_ex_index().unwrap();
    static ref HOST_INDEX: Index<Ssl, Host> = Ssl::new_ex_index().unwrap();
}

pub fn create_tls_config(
    certs: &str,
    alpn: &[u8],
    extra_certs: ExtraCerts,
    connection_certs: ConnectionCerts,
) -> TlsConfig {
    // certs include multiple certificates. We could add all of them at once,
    // but if any of them were already added, openssl would fail to insert all
    // of them.
    let mut certs = certs;
    let mut cfg = SslConnector::builder(SslMethod::tls()).unwrap();
    loop {
        let token = "-----END CERTIFICATE-----";
        if let Some(index) = certs.find(token) {
            let (cert, rest) = certs.split_at(index + token.len());
            certs = rest;
            let cert = x509::X509::from_pem(cert.as_bytes()).unwrap();
            cfg.cert_store_mut()
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
    cfg.set_alpn_protos(alpn)
        .expect("could not set alpn protocols");
    cfg.set_cipher_list(TLS1_2_CIPHERSUITES)
        .expect("could not set TLS 1.2 ciphersuites");
    cfg.set_sigalgs_list(SIGNATURE_ALGORITHMS)
        .expect("could not set signature algorithms");
    cfg.set_options(
        SslOptions::NO_SSLV2 |
            SslOptions::NO_SSLV3 |
            SslOptions::NO_TLSV1 |
            SslOptions::NO_TLSV1_1 |
            SslOptions::NO_COMPRESSION,
    );

    cfg.set_ex_data(*EXTRA_INDEX, extra_certs);
    cfg.set_ex_data(*CONNECTION_INDEX, connection_certs);
    cfg.set_verify_callback(SslVerifyMode::PEER, |verified, x509_store_context| {
        if verified {
            return true;
        }

        let ssl_idx = X509StoreContext::ssl_idx().unwrap();
        let ssl = x509_store_context.ex_data(ssl_idx).unwrap();

        // Obtain the cert bytes for this connection.
        let cert = match x509_store_context.current_cert() {
            Some(cert) => cert,
            None => return false,
        };
        let pem = match cert.to_pem() {
            Ok(pem) => pem,
            Err(_) => return false,
        };

        let ssl_context = ssl.ssl_context();

        // Ensure there's an entry stored in the set of known connection certs for this connection.
        if let Some(host) = ssl.ex_data(*HOST_INDEX) {
            let connection_certs = ssl_context.ex_data(*CONNECTION_INDEX).unwrap();
            connection_certs.store((*host).0.clone(), pem.clone());
        }

        // Fall back to the dynamic set of allowed certs.
        let extra_certs = ssl_context.ex_data(*EXTRA_INDEX).unwrap();
        for cert in &*extra_certs.0.lock().unwrap() {
            if pem == *cert {
                return true;
            }
        }
        false
    });

    cfg
}

struct TokioExecutor {}

impl<F> Executor<F> for TokioExecutor
where
    F: Future<Output = ()> + 'static + std::marker::Send,
{
    fn execute(&self, fut: F) {
        HANDLE.lock().unwrap().as_ref().unwrap().spawn(fut);
    }
}

pub fn create_http_client(tls_config: TlsConfig) -> Client<Connector, Body> {
    let mut connector = HttpsConnector::with_connector(HttpConnector::new(), tls_config).unwrap();
    connector.set_callback(|configuration, destination| {
        if let Some(host) = destination.host() {
            configuration.set_ex_data(*HOST_INDEX, Host(host.to_owned()));
        }
        Ok(())
    });

    Client::builder()
        .http1_title_case_headers(true)
        .executor(TokioExecutor {})
        .build(connector)
}
