/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::{ControlMsg, CookieSource, LoadData, Metadata, LoadConsumer, IncludeSubdomains};
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::hosts::replace_hosts;
use devtools_traits::{ChromeToDevtoolsControlMsg, DevtoolsControlMsg, NetworkEvent};
use mime_classifier::MIMEClassifier;
use resource_task::{start_sending_opt, start_sending_sniffed_opt};
use hsts::{HSTSList, secure_url};

use ipc_channel::ipc::{self, IpcSender};
use log;
use std::collections::HashSet;
use file_loader;
use flate2::read::{DeflateDecoder, GzDecoder};
use hyper::client::Request;
use hyper::header::{AcceptEncoding, Accept, ContentLength, ContentType, Host, Location, qitem, Quality, QualityItem};
use hyper::header::StrictTransportSecurity;
use hyper::Error as HttpError;
use hyper::method::Method;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::net::{HttpConnector, HttpsConnector, Openssl};
use hyper::status::{StatusCode, StatusClass};
use std::error::Error;
use openssl::ssl::{SslContext, SslMethod, SSL_VERIFY_PEER};
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
use util::task::spawn_named;
use util::resource_files::resources_dir_path;
use util::opts;
use url::{Url, UrlParser};

use uuid;
use std::borrow::ToOwned;
use std::boxed::FnBox;

pub fn factory(resource_mgr_chan: IpcSender<ControlMsg>,
               devtools_chan: Option<Sender<DevtoolsControlMsg>>,
               hsts_list: Arc<Mutex<HSTSList>>)
               -> Box<FnBox(LoadData, LoadConsumer, Arc<MIMEClassifier>) + Send> {
    box move |load_data, senders, classifier| {
        spawn_named("http_loader".to_owned(),
                    move || load(load_data, senders, classifier, resource_mgr_chan, devtools_chan, hsts_list))
    }
}

fn send_error(url: Url, err: String, start_chan: LoadConsumer) {
    let mut metadata: Metadata = Metadata::default(url);
    metadata.status = None;

    match start_sending_opt(start_chan, metadata) {
        Ok(p) => p.send(Done(Err(err))).unwrap(),
        _ => {}
    };
}

enum ReadResult {
    Payload(Vec<u8>),
    EOF,
}

fn read_block<R: Read>(reader: &mut R) -> Result<ReadResult, ()> {
    let mut buf = vec![0; 1024];

    match reader.read(&mut buf) {
        Ok(len) if len > 0 => {
            unsafe { buf.set_len(len); }
            Ok(ReadResult::Payload(buf))
        }
        Ok(_) => Ok(ReadResult::EOF),
        Err(_) => Err(()),
    }
}

fn request_must_be_secured(hsts_list: &HSTSList, url: &Url) -> bool {
    match url.domain() {
        Some(ref h) => {
            hsts_list.is_host_secure(h)
        },
        _ => false
    }
}

fn load(mut load_data: LoadData,
        start_chan: LoadConsumer,
        classifier: Arc<MIMEClassifier>,
        resource_mgr_chan: IpcSender<ControlMsg>,
        devtools_chan: Option<Sender<DevtoolsControlMsg>>,
        hsts_list: Arc<Mutex<HSTSList>>) {
    // FIXME: At the time of writing this FIXME, servo didn't have any central
    //        location for configuration. If you're reading this and such a
    //        repository DOES exist, please update this constant to use it.
    let max_redirects = 50;
    let mut iters = 0;
    // URL of the document being loaded, as seen by all the higher-level code.
    let mut doc_url = load_data.url.clone();
    // URL that we actually fetch from the network, after applying the replacements
    // specified in the hosts file.
    let mut url = replace_hosts(&load_data.url);
    let mut redirected_to = HashSet::new();

    // If the URL is a view-source scheme then the scheme data contains the
    // real URL that should be used for which the source is to be viewed.
    // Change our existing URL to that and keep note that we are viewing
    // the source rather than rendering the contents of the URL.
    let viewing_source = url.scheme == "view-source";
    if viewing_source {
        let inner_url = load_data.url.non_relative_scheme_data().unwrap();
        doc_url = Url::parse(inner_url).unwrap();
        url = replace_hosts(&doc_url);
        match &*url.scheme {
            "http" | "https" => {}
            _ => {
                let s = format!("The {} scheme with view-source is not supported", url.scheme);
                send_error(url, s, start_chan);
                return;
            }
        };
    }

    // Loop to handle redirects.
    loop {
        iters = iters + 1;

        if &*url.scheme != "https" && request_must_be_secured(&hsts_list.lock().unwrap(), &url) {
            info!("{} is in the strict transport security list, requesting secure host", url);
            url = secure_url(&url);
        }

        if iters > max_redirects {
            send_error(url, "too many redirects".to_string(), start_chan);
            return;
        }

        match &*url.scheme {
            "http" | "https" => {}
            _ => {
                let s = format!("{} request, but we don't support that scheme", url.scheme);
                send_error(url, s, start_chan);
                return;
            }
        }

        info!("requesting {}", url.serialize());

        let ssl_err_string = "Some(OpenSslErrors([UnknownError { library: \"SSL routines\", \
function: \"SSL3_GET_SERVER_CERTIFICATE\", \
reason: \"certificate verify failed\" }]))";

        let req = if opts::get().nossl {
            Request::with_connector(load_data.method.clone(), url.clone(), &HttpConnector)
        } else {
            let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
            context.set_verify(SSL_VERIFY_PEER, None);
            context.set_CA_file(&resources_dir_path().join("certs")).unwrap();
            Request::with_connector(load_data.method.clone(), url.clone(),
                &HttpsConnector::new(Openssl { context: Arc::new(context) }))
        };

        let mut req = match req {
            Ok(req) => req,
            Err(HttpError::Io(ref io_error)) if (
                io_error.kind() == io::ErrorKind::Other &&
                io_error.description() == "Error in OpenSSL" &&
                // FIXME: This incredibly hacky. Make it more robust, and at least test it.
                format!("{:?}", io_error.cause()) == ssl_err_string
            ) => {
                let mut image = resources_dir_path();
                image.push("badcert.html");
                let load_data = LoadData::new(Url::from_file_path(&*image).unwrap(), None);
                file_loader::factory(load_data, start_chan, classifier);
                return;
            },
            Err(e) => {
                println!("{:?}", e);
                send_error(url, e.description().to_string(), start_chan);
                return;
            }
        };

        //Ensure that the host header is set from the original url
        let host = Host {
            hostname: doc_url.serialize_host().unwrap(),
            port: doc_url.port_or_default()
        };

        // Avoid automatically preserving request headers when redirects occur.
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=401564 and
        // https://bugzilla.mozilla.org/show_bug.cgi?id=216828 .
        // Only preserve ones which have been explicitly marked as such.
        if iters == 1 {
            let mut combined_headers = load_data.headers.clone();
            combined_headers.extend(load_data.preserved_headers.iter());
            *req.headers_mut() = combined_headers;
        } else {
            *req.headers_mut() = load_data.preserved_headers.clone();
        }

        req.headers_mut().set(host);

        if !req.headers().has::<Accept>() {
            let accept = Accept(vec![
                qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
                qitem(Mime(TopLevel::Application, SubLevel::Ext("xhtml+xml".to_string()), vec![])),
                QualityItem::new(Mime(TopLevel::Application, SubLevel::Xml, vec![]), Quality(900u16)),
                QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(800u16)),
            ]);
            req.headers_mut().set(accept);
        }

        let (tx, rx) = ipc::channel().unwrap();
        resource_mgr_chan.send(ControlMsg::GetCookiesForUrl(doc_url.clone(),
                                                            tx,
                                                            CookieSource::HTTP)).unwrap();
        if let Some(cookie_list) = rx.recv().unwrap() {
            let mut v = Vec::new();
            v.push(cookie_list.into_bytes());
            req.headers_mut().set_raw("Cookie".to_owned(), v);
        }

        if !req.headers().has::<AcceptEncoding>() {
            req.headers_mut().set_raw("Accept-Encoding".to_owned(), vec![b"gzip, deflate".to_vec()]);
        }
        if log_enabled!(log::LogLevel::Info) {
            info!("{}", load_data.method);
            for header in req.headers().iter() {
                info!(" - {}", header);
            }
            info!("{:?}", load_data.data);
        }

        // Avoid automatically sending request body if a redirect has occurred.
        let writer = match load_data.data {
            Some(ref data) if iters == 1 => {
                req.headers_mut().set(ContentLength(data.len() as u64));
                let mut writer = match req.start() {
                    Ok(w) => w,
                    Err(e) => {
                        send_error(url, e.description().to_string(), start_chan);
                        return;
                    }
                };
                match writer.write_all(&*data) {
                    Err(e) => {
                        send_error(url, e.description().to_string(), start_chan);
                        return;
                    }
                    _ => {}
                };
                writer
            },
            _ => {
                match load_data.method {
                    Method::Get | Method::Head => (),
                    _ => req.headers_mut().set(ContentLength(0))
                }
                match req.start() {
                    Ok(w) => w,
                    Err(e) => {
                        send_error(url, e.description().to_string(), start_chan);
                        return;
                    }
                }
            }
        };

        // Send an HttpRequest message to devtools with a unique request_id
        // TODO: Do this only if load_data has some pipeline_id, and send the pipeline_id in the message
        let request_id = uuid::Uuid::new_v4().to_simple_string();
        if let Some(ref chan) = devtools_chan {
            let net_event = NetworkEvent::HttpRequest(load_data.url.clone(),
                                                      load_data.method.clone(),
                                                      load_data.headers.clone(),
                                                      load_data.data.clone());
            chan.send(DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::NetworkEvent(request_id.clone(),
                                                             net_event))).unwrap();
        }

        let mut response = match writer.send() {
            Ok(r) => r,
            Err(e) => {
                send_error(url, e.description().to_string(), start_chan);
                return;
            }
        };

        // Dump headers, but only do the iteration if info!() is enabled.
        info!("got HTTP response {}, headers:", response.status);
        if log_enabled!(log::LogLevel::Info) {
            for header in response.headers.iter() {
                info!(" - {}", header);
            }
        }

        if let Some(cookies) = response.headers.get_raw("set-cookie") {
            for cookie in cookies {
                if let Ok(cookies) = String::from_utf8(cookie.clone()) {
                    resource_mgr_chan.send(ControlMsg::SetCookiesForUrl(doc_url.clone(),
                                                                        cookies,
                                                                        CookieSource::HTTP)).unwrap();
                }
            }
        }

        if url.scheme == "https" {
            if let Some(header) = response.headers.get::<StrictTransportSecurity>() {
                if let Some(host) = url.domain() {
                    info!("adding host {} to the strict transport security list", host);
                    info!("- max-age {}", header.max_age);

                    let include_subdomains = if header.include_subdomains {
                        info!("- includeSubdomains");
                        IncludeSubdomains::Included
                    } else {
                        IncludeSubdomains::NotIncluded
                    };

                    resource_mgr_chan.send(
                        ControlMsg::SetHSTSEntryForHost(
                            host.to_string(), include_subdomains, header.max_age
                        )
                    ).unwrap();
                }
            }
        }


        if response.status.class() == StatusClass::Redirection {
            match response.headers.get::<Location>() {
                Some(&Location(ref new_url)) => {
                    // CORS (https://fetch.spec.whatwg.org/#http-fetch, status section, point 9, 10)
                    match load_data.cors {
                        Some(ref c) => {
                            if c.preflight {
                                // The preflight lied
                                send_error(url,
                                           "Preflight fetch inconsistent with main fetch".to_string(),
                                           start_chan);
                                return;
                            } else {
                                // XXXManishearth There are some CORS-related steps here,
                                // but they don't seem necessary until credentials are implemented
                            }
                        }
                        _ => {}
                    }
                    let new_doc_url = match UrlParser::new().base_url(&doc_url).parse(&new_url) {
                        Ok(u) => u,
                        Err(e) => {
                            send_error(doc_url, e.to_string(), start_chan);
                            return;
                        }
                    };
                    info!("redirecting to {}", new_doc_url);
                    url = replace_hosts(&new_doc_url);
                    doc_url = new_doc_url;

                    // According to https://tools.ietf.org/html/rfc7231#section-6.4.2,
                    // historically UAs have rewritten POST->GET on 301 and 302 responses.
                    if load_data.method == Method::Post &&
                        (response.status == StatusCode::MovedPermanently ||
                         response.status == StatusCode::Found) {
                        load_data.method = Method::Get;
                    }

                    if redirected_to.contains(&doc_url) {
                        send_error(doc_url, "redirect loop".to_string(), start_chan);
                        return;
                    }

                    redirected_to.insert(doc_url.clone());
                    continue;
                }
                None => ()
            }
        }

        let mut adjusted_headers = response.headers.clone();
        if viewing_source {
            adjusted_headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])));
        }
        let mut metadata: Metadata = Metadata::default(doc_url);
        metadata.set_content_type(match adjusted_headers.get() {
            Some(&ContentType(ref mime)) => Some(mime),
            None => None
        });
        metadata.headers = Some(adjusted_headers);
        metadata.status = Some(response.status_raw().clone());

        let mut encoding_str: Option<String> = None;
        //FIXME: Implement Content-Encoding Header https://github.com/hyperium/hyper/issues/391
        if let Some(encodings) = response.headers.get_raw("content-encoding") {
            for encoding in encodings {
                if let Ok(encodings) = String::from_utf8(encoding.clone()) {
                    if encodings == "gzip" || encodings == "deflate" {
                        encoding_str = Some(encodings);
                        break;
                    }
                }
            }
        }

        // Send an HttpResponse message to devtools with the corresponding request_id
        // TODO: Send this message only if load_data has a pipeline_id that is not None
        if let Some(ref chan) = devtools_chan {
            let net_event_response =
                NetworkEvent::HttpResponse(metadata.headers.clone(),
                                           metadata.status.clone(),
                                           None);
            chan.send(DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::NetworkEvent(request_id,
                                                             net_event_response))).unwrap();
        }

        match encoding_str {
            Some(encoding) => {
                if encoding == "gzip" {
                    let result = GzDecoder::new(response);
                    match result {
                        Ok(mut response_decoding) => {
                            send_data(&mut response_decoding, start_chan, metadata, classifier);
                        }
                        Err(err) => {
                            send_error(metadata.final_url, err.to_string(), start_chan);
                            return;
                        }
                    }
                } else if encoding == "deflate" {
                    let mut response_decoding = DeflateDecoder::new(response);
                    send_data(&mut response_decoding, start_chan, metadata, classifier);
                }
            },
            None => {
                send_data(&mut response, start_chan, metadata, classifier);
            }
        }

        // We didn't get redirected.
        break;
    }
}

fn send_data<R: Read>(reader: &mut R,
                      start_chan: LoadConsumer,
                      metadata: Metadata,
                      classifier: Arc<MIMEClassifier>) {
    let (progress_chan, mut chunk) = {
        let buf = match read_block(reader) {
            Ok(ReadResult::Payload(buf)) => buf,
            _ => vec!(),
        };
        let p = match start_sending_sniffed_opt(start_chan, metadata, classifier, &buf) {
            Ok(p) => p,
            _ => return
        };
        (p, buf)
    };

    loop {
        if progress_chan.send(Payload(chunk)).is_err() {
            // The send errors when the receiver is out of scope,
            // which will happen if the fetch has timed out (or has been aborted)
            // so we don't need to continue with the loading of the file here.
            return;
        }

        chunk = match read_block(reader) {
            Ok(ReadResult::Payload(buf)) => buf,
            Ok(ReadResult::EOF) | Err(_) => break,
        };
    }

    let _ = progress_chan.send(Done(Ok(())));
}
