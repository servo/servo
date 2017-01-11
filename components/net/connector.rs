/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper;
use hyper::client::Pool;
use hyper_openssl;
use openssl::ssl::{SSL_OP_NO_COMPRESSION, SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3};
use openssl::ssl::{SslConnectorBuilder, SslMethod};
use servo_config::resource_files::resources_dir_path;
use std::sync::Arc;

pub type Connector = hyper::net::HttpsConnector<hyper_openssl::OpensslClient>;

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

pub fn create_http_connector(certificate_file: &str) -> Arc<Pool<Connector>> {
    let ca_file = &resources_dir_path()
        .expect("Need certificate file to make network requests")
        .join(certificate_file);

    let mut ssl_connector_builder = SslConnectorBuilder::new(SslMethod::tls()).unwrap();
    {
        let context = ssl_connector_builder.builder_mut();
        context.set_ca_file(ca_file).expect("could not set CA file");
        context.set_cipher_list(DEFAULT_CIPHERS).expect("could not set ciphers");
        context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);
    }
    let ssl_connector = ssl_connector_builder.build();
    let ssl_client = hyper_openssl::OpensslClient::from(ssl_connector);
    let https_connector = hyper::net::HttpsConnector::new(ssl_client);

    Arc::new(Pool::with_connector(Default::default(), https_connector))
}
