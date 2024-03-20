/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use futures::task::{Context, Poll};
use futures::Future;
use http::uri::{Authority, Uri as Destination};
use hyper::client::HttpConnector as HyperHttpConnector;
use hyper::rt::Executor;
use hyper::service::Service;
use hyper::{Body, Client};
use hyper_rustls::HttpsConnector as HyperRustlsHttpsConnector;
use log::warn;
use rustls::client::WebPkiVerifier;
use rustls::{Certificate, ClientConfig, OwnedTrustAnchor, RootCertStore, ServerName};

use crate::async_runtime::HANDLE;
use crate::hosts::replace_host;

pub const BUF_SIZE: usize = 32768;

#[derive(Clone)]
pub struct ServoHttpConnector {
    inner: HyperHttpConnector,
}

impl ServoHttpConnector {
    fn new() -> ServoHttpConnector {
        let mut inner = HyperHttpConnector::new();
        inner.enforce_http(false);
        inner.set_happy_eyeballs_timeout(None);
        ServoHttpConnector { inner }
    }
}

impl Service<Destination> for ServoHttpConnector {
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
                (*host).to_string()
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

pub type Connector = HyperRustlsHttpsConnector<ServoHttpConnector>;
pub type TlsConfig = ClientConfig;

#[derive(Clone, Debug, Default)]
struct CertificateErrorOverrideManagerInternal {
    /// A mapping of certificates and their hosts, which have seen certificate errors.
    /// This is used to later create an override in this [CertificateErrorOverrideManager].
    certificates_failing_to_verify: HashMap<ServerName, Certificate>,
    /// A list of certificates that should be accepted despite encountering verification
    /// errors.
    overrides: Vec<Certificate>,
}

/// This data structure is used to track certificate verification errors and overrides.
/// It tracks:
///  - A list of [Certificate]s with verification errors mapped by their [ServerName]
///  - A list of [Certificate]s for which to ignore verification errors.
#[derive(Clone, Debug, Default)]
pub struct CertificateErrorOverrideManager(Arc<Mutex<CertificateErrorOverrideManagerInternal>>);

impl CertificateErrorOverrideManager {
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Add a certificate to this manager's list of certificates for which to ignore
    /// validation errors.
    pub fn add_override(&self, certificate: &Certificate) {
        self.0.lock().unwrap().overrides.push(certificate.clone());
    }

    /// Given the a string representation of a sever host name, remove information about
    /// a [Certificate] with verification errors. If a certificate with
    /// verification errors was found, return it, otherwise None.
    pub(crate) fn remove_certificate_failing_verification(
        &self,
        host: &str,
    ) -> Option<Certificate> {
        let server_name = match ServerName::try_from(host) {
            Ok(name) => name,
            Err(error) => {
                warn!("Could not convert host string into RustTLS ServerName: {error:?}");
                return None;
            },
        };
        self.0
            .lock()
            .unwrap()
            .certificates_failing_to_verify
            .remove(&server_name)
    }
}

#[derive(Clone, Debug)]
pub enum CACertificates {
    Default,
    Override(RootCertStore),
}

/// Create a [TlsConfig] to use for managing a HTTP connection. This currently creates
/// a rustls [ClientConfig].
///
/// FIXME: The `ignore_certificate_errors` argument ignores all certificate errors. This
/// is used when running the WPT tests, because rustls currently rejects the WPT certificiate.
/// See <https://github.com/servo/servo/issues/30080>
pub fn create_tls_config(
    ca_certificates: CACertificates,
    ignore_certificate_errors: bool,
    override_manager: CertificateErrorOverrideManager,
) -> TlsConfig {
    let verifier = CertificateVerificationOverrideVerifier::new(
        ca_certificates,
        ignore_certificate_errors,
        override_manager,
    );
    rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth()
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

struct CertificateVerificationOverrideVerifier {
    webpki_verifier: WebPkiVerifier,
    ignore_certificate_errors: bool,
    override_manager: CertificateErrorOverrideManager,
}

impl CertificateVerificationOverrideVerifier {
    fn new(
        ca_certficates: CACertificates,
        ignore_certificate_errors: bool,
        override_manager: CertificateErrorOverrideManager,
    ) -> Self {
        let root_cert_store = match ca_certficates {
            CACertificates::Default => {
                let mut root_cert_store = rustls::RootCertStore::empty();
                root_cert_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(
                    |trust_anchor| {
                        OwnedTrustAnchor::from_subject_spki_name_constraints(
                            trust_anchor.subject,
                            trust_anchor.spki,
                            trust_anchor.name_constraints,
                        )
                    },
                ));
                root_cert_store
            },
            CACertificates::Override(root_cert_store) => root_cert_store,
        };

        Self {
            // See https://github.com/rustls/rustls/blame/v/0.21.6/rustls/src/client/builder.rs#L141
            // This is the default verifier for Rustls that we are wrapping.
            webpki_verifier: WebPkiVerifier::new(root_cert_store, None),
            ignore_certificate_errors,
            override_manager,
        }
    }
}

impl rustls::client::ServerCertVerifier for CertificateVerificationOverrideVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        intermediates: &[Certificate],
        server_name: &ServerName,
        scts: &mut dyn Iterator<Item = &[u8]>,
        ocsp_response: &[u8],
        now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        let error = match self.webpki_verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            scts,
            ocsp_response,
            now,
        ) {
            Ok(result) => return Ok(result),
            Err(error) => error,
        };

        if self.ignore_certificate_errors {
            warn!("Ignoring certficate error: {error:?}");
            return Ok(rustls::client::ServerCertVerified::assertion());
        }

        // If there's an override for this certificate, just accept it.
        for cert_with_exception in &*self.override_manager.0.lock().unwrap().overrides {
            if *end_entity == *cert_with_exception {
                return Ok(rustls::client::ServerCertVerified::assertion());
            }
        }
        self.override_manager
            .0
            .lock()
            .unwrap()
            .certificates_failing_to_verify
            .insert(server_name.clone(), end_entity.clone());
        Err(error)
    }
}

pub fn create_http_client(tls_config: TlsConfig) -> Client<Connector, Body> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls_config)
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .wrap_connector(ServoHttpConnector::new());

    Client::builder()
        .http1_title_case_headers(true)
        .executor(TokioExecutor {})
        .build(connector)
}
