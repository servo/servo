/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::marker::PhantomData;
use std::net::{SocketAddr, TcpListener as StdTcpListener};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use bytes::Bytes;
use http::{Method, StatusCode};
use log::{debug, error, trace, warn};
use tokio::net::TcpListener;
use url::{Host, Url};
use warp::{Buf, Filter, Rejection};
use webdriver::command::{WebDriverCommand, WebDriverMessage};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::httpapi::{
    Route, VoidWebDriverExtensionRoute, WebDriverExtensionRoute, standard_routes,
};
use webdriver::response::{CloseWindowResponse, WebDriverResponse};

use crate::Parameters;

// Silence warning about Quit being unused for now.
#[allow(dead_code)]
enum DispatchMessage<U: WebDriverExtensionRoute> {
    HandleWebDriver(
        WebDriverMessage<U>,
        Sender<WebDriverResult<WebDriverResponse>>,
    ),
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
/// Representation of whether we managed to successfully send a DeleteSession message
/// and read the response during session teardown.
pub enum SessionTeardownKind {
    /// A DeleteSession message has been sent and the response handled.
    Deleted,
    /// No DeleteSession message has been sent, or the response was not received.
    NotDeleted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
}

impl Session {
    fn new(id: String) -> Session {
        Session { id }
    }
}

pub trait WebDriverHandler<U: WebDriverExtensionRoute = VoidWebDriverExtensionRoute>: Send {
    fn handle_command(
        &mut self,
        session: &Option<Session>,
        msg: WebDriverMessage<U>,
    ) -> WebDriverResult<WebDriverResponse>;
    fn teardown_session(&mut self, kind: SessionTeardownKind);
}

#[derive(Debug)]
struct Dispatcher<T: WebDriverHandler<U>, U: WebDriverExtensionRoute> {
    handler: T,
    session: Option<Session>,
    extension_type: PhantomData<U>,
}

impl<T: WebDriverHandler<U>, U: WebDriverExtensionRoute> Dispatcher<T, U> {
    fn new(handler: T) -> Dispatcher<T, U> {
        Dispatcher {
            handler,
            session: None,
            extension_type: PhantomData,
        }
    }

    fn run(&mut self, msg_chan: &Receiver<DispatchMessage<U>>) {
        loop {
            match msg_chan.recv() {
                Ok(DispatchMessage::HandleWebDriver(msg, resp_chan)) => {
                    let resp = match self.check_session(&msg) {
                        Ok(_) => self.handler.handle_command(&self.session, msg),
                        Err(e) => Err(e),
                    };

                    match resp {
                        Ok(WebDriverResponse::NewSession(ref new_session)) => {
                            self.session = Some(Session::new(new_session.session_id.clone()));
                        },
                        Ok(WebDriverResponse::CloseWindow(CloseWindowResponse(ref handles))) => {
                            if handles.is_empty() {
                                debug!("Last window was closed, deleting session");
                                // The teardown_session implementation is responsible for actually
                                // sending the DeleteSession message in this case
                                self.teardown_session(SessionTeardownKind::NotDeleted);
                            }
                        },
                        Ok(WebDriverResponse::DeleteSession) => {
                            self.teardown_session(SessionTeardownKind::Deleted);
                        },
                        Err(ref x) if x.delete_session => {
                            // This includes the case where we failed during session creation
                            self.teardown_session(SessionTeardownKind::NotDeleted)
                        },
                        _ => {},
                    }

                    if resp_chan.send(resp).is_err() {
                        error!("Sending response to the main thread failed");
                    };
                },
                Ok(DispatchMessage::Quit) => break,
                Err(e) => panic!("Error receiving message in handler: {:?}", e),
            }
        }
    }

    fn teardown_session(&mut self, kind: SessionTeardownKind) {
        debug!("Teardown session");
        let final_kind = match kind {
            SessionTeardownKind::NotDeleted if self.session.is_some() => {
                let delete_session = WebDriverMessage {
                    session_id: Some(
                        self.session
                            .as_ref()
                            .expect("Failed to get session")
                            .id
                            .clone(),
                    ),
                    command: WebDriverCommand::DeleteSession,
                };
                match self.handler.handle_command(&self.session, delete_session) {
                    Ok(_) => SessionTeardownKind::Deleted,
                    Err(_) => SessionTeardownKind::NotDeleted,
                }
            },
            _ => kind,
        };
        self.handler.teardown_session(final_kind);
        self.session = None;
    }

    fn check_session(&self, msg: &WebDriverMessage<U>) -> WebDriverResult<()> {
        match msg.session_id {
            Some(ref msg_session_id) => match self.session {
                Some(ref existing_session) => {
                    if existing_session.id != *msg_session_id {
                        Err(WebDriverError::new(
                            ErrorStatus::InvalidSessionId,
                            format!("Got unexpected session id {}", msg_session_id),
                        ))
                    } else {
                        Ok(())
                    }
                },
                None => Ok(()),
            },
            None => {
                match self.session {
                    Some(_) => {
                        match msg.command {
                            WebDriverCommand::Status => Ok(()),
                            WebDriverCommand::NewSession(_) => Err(WebDriverError::new(
                                ErrorStatus::SessionNotCreated,
                                "Session is already started",
                            )),
                            _ => {
                                // This should be impossible
                                error!("Got a message with no session id");
                                Err(WebDriverError::new(
                                    ErrorStatus::UnknownError,
                                    "Got a command with no session?!",
                                ))
                            },
                        }
                    },
                    None => match msg.command {
                        WebDriverCommand::NewSession(_) => Ok(()),
                        WebDriverCommand::Status => Ok(()),
                        _ => Err(WebDriverError::new(
                            ErrorStatus::InvalidSessionId,
                            "Tried to run a command before creating a session",
                        )),
                    },
                }
            },
        }
    }
}

pub struct Listener {
    guard: Option<thread::JoinHandle<()>>,
    pub socket: SocketAddr,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.guard.take().map(|j| j.join());
    }
}

pub fn start<T, U>(
    mut address: SocketAddr,
    allow_hosts: Vec<Host>,
    allow_origins: Vec<Url>,
    handler: T,
    extension_routes: Vec<(Method, &'static str, U)>,
) -> ::std::io::Result<Listener>
where
    T: 'static + WebDriverHandler<U>,
    U: 'static + WebDriverExtensionRoute + Send + Sync,
{
    let listener = StdTcpListener::bind(address)?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    if address.port() == 0 {
        // If we passed in 0 as the port number the OS will assign an unused port;
        // we want to update the address to the actual used port
        address.set_port(addr.port())
    }
    let (msg_send, msg_recv) = channel();

    let builder = thread::Builder::new().name("webdriver server".to_string());
    let handle = builder.spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        let listener = rt.block_on(async { TcpListener::from_std(listener).unwrap() });
        let wroutes = build_warp_routes(
            address,
            allow_hosts,
            allow_origins,
            &extension_routes,
            msg_send.clone(),
        );
        let fut = warp::serve(wroutes).incoming(listener).run();
        rt.block_on(fut);
    })?;

    let builder = thread::Builder::new().name("webdriver dispatcher".to_string());
    builder.spawn(move || {
        let mut dispatcher = Dispatcher::new(handler);
        dispatcher.run(&msg_recv);
    })?;

    Ok(Listener {
        guard: Some(handle),
        socket: addr,
    })
}

fn build_warp_routes<U: 'static + WebDriverExtensionRoute + Send + Sync>(
    address: SocketAddr,
    allow_hosts: Vec<Host>,
    allow_origins: Vec<Url>,
    ext_routes: &[(Method, &'static str, U)],
    chan: Sender<DispatchMessage<U>>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone + 'static {
    let chan = Arc::new(Mutex::new(chan));
    let mut std_routes = standard_routes::<U>();

    let (method, path, res) = std_routes.pop().unwrap();
    trace!("Build standard route for {path}");
    let mut wroutes = build_route(
        address,
        allow_hosts.clone(),
        allow_origins.clone(),
        convert_method(method),
        path,
        res,
        chan.clone(),
    );

    for (method, path, res) in std_routes {
        trace!("Build standard route for {path}");
        wroutes = wroutes
            .or(build_route(
                address,
                allow_hosts.clone(),
                allow_origins.clone(),
                convert_method(method),
                path,
                res.clone(),
                chan.clone(),
            ))
            .unify()
            .boxed()
    }

    for (method, path, res) in ext_routes {
        trace!("Build vendor route for {path}");
        wroutes = wroutes
            .or(build_route(
                address,
                allow_hosts.clone(),
                allow_origins.clone(),
                method.clone(),
                path,
                Route::Extension(res.clone()),
                chan.clone(),
            ))
            .unify()
            .boxed()
    }

    wroutes
}

fn is_host_allowed(server_address: &SocketAddr, allow_hosts: &[Host], host_header: &str) -> bool {
    // Validate that the Host header value has a hostname in allow_hosts and
    // the port matches the server configuration
    let header_host_url = match Url::parse(&format!("http://{}", &host_header)) {
        Ok(x) => x,
        Err(_) => {
            return false;
        },
    };

    let host = match header_host_url.host() {
        Some(host) => host.to_owned(),
        None => {
            // This shouldn't be possible since http URL always have a
            // host, but conservatively return false here, which will cause
            // an error response
            return false;
        },
    };
    let port = match header_host_url.port_or_known_default() {
        Some(port) => port,
        None => {
            // This shouldn't be possible since http URL always have a
            // default port, but conservatively return false here, which will cause
            // an error response
            return false;
        },
    };

    let host_matches = match host {
        Host::Domain(_) => allow_hosts.contains(&host),
        Host::Ipv4(_) | Host::Ipv6(_) => true,
    };
    let port_matches = server_address.port() == port;
    host_matches && port_matches
}

fn is_origin_allowed(allow_origins: &[Url], origin_url: Url) -> bool {
    // Validate that the Origin header value is in allow_origins
    allow_origins.contains(&origin_url)
}

fn build_route<U: 'static + WebDriverExtensionRoute + Send + Sync>(
    server_address: SocketAddr,
    allow_hosts: Vec<Host>,
    allow_origins: Vec<Url>,
    method: Method,
    path: &'static str,
    route: Route<U>,
    chan: Arc<Mutex<Sender<DispatchMessage<U>>>>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    // Create an empty filter based on the provided method and append an empty hashmap to it. The
    // hashmap will be used to store path parameters.
    let mut subroute = match method {
        Method::GET => warp::get().boxed(),
        Method::POST => warp::post().boxed(),
        Method::DELETE => warp::delete().boxed(),
        Method::OPTIONS => warp::options().boxed(),
        Method::PUT => warp::put().boxed(),
        _ => panic!("Unsupported method"),
    }
    .or(warp::head())
    .unify()
    .map(Parameters::new)
    .boxed();

    // For each part of the path, if it's a normal part, just append it to the current filter,
    // otherwise if it's a parameter (a named enclosed in { }), we take that parameter and insert
    // it into the hashmap created earlier.
    for part in path.split('/') {
        if part.is_empty() {
            continue;
        } else if part.starts_with('{') {
            assert!(part.ends_with('}'));

            subroute = subroute
                .and(warp::path::param())
                .map(move |mut params: Parameters, param: String| {
                    let name = &part[1..part.len() - 1];
                    params.insert(name.to_string(), param);
                    params
                })
                .boxed();
        } else {
            subroute = subroute.and(warp::path(part)).boxed();
        }
    }

    // Finally, tell warp that the path is complete
    subroute
        .and(warp::path::end())
        .and(warp::path::full())
        .and(warp::method())
        .and(warp::header::optional::<String>("origin"))
        .and(warp::header::optional::<String>("host"))
        .and(warp::header::optional::<String>("content-type"))
        .and(warp::body::bytes())
        .map(
            move |params,
                  full_path: warp::path::FullPath,
                  method,
                  origin_header: Option<String>,
                  host_header: Option<String>,
                  content_type_header: Option<String>,
                  body: Bytes| {
                if method == Method::HEAD {
                    return warp::reply::with_status("".into(), StatusCode::OK);
                }
                if let Some(host) = host_header {
                    if !is_host_allowed(&server_address, &allow_hosts, &host) {
                        warn!(
                            "Rejected request with Host header {}, allowed values are [{}]",
                            host,
                            allow_hosts
                                .iter()
                                .map(|x| format!("{}:{}", x, server_address.port()))
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                        let err = WebDriverError::new(
                            ErrorStatus::UnknownError,
                            format!("Invalid Host header {}", host),
                        );
                        return warp::reply::with_status(
                            serde_json::to_string(&err).unwrap(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        );
                    };
                } else {
                    warn!("Rejected request with missing Host header");
                    let err = WebDriverError::new(
                        ErrorStatus::UnknownError,
                        "Missing Host header".to_string(),
                    );
                    return warp::reply::with_status(
                        serde_json::to_string(&err).unwrap(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    );
                }
                if let Some(origin) = origin_header {
                    let make_err = || {
                        warn!(
                            "Rejected request with Origin header {}, allowed values are [{}]",
                            origin,
                            allow_origins
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                        WebDriverError::new(
                            ErrorStatus::UnknownError,
                            format!("Invalid Origin header {}", origin),
                        )
                    };
                    let origin_url = match Url::parse(&origin) {
                        Ok(url) => url,
                        Err(_) => {
                            return warp::reply::with_status(
                                serde_json::to_string(&make_err()).unwrap(),
                                StatusCode::INTERNAL_SERVER_ERROR,
                            );
                        },
                    };
                    if !is_origin_allowed(&allow_origins, origin_url) {
                        return warp::reply::with_status(
                            serde_json::to_string(&make_err()).unwrap(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        );
                    }
                }
                if method == Method::POST {
                    // Disallow CORS-safelisted request headers
                    // c.f. https://fetch.spec.whatwg.org/#cors-safelisted-request-header
                    let content_type = content_type_header
                        .as_ref()
                        .map(|x| x.find(';').and_then(|idx| x.get(0..idx)).unwrap_or(x))
                        .map(|x| x.trim())
                        .map(|x| x.to_lowercase());
                    match content_type.as_ref().map(|x| x.as_ref()) {
                        Some("application/x-www-form-urlencoded") |
                        Some("multipart/form-data") |
                        Some("text/plain") => {
                            warn!(
                                "Rejected POST request with disallowed content type {}",
                                content_type.unwrap_or_else(|| "".into())
                            );
                            let err = WebDriverError::new(
                                ErrorStatus::UnknownError,
                                "Invalid Content-Type",
                            );
                            return warp::reply::with_status(
                                serde_json::to_string(&err).unwrap(),
                                StatusCode::INTERNAL_SERVER_ERROR,
                            );
                        },
                        Some(_) | None => {},
                    }
                }
                let body = String::from_utf8(body.chunk().to_vec());
                if body.is_err() {
                    let err = WebDriverError::new(
                        ErrorStatus::UnknownError,
                        "Request body wasn't valid UTF-8",
                    );
                    return warp::reply::with_status(
                        serde_json::to_string(&err).unwrap(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    );
                }
                let body = body.unwrap();

                debug!("-> {} {} {}", method, full_path.as_str(), body);
                let msg_result = WebDriverMessage::from_http(
                    route.clone(),
                    &params,
                    &body,
                    method == Method::POST,
                );

                let (status, resp_body) = match msg_result {
                    Ok(message) => {
                        let (send_res, recv_res) = channel();
                        match chan.lock() {
                            Ok(ref c) => {
                                let res =
                                    c.send(DispatchMessage::HandleWebDriver(message, send_res));
                                match res {
                                    Ok(x) => x,
                                    Err(e) => panic!("Error: {:?}", e),
                                }
                            },
                            Err(e) => panic!("Error reading response: {:?}", e),
                        }

                        match recv_res.recv() {
                            Ok(data) => match data {
                                Ok(response) => {
                                    (StatusCode::OK, serde_json::to_string(&response).unwrap())
                                },
                                Err(e) => (
                                    StatusCode::from_u16(e.http_status().as_u16()).unwrap(),
                                    serde_json::to_string(&e).unwrap(),
                                ),
                            },
                            Err(e) => panic!("Error reading response: {:?}", e),
                        }
                    },
                    Err(e) => (
                        convert_status(e.http_status()),
                        serde_json::to_string(&e).unwrap(),
                    ),
                };

                debug!("<- {} {}", status, resp_body);
                warp::reply::with_status(resp_body, status)
            },
        )
        .with(warp::reply::with::header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        ))
        .with(warp::reply::with::header(
            http::header::CACHE_CONTROL,
            "no-cache",
        ))
        .boxed()
}

/// Convert from http 0.2 StatusCode to http 1.0 StatusCode
fn convert_status(status: http02::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap()
}

/// Convert from http 0.2 Method to http 1.0 Method
fn convert_method(method: http02::Method) -> Method {
    match method {
        http02::Method::OPTIONS => http::Method::OPTIONS,
        http02::Method::GET => http::Method::GET,
        http02::Method::POST => http::Method::POST,
        http02::Method::PUT => http::Method::PUT,
        http02::Method::DELETE => http::Method::DELETE,
        http02::Method::HEAD => http::Method::HEAD,
        http02::Method::TRACE => http::Method::TRACE,
        http02::Method::CONNECT => http::Method::CONNECT,
        http02::Method::PATCH => http::Method::PATCH,
        _ => http::Method::from_bytes(method.as_str().as_bytes()).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_host_allowed() {
        let addr_80 = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 80);
        let addr_8000 = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 8000);
        let addr_v6_80 = SocketAddr::new(IpAddr::from_str("::1").unwrap(), 80);
        let addr_v6_8000 = SocketAddr::new(IpAddr::from_str("::1").unwrap(), 8000);

        // We match the host ip address to the server, so we can only use hosts that actually resolve
        let localhost_host = Host::Domain("localhost".to_string());
        let test_host = Host::Domain("example.test".to_string());
        let subdomain_localhost_host = Host::Domain("subdomain.localhost".to_string());

        assert!(is_host_allowed(
            &addr_80,
            &[localhost_host.clone()],
            "localhost:80"
        ));
        assert!(is_host_allowed(
            &addr_80,
            &[test_host.clone()],
            "example.test:80"
        ));
        assert!(is_host_allowed(
            &addr_80,
            &[test_host.clone(), localhost_host.clone()],
            "example.test"
        ));
        assert!(is_host_allowed(
            &addr_80,
            &[subdomain_localhost_host.clone()],
            "subdomain.localhost"
        ));

        // ip address cases
        assert!(is_host_allowed(&addr_80, &[], "127.0.0.1:80"));
        assert!(is_host_allowed(&addr_v6_80, &[], "127.0.0.1"));
        assert!(is_host_allowed(&addr_80, &[], "[::1]"));
        assert!(is_host_allowed(&addr_8000, &[], "127.0.0.1:8000"));
        assert!(is_host_allowed(
            &addr_80,
            &[subdomain_localhost_host.clone()],
            "[::1]"
        ));
        assert!(is_host_allowed(
            &addr_v6_8000,
            &[subdomain_localhost_host.clone()],
            "[::1]:8000"
        ));

        // Mismatch cases

        assert!(!is_host_allowed(&addr_80, &[test_host], "localhost"));

        assert!(!is_host_allowed(&addr_80, &[], "localhost:80"));

        // Port mismatch cases

        assert!(!is_host_allowed(
            &addr_80,
            &[localhost_host.clone()],
            "localhost:8000"
        ));
        assert!(!is_host_allowed(
            &addr_8000,
            &[localhost_host.clone()],
            "localhost"
        ));
        assert!(!is_host_allowed(
            &addr_v6_8000,
            &[localhost_host.clone()],
            "[::1]"
        ));
    }

    #[test]
    fn test_origin_allowed() {
        assert!(is_origin_allowed(
            &[Url::parse("http://localhost").unwrap()],
            Url::parse("http://localhost").unwrap()
        ));
        assert!(is_origin_allowed(
            &[Url::parse("http://localhost").unwrap()],
            Url::parse("http://localhost:80").unwrap()
        ));
        assert!(is_origin_allowed(
            &[
                Url::parse("https://test.example").unwrap(),
                Url::parse("http://localhost").unwrap()
            ],
            Url::parse("http://localhost").unwrap()
        ));
        assert!(is_origin_allowed(
            &[
                Url::parse("https://test.example").unwrap(),
                Url::parse("http://localhost").unwrap()
            ],
            Url::parse("https://test.example:443").unwrap()
        ));
        // Mismatch cases
        assert!(!is_origin_allowed(
            &[],
            Url::parse("http://localhost").unwrap()
        ));
        assert!(!is_origin_allowed(
            &[Url::parse("http://localhost").unwrap()],
            Url::parse("http://localhost:8000").unwrap()
        ));
        assert!(!is_origin_allowed(
            &[Url::parse("https://localhost").unwrap()],
            Url::parse("http://localhost").unwrap()
        ));
        assert!(!is_origin_allowed(
            &[Url::parse("https://example.test").unwrap()],
            Url::parse("http://subdomain.example.test").unwrap()
        ));
    }
}
