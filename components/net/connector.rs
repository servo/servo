/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::{fmt, io};

use futures::Future;
use futures::task::{Context, Poll};
use http::uri::{Authority, Uri as Destination};
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::rt::Executor;
use hyper_rustls::HttpsConnector as HyperRustlsHttpsConnector;
use hyper_rustls::MaybeHttpsStream;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::{
    Connected, Connection, HttpConnector as HyperHttpConnector,
};
use log::warn;
use rustls::client::{ClientConnection, EchStatus, WebPkiServerVerifier};
use rustls::{ClientConfig, ProtocolVersion, RootCertStore};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use tower_service::Service;

use crate::async_runtime::spawn_task;
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

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct InstrumentedConnector<T> {
    inner: HyperRustlsHttpsConnector<T>,
}

impl<T> InstrumentedConnector<T> {
    fn new(inner: HyperRustlsHttpsConnector<T>) -> Self {
        Self { inner }
    }
}

impl<T> From<HyperRustlsHttpsConnector<T>> for InstrumentedConnector<T> {
    fn from(inner: HyperRustlsHttpsConnector<T>) -> Self {
        Self::new(inner)
    }
}

pub struct InstrumentedStream<T> {
    inner: MaybeHttpsStream<T>,
    tls_info: Option<TlsHandshakeInfo>,
}

impl<T: Unpin> Unpin for InstrumentedStream<T> {}

impl<T> fmt::Debug for InstrumentedStream<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedStream")
            .field("tls_info", &self.tls_info)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct TlsHandshakeInfo {
    pub protocol_version: Option<String>,
    pub cipher_suite: Option<String>,
    pub kea_group_name: Option<String>,
    pub signature_scheme_name: Option<String>,
    pub alpn_protocol: Option<String>,
    pub certificate_chain_der: Vec<Vec<u8>>,
    pub used_ech: bool,
}

impl TlsHandshakeInfo {
    fn from_connection(conn: &ClientConnection) -> Self {
        let protocol_version = conn.protocol_version().map(protocol_version_to_string);
        let cipher_suite = conn
            .negotiated_cipher_suite()
            .map(|suite| format!("{:?}", suite.suite()));
        let kea_group_name = conn
            .negotiated_key_exchange_group()
            .map(|group| format!("{:?}", group.name()));
        let certificate_chain_der = conn
            .peer_certificates()
            .map(|certs| certs.iter().map(|cert| cert.as_ref().to_vec()).collect())
            .unwrap_or_default();
        let alpn_protocol = conn
            .alpn_protocol()
            .map(|proto| String::from_utf8_lossy(proto).into_owned());
        let used_ech = matches!(conn.ech_status(), EchStatus::Accepted);

        Self {
            protocol_version,
            cipher_suite,
            kea_group_name,
            signature_scheme_name: None,
            alpn_protocol,
            certificate_chain_der,
            used_ech,
        }
    }
}

impl<T> InstrumentedStream<T>
where
    T: Connection + hyper::rt::Read + hyper::rt::Write + Unpin,
{
    fn from_maybe_https_stream(stream: MaybeHttpsStream<T>) -> Self {
        match stream {
            MaybeHttpsStream::Http(inner) => Self {
                inner: MaybeHttpsStream::Http(inner),
                tls_info: None,
            },
            MaybeHttpsStream::Https(tls_stream) => {
                let tls_info = {
                    let (_, conn) = tls_stream.inner().get_ref();
                    TlsHandshakeInfo::from_connection(conn)
                };
                Self {
                    inner: MaybeHttpsStream::Https(tls_stream),
                    tls_info: Some(tls_info),
                }
            },
        }
    }
}

impl<T> Connection for InstrumentedStream<T>
where
    T: Connection + hyper::rt::Read + hyper::rt::Write + Unpin,
{
    fn connected(&self) -> Connected {
        let connected = match &self.inner {
            MaybeHttpsStream::Http(stream) => stream.connected(),
            MaybeHttpsStream::Https(stream) => {
                let (tcp, tls) = stream.inner().get_ref();
                if tls.alpn_protocol() == Some(b"h2") {
                    tcp.inner().connected().negotiated_h2()
                } else {
                    tcp.inner().connected()
                }
            },
        };
        if let Some(info) = &self.tls_info {
            connected.extra(info.clone())
        } else {
            connected
        }
    }
}

impl<T> hyper::rt::Read for InstrumentedStream<T>
where
    T: Connection + hyper::rt::Read + hyper::rt::Write + Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match &mut self.get_mut().inner {
            MaybeHttpsStream::Http(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            MaybeHttpsStream::Https(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl<T> hyper::rt::Write for InstrumentedStream<T>
where
    T: Connection + hyper::rt::Read + hyper::rt::Write + Unpin,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match &mut self.get_mut().inner {
            MaybeHttpsStream::Http(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            MaybeHttpsStream::Https(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match &mut self.get_mut().inner {
            MaybeHttpsStream::Http(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            MaybeHttpsStream::Https(stream) => std::pin::Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        match &mut self.get_mut().inner {
            MaybeHttpsStream::Http(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            MaybeHttpsStream::Https(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
        }
    }

    fn is_write_vectored(&self) -> bool {
        match &self.inner {
            MaybeHttpsStream::Http(stream) => stream.is_write_vectored(),
            MaybeHttpsStream::Https(stream) => stream.is_write_vectored(),
        }
    }

    fn poll_write_vectored(
        self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        match &mut self.get_mut().inner {
            MaybeHttpsStream::Http(stream) => {
                std::pin::Pin::new(stream).poll_write_vectored(cx, bufs)
            },
            MaybeHttpsStream::Https(stream) => {
                std::pin::Pin::new(stream).poll_write_vectored(cx, bufs)
            },
        }
    }
}

impl<T> Service<Destination> for InstrumentedConnector<T>
where
    T: Service<Destination>,
    T::Response: Connection + hyper::rt::Read + hyper::rt::Write + Send + Unpin + 'static,
    T::Future: Send + 'static,
    T::Error: Into<BoxError>,
{
    type Response = InstrumentedStream<T::Response>;
    type Error = BoxError;
    type Future = std::pin::Pin<
        Box<dyn Future<Output = Result<InstrumentedStream<T::Response>, BoxError>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, dst: Destination) -> Self::Future {
        let future = self.inner.call(dst);
        Box::pin(async move {
            let stream = future.await.map_err(|error| -> BoxError { error.into() })?;
            Ok(InstrumentedStream::from_maybe_https_stream(stream))
        })
    }
}

fn protocol_version_to_string(version: ProtocolVersion) -> String {
    match version {
        ProtocolVersion::TLSv1_3 => "TLS 1.3".to_string(),
        ProtocolVersion::TLSv1_2 => "TLS 1.2".to_string(),
        ProtocolVersion::TLSv1_1 => "TLS 1.1".to_string(),
        ProtocolVersion::TLSv1_0 => "TLS 1.0".to_string(),
        ProtocolVersion::SSLv3 => "SSL 3.0".to_string(),
        ProtocolVersion::DTLSv1_2 => "DTLS 1.2".to_string(),
        ProtocolVersion::DTLSv1_0 => "DTLS 1.0".to_string(),
        other => format!("{other:?}"),
    }
}

pub type Connector = InstrumentedConnector<ServoHttpConnector>;
pub type TlsConfig = ClientConfig;

#[derive(Clone, Debug, Default)]
struct CertificateErrorOverrideManagerInternal {
    /// A mapping of certificates and their hosts, which have seen certificate errors.
    /// This is used to later create an override in this [CertificateErrorOverrideManager].
    certificates_failing_to_verify: HashMap<ServerName<'static>, CertificateDer<'static>>,
    /// A list of certificates that should be accepted despite encountering verification
    /// errors.
    overrides: Vec<CertificateDer<'static>>,
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
    pub fn add_override(&self, certificate: &CertificateDer<'static>) {
        self.0.lock().unwrap().overrides.push(certificate.clone());
    }

    /// Given the a string representation of a sever host name, remove information about
    /// a [Certificate] with verification errors. If a certificate with
    /// verification errors was found, return it, otherwise None.
    pub(crate) fn remove_certificate_failing_verification(
        &self,
        host: &str,
    ) -> Option<CertificateDer<'static>> {
        let server_name = match ServerName::try_from(host) {
            Ok(name) => name.to_owned(),
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
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth()
}

#[derive(Clone)]
struct TokioExecutor {}

impl<F> Executor<F> for TokioExecutor
where
    F: Future<Output = ()> + 'static + std::marker::Send,
{
    fn execute(&self, fut: F) {
        spawn_task(fut);
    }
}

#[derive(Debug)]
struct CertificateVerificationOverrideVerifier {
    webpki_verifier: Arc<WebPkiServerVerifier>,
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
            CACertificates::Default => rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            },
            CACertificates::Override(root_cert_store) => root_cert_store,
        };

        Self {
            // See https://github.com/rustls/rustls/blame/v/0.21.6/rustls/src/client/builder.rs#L141
            // This is the default verifier for Rustls that we are wrapping.
            webpki_verifier: WebPkiServerVerifier::builder(root_cert_store.into())
                .build()
                .unwrap(),
            ignore_certificate_errors,
            override_manager,
        }
    }
}

impl rustls::client::danger::ServerCertVerifier for CertificateVerificationOverrideVerifier {
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.webpki_verifier
            .verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        self.webpki_verifier
            .verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.webpki_verifier.supported_verify_schemes()
    }

    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let error = match self.webpki_verifier.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        ) {
            Ok(result) => return Ok(result),
            Err(error) => error,
        };

        if self.ignore_certificate_errors {
            warn!("Ignoring certficate error: {error:?}");
            return Ok(rustls::client::danger::ServerCertVerified::assertion());
        }

        // If there's an override for this certificate, just accept it.
        for cert_with_exception in &*self.override_manager.0.lock().unwrap().overrides {
            if *end_entity == *cert_with_exception {
                return Ok(rustls::client::danger::ServerCertVerified::assertion());
            }
        }
        self.override_manager
            .0
            .lock()
            .unwrap()
            .certificates_failing_to_verify
            .insert(server_name.to_owned(), end_entity.clone().into_owned());
        Err(error)
    }
}

pub type BoxedBody = BoxBody<Bytes, hyper::Error>;

pub fn create_http_client(tls_config: TlsConfig) -> Client<Connector, BoxedBody> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls_config)
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .wrap_connector(ServoHttpConnector::new());

    Client::builder(TokioExecutor {})
        .http1_title_case_headers(true)
        .build(InstrumentedConnector::from(connector))
}
