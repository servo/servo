/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::client::Pool;
use hyper::net::{HttpStream, HttpsConnector, SslClient};
use hyper_openssl::OpensslClient;
use openssl::ssl::{SSL_OP_NO_COMPRESSION, SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3, SSL_VERIFY_PEER};
use openssl::ssl::{Ssl, SslConnectorBuilder, SslContext, SslMethod, SslStream};
use std::sync::Arc;
use util::resource_files::resources_dir_path;

pub type Connector = HttpsConnector<OpensslClient>;

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

pub fn create_http_connector() -> Arc<Pool<Connector>> {
    let ca_file = &resources_dir_path()
        .expect("Need certificate file to make network requests")
        .join("certs");
    let mut connector_builder = SslConnectorBuilder::new(SslMethod::tls()).unwrap();
    {
        let context_builder = connector_builder.builder_mut();
        context_builder.set_ca_file(ca_file).expect("could not set CA file");
        context_builder.set_cipher_list(DEFAULT_CIPHERS).expect("could not set ciphers");
        context_builder.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);
    }
    let connector = connector_builder.build();
    let client = OpensslClient::from(connector);
    let https_connector = HttpsConnector::new(client);
    Arc::new(Pool::with_connector(Default::default(), https_connector))
    /*
    let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
    context.set_CA_file(;
    context.set_cipher_list(DEFAULT_CIPHERS).unwrap();
    context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);
    let connector = HttpsConnector::new(ServoSslClient {
        context: Arc::new(context)
    });

    Arc::new(Pool::with_connector(Default::default(), connector))
    */
}

/*
#[derive(Clone)]
pub struct ServoSslClient {
    context: Arc<SslContext>,
}

impl SslClient for ServoSslClient {
    type Stream = SslStream<HttpStream>;

    fn wrap_client(&self, stream: HttpStream, host: &str) -> Result<Self::Stream, ::hyper::Error> {
        let mut ssl = try!(Ssl::new(&self.context));
        try!(ssl.set_hostname(host));
        let host = host.to_owned();
        SslStream::connect(ssl, stream).map_err(From::from)
    }
}
*/
