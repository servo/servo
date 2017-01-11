/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use antidote::Mutex;
use hyper;
use hyper::Result;
use hyper::client::Pool;
use hyper::net::{HttpStream, HttpsConnector, SslClient};
use hyper_openssl;
use openssl;
use openssl::ssl::{SSL_OP_NO_COMPRESSION, SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3, SSL_VERIFY_PEER};
use openssl::ssl::{Ssl, SslContext, SslContextBuilder, SslMethod};
use openssl::x509::X509StoreContextRef;
use rustls;
use rustls::RootCertStore;
use servo_config::resource_files::resources_dir_path;       //FIXME are we using this or the cert file arg
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

pub type Connector = HttpsConnector<ServoSslClient>;

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
    let mut context = SslContextBuilder::new(SslMethod::tls()).unwrap();
    context.set_ca_file(certificate_file);
    context.set_cipher_list(DEFAULT_CIPHERS).unwrap();
    context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);

    //create the rustls root cert store
    let ca_pem = File::open(certificate_file).unwrap();
    let mut ca_pem = BufReader::new(ca_pem);
    let mut root_store = RootCertStore::empty();
    root_store.add_pem_file(&mut ca_pem).unwrap().0;

    let servo_connector = ServoSslConnector {
        context: Arc::new(context.build()),
        roots: Arc::new(root_store),
    };

    let connector = HttpsConnector::new(ServoSslClient {
        connector: Arc::new(servo_connector),
    });

    Arc::new(Pool::with_connector(Default::default(), connector))
}

#[derive(Clone)]
pub struct ServoSslClient {
    connector: Arc<ServoSslConnector>,
}

impl SslClient for ServoSslClient {
    type Stream = hyper_openssl::SslStream<HttpStream>;

    fn wrap_client(&self, stream: HttpStream, host: &str) -> Result<Self::Stream> {
        match self.connector.connect(host, stream) {
            Ok(stream) => Ok(hyper_openssl::SslStream(Arc::new(Mutex::new(stream)))),
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone)]
pub struct ServoSslConnector {
    context: Arc<SslContext>,
    roots: Arc<RootCertStore>,
}

impl ServoSslConnector {
    pub fn connect(&self, domain: &str, stream: HttpStream) -> Result<openssl::ssl::SslStream<HttpStream>>
    {
        let mut ssl = Ssl::new(&self.context).unwrap();
        ssl.set_hostname(domain).unwrap();
        let domain = domain.to_owned();
        let roots = self.roots.clone();

        ssl.set_verify_callback(SSL_VERIFY_PEER, move |p, x| {
            rustls_verify(&domain, &roots, p, x)
        });


        match ssl.connect(stream) {
            Ok(stream) => Ok(stream),
            Err(err) => Err(hyper::Error::Ssl(Box::new(err))),
        }
    }
}

fn rustls_verify(domain: &str,
                roots: &RootCertStore,
                preverify_ok: bool,
                x509_ctx: &X509StoreContextRef) -> bool {
    if !preverify_ok || x509_ctx.error_depth() != 0 {
        return preverify_ok;
    }

    // create presented certs
    let mut presented_certs = vec!();
    match x509_ctx.chain() {
        Some(chain) => {
            for cert in chain {
                presented_certs.push(rustls::Certificate(cert.to_der().unwrap()));
            }
        },
        None => (),
    };

    // verify certificate
    match rustls::parallel_verify_server_cert(&roots, &presented_certs, &domain) {
        Ok(_) => true,
        Err(error) => { error!("Verification error: {:?}", error);
                      false },
    }

    //TODO when to use serial
    /*match rustls::verify_server_cert(&roots, &presented_certs, &domain) {
        Ok(_) => true,
        Err(error) => { error!("Verification error: {:?}", error);
                      false },
    }*/
}

