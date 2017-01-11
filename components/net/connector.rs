/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper;
use hyper::client::Pool;
use hyper_openssl;
use openssl::ssl::{SslConnectorBuilder, SslMethod};
use hyper::net::HttpsConnector;
use hyper::net::{SslClient, SslServer, NetworkStream, HttpStream};
use openssl::x509::{X509StoreContextRef, X509, X509Ref};
use hyper;
use openssl;
use std::io::{Read, Write};
use hyper_openssl;
use openssl::ssl::{SSL_OP_NO_COMPRESSION, SSL_OP_NO_SSLV2, SSL_OP_NO_SSLV3, SSL_VERIFY_PEER};
use servo_config::resource_files::resources_dir_path;
use openssl::ssl::{Ssl, SslContext, SslContextBuilder, SslMethod};
use openssl::error::ErrorStack;
use std::sync::Arc;
use webpki::*;
use std::fmt::Debug;
use antidote::Mutex;
use hyper::Result;

use untrusted::Input;
use webpki::trust_anchor_util;
use rustls::internal::pemfile;
use rustls::RootCertStore;
use std::io::{BufReader};
use std::fs::File;
use time;
use openssl::hash::MessageDigest;
use rustls;

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

static ALL_SIGALGS: &'static [&'static SignatureAlgorithm] = &[
    &ECDSA_P256_SHA256,
    &ECDSA_P256_SHA384,
    &ECDSA_P384_SHA256,
    &ECDSA_P384_SHA384,
    &RSA_PKCS1_2048_8192_SHA1,
    &RSA_PKCS1_2048_8192_SHA256,
    &RSA_PKCS1_2048_8192_SHA384,
    &RSA_PKCS1_2048_8192_SHA512,
    &RSA_PKCS1_3072_8192_SHA384
];

pub fn create_http_connector(certificate_file: &str) -> Arc<Pool<Connector>> {
    let ca_file = &resources_dir_path()
        .expect("Need certificate file to make network requests")
        .join(certificate_file);

    let mut context = SslContextBuilder::new(SslMethod::tls()).unwrap();
    context.set_ca_file(ca_file);
    context.set_cipher_list(DEFAULT_CIPHERS).unwrap();
    context.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);

    //create the rustls root cert store
    let ca_pem = File::open(ca_file).unwrap();
    let mut ca_pem = BufReader::new(ca_pem);
    let mut root_store = RootCertStore::empty();
    let num_added = root_store.add_pem_file(&mut ca_pem).unwrap().0;

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
pub struct ServoSslClient{
    connector: Arc<ServoSslConnector>,
}

impl SslClient for ServoSslClient {
    type Stream = hyper_openssl::SslStream<HttpStream>;

    fn wrap_client(&self, stream: HttpStream, host: &str) -> Result<Self::Stream> {
        debug!("wrapping client");
        match self.connector.connect(host, stream){
            Ok(stream) => Ok(hyper_openssl::SslStream(Arc::new(Mutex::new(stream)))),
            Err(err) => Err(err)
        }
    }
}

#[derive(Clone)]
pub struct ServoSslConnector {
    context: Arc<SslContext>,
    roots: Arc<RootCertStore>,
}

impl ServoSslConnector {
    pub fn connect(&self, domain: &str, stream: HttpStream) -> Result<openssl::ssl::SslStream<HttpStream>> {
        let mut ssl = Ssl::new(&self.context).unwrap();
        ssl.set_hostname(domain).unwrap();
        let domain = domain.to_owned();
        let roots = self.roots.clone();

        ssl.set_verify_callback(SSL_VERIFY_PEER, move |p, x| {
            //::openssl_verify::verify_callback(&host, p, x)
            rustls_verify(&domain, &roots, p, x)
        });

        match ssl.connect(stream) {
            Ok(stream) => Ok(stream),
            Err(err) => Err(hyper::Error::Ssl(Box::new(err))),
        }
}

fn rustls_verify (domain: &str,
                roots: &RootCertStore,
                preverify_ok: bool,
                x509_ctx: &X509StoreContextRef) -> bool {

    // step 1: create presented certs
    // certs.0 must be end entity cert
    // presented certs is a vec<asn1cert>
    let mut presented_certs = vec!();
    match x509_ctx.current_cert() {
        Some(x509) => {
            let der = x509.to_der().unwrap();
            let cert = rustls::Certificate(der);
            presented_certs.push(cert);
        },
        None => (return false),
    };

    match x509_ctx.chain() {
        Some(mut chain) => {
            for cert in chain {
                presented_certs.push(rustls::Certificate(cert.to_der().unwrap()));
            }
        },
        None => (),
    };

    debug!("length of presented certs: {}", presented_certs.len());

    //step 3: verify certificate
    // not totally sure how to get the roots from the connector now. having a lifetime issue with the closure. figure that out later
    match rustls::verify_server_cert(&roots, &presented_certs, &domain) {
        Ok(_) => true,
        Err(error) => {debug!("Verification error: {:?}", error);
                      false },
    }

}


fn get_inter_vec(x509_ctx: &X509StoreContextRef) -> Vec<&X509Ref> {
    let mut inter_vec = vec!();
    match x509_ctx.chain() {
        //Some(chain) => vec!(),//.extend(chain),
        Some(chain) => inter_vec.extend(chain),
        None => (),
    };
    inter_vec
}