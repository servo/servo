// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A Chrome DevTools Protocol backend for Servo, building on
//! tokio-cdp's server implementation.
#![crate_name = "servo_cdp"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]
#![feature(conservative_impl_trait)]

extern crate bincode;
extern crate cdp;
extern crate cdp_traits;
extern crate futures;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
extern crate script_traits;
extern crate serde;
extern crate serde_json;
extern crate servo_config;
extern crate servo_url;
extern crate tokio_cdp;
extern crate tokio_core;
extern crate tokio_service;

mod ipc;

use cdp::http::{DevToolsUrls, Page, PageType, VersionInfo};
use cdp::http::{OwnedCommand as OwnedHttpCommand, Response as HttpResponse};
use cdp::ws::{Command as WsCommand, Response as WsResponse, ServerError};
use cdp::ws::page;
use cdp_traits::{CdpControlMsg, CdpControlReceiver, CdpControlSender};
use futures::{Future, Stream};
use futures::future::{self, Either, Loop};
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId};
use script_traits::{ConstellationMsg, LoadData};
use serde_json::Value;
use servo_url::ServoUrl;
use std::fmt::Display;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::mpsc::Sender;
use std::thread;
use tokio_cdp::server::HttpService;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_service::Service;

/// Spin up a Chrome DevTools Protocol server that listens for connections on
/// the specified port.
pub fn start_server(port: u16) -> CdpControlSender {
    let (sender, receiver) = cdp_traits::control_channel();
    {
        thread::Builder::new()
            .name("ChromeDevtools".to_owned())
            .spawn(move || run_server(receiver, port))
            .expect("Thread spawning failed");
    }
    sender
}

fn run_server(receiver: CdpControlReceiver, port: u16) {
    let mut core = Core::new().expect("Core creation failed");
    let handle = core.handle();

    let server_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port);
    let listener = TcpListener::bind(&server_addr, &handle).expect("TCP listener bind failed");
    debug!("CDP server listening locally on port {}.", port);

    let start_accepting = move |constellation_chan: Sender<ConstellationMsg>| {
        let accept = {
            let handle = handle.clone();
            listener
                .incoming()
                .for_each(move |(tcp, remote_addr)| {
                    debug!("CDP client connected from {}.", remote_addr);
                    let service = ServoHttpService {
                        constellation_chan: constellation_chan.clone(),
                        server_addr: server_addr,
                    };
                    tokio_cdp::server::bind_connection(&handle, tcp, remote_addr, service);
                    future::ok(())
                })
                .then(|result| Ok(result.expect("CDP listener error")))
        };
        handle.spawn(accept);
    };

    let receive = future::loop_fn((receiver, Some(start_accepting)), |(receiver, maybe_start)| {
        receiver.into_future().and_then(move |(message, receiver)| match message {
            Some(CdpControlMsg::StartAccepting(constellation_chan)) => match maybe_start {
                Some(start_accepting) => {
                    start_accepting(constellation_chan);
                    debug!("CDP server now accpeting clients.");
                    Ok(Loop::Continue((receiver, None)))
                }
                None => {
                    debug!("CDP server already accepting clients.");
                    Ok(Loop::Continue((receiver, None)))
                }
            },
            None => {
                debug!("CDP channel closed; shutting down.");
                Ok(Loop::Break(()))
            }
        })
    }).map_err(
        |_| io::Error::new(io::ErrorKind::BrokenPipe, "CDP channel closed abnormally"),
    );

    core.run(receive).expect("CDP run failed");
}

#[derive(Clone)]
struct ServoHttpService {
    constellation_chan: Sender<ConstellationMsg>,
    server_addr: SocketAddr,
}

impl ServoHttpService {
    fn version_info(&self) -> Box<Future<Item = HttpResponse, Error = ()>> {
        Box::new(future::ok(HttpResponse::VersionInfo(VersionInfo {
            browser: servo_config::servo_version(),
            protocol_version: cdp::STABLE_PROTOCOL_VERSION.into(),
            user_agent: servo_config::opts::get().user_agent.clone().into_owned(),
            v8_version: None,
            webkit_version: None,
        })))
    }

    fn page_list(&self) -> Box<Future<Item = HttpResponse, Error = ()>> {
        let constellation_chan = self.constellation_chan.clone();
        let server_addr = self.server_addr;

        Box::new(
            ipc::wrap_receive_ipc(ipc::focus_top_level_browsing_context_id(&constellation_chan))
                .and_then(move |maybe_top_level_browsing_context_id| {
                    let browsing_context_id: BrowsingContextId =
                        match maybe_top_level_browsing_context_id {
                            Some(top_level_browsing_context_id) => {
                                top_level_browsing_context_id.into()
                            }
                            None => return Either::A(future::ok(HttpResponse::PageList(vec![]))),
                        };
                    let get_title = ipc::wrap_receive_ipc(
                        ipc::get_page_title(&constellation_chan, browsing_context_id.clone()),
                    );
                    let get_url = ipc::wrap_receive_ipc(
                        ipc::get_page_url(&constellation_chan, browsing_context_id.clone()),
                    );
                    Either::B(get_title.join(get_url).and_then(move |(title, url)| {
                        let page_id = make_page_id(&browsing_context_id);
                        let devtools_urls = DevToolsUrls::new(&server_addr, &page_id);
                        let page = Page {
                            id: page_id,
                            ty: PageType::Tab,
                            url: url.into_string(),
                            title: title,
                            description: None,
                            favicon_url: None, // TODO
                            devtools_urls: Some(devtools_urls),
                        };
                        Ok(HttpResponse::PageList(vec![page]))
                    }))
                })
                .map_err(|err| {
                    debug!("CDP PageList command error: {}", err);
                }),
        )
    }

    fn new_page(
        &self,
        _maybe_url: Option<String>,
    ) -> Box<Future<Item = HttpResponse, Error = ()>> {
        Box::new(future::err(())) // Not supported
    }

    fn activate_page(&self, _page_id: String) -> Box<Future<Item = HttpResponse, Error = ()>> {
        Box::new(future::err(())) // Not supported
    }
}

impl Service for ServoHttpService {
    type Request = OwnedHttpCommand;
    type Response = HttpResponse;
    type Error = ();
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            OwnedHttpCommand::VersionInfo => self.version_info(),
            OwnedHttpCommand::PageList => self.page_list(),
            OwnedHttpCommand::NewPage(maybe_url) => self.new_page(maybe_url),
            OwnedHttpCommand::ActivatePage(page_id) => self.activate_page(page_id),
        }
    }
}

impl HttpService for ServoHttpService {
    type WsService = ServoWsService;
    type WsFuture = Box<Future<Item = Option<Self::WsService>, Error = ()>>;

    fn start_ws_session(&self, page_id: &str) -> Self::WsFuture {
        let page_id = page_id.to_owned();
        let constellation_chan = self.constellation_chan.clone();

        Box::new(
            ipc::wrap_receive_ipc(ipc::focus_top_level_browsing_context_id(&constellation_chan))
                .and_then(move |maybe_top_level_browsing_context_id| {
                    let top_level_browsing_context_id = match maybe_top_level_browsing_context_id {
                        Some(top_level_browsing_context_id) => top_level_browsing_context_id,
                        None => return Ok(None),
                    };

                    if page_id != make_page_id(top_level_browsing_context_id.as_ref()) {
                        return Ok(None);
                    }

                    Ok(Some(ServoWsService {
                        constellation_chan: constellation_chan,
                        top_level_browsing_context_id: top_level_browsing_context_id,
                    }))
                })
                .map_err(|err| {
                    debug!("CDP start_ws_session error: {}", err);
                }),
        )
    }
}

#[derive(Clone)]
struct ServoWsService {
    constellation_chan: Sender<ConstellationMsg>,
    top_level_browsing_context_id: TopLevelBrowsingContextId,
}

impl ServoWsService {
    fn page_navigate(
        &self,
        params: page::NavigateParams,
    ) -> Box<Future<Item = WsResponse, Error = ServerError>> {
        let page_id = make_page_id(self.top_level_browsing_context_id.as_ref());

        // The request should be rejected if the page URL is invalid.
        let url = match ServoUrl::parse(params.url.as_str()) {
            Ok(url) => url,
            Err(_) => {
                return Box::new(
                    future::err(ServerError::server_error("Cannot navigate to invalid URL", None)),
                )
            }
        };

        // However, if the referrer is an invalid URL, it is simply omitted.
        let referrer =
            params.referrer.and_then(|referrer| ServoUrl::parse(referrer.as_str()).ok());

        let load_data = LoadData::new(url, None, None, referrer);
        Box::new(future::result(
            ipc::load_url(&self.constellation_chan, self.top_level_browsing_context_id, load_data)
                .map(|_| WsResponse::PageNavigate(page::NavigateResponse { frame_id: page_id }))
                .map_err(internal_error),
        ))
    }
}

fn internal_error<T>(err: T) -> ServerError
where
    T: Display,
{
    ServerError::internal_error(Some(Value::String(err.to_string())))
}

impl Service for ServoWsService {
    type Request = WsCommand;
    type Response = WsResponse;
    type Error = ServerError;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            WsCommand::PageNavigate(params) => self.page_navigate(params),
            _ => Box::new(future::err(ServerError::method_not_found(req.name()))),
        }
    }
}

fn make_page_id(browsing_context_id: &BrowsingContextId) -> String {
    format!("{}-{}", browsing_context_id.namespace_id.0, browsing_context_id.index.0)
}
