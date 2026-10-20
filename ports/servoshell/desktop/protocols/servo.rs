/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Loads resources using a mapping from well-known shortcuts to resource: urls.
//! Recognized shortcuts:
//! - servo:default-user-agent
//! - servo:experimental-preferences
//! - servo:config
//! - servo:newtab
//! - servo:preferences

use std::future::Future;
use std::pin::Pin;

use headers::{ContentType, HeaderMapExt};
use servo::UserAgentPlatform;
use servo::protocol_handler::{
    DoneChannel, FetchContext, NetworkError, ProtocolHandler, Request, ResourceFetchTiming,
    Response, ResponseBody,
};

use crate::desktop::protocols::resource::ResourceProtocolHandler;
use crate::prefs::EXPERIMENTAL_PREFS;

#[derive(Default)]
pub struct ServoProtocolHandler {}

impl ProtocolHandler for ServoProtocolHandler {
    fn privileged_paths(&self) -> &'static [&'static str] {
        &["config", "preferences"]
    }

    fn is_fetchable(&self) -> bool {
        true
    }

    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        match url.path() {
            "config" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/config.html",
            ),
            "newtab" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/newtab.html",
            ),

            "preferences" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/preferences.html",
            ),

            "license" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/license.html",
            ),

            "experimental-preferences" => {
                let pref_list = EXPERIMENTAL_PREFS
                    .iter()
                    .map(|pref| format!("\"{pref}\""))
                    .collect::<Vec<String>>()
                    .join(",");
                json_response(request, format!("[{pref_list}]"))
            },

            "default-user-agent" => {
                let user_agent = UserAgentPlatform::default().to_user_agent_string();
                json_response(request, format!("\"{user_agent}\""))
            },

            "about" | "version" => {
                let user_agent = UserAgentPlatform::default().to_user_agent_string();
                let body = format!(
                    "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>About Servo</title><link rel=\"stylesheet\" href=\"resource:///newtab.css\" /></head><body><div style=\"max-width: 600px; margin: 40px auto; font-family: sans-serif;\"><h1>Servo Browser</h1><p><b>Engine:</b> Servo Full Browser Engine</p><p><b>User Agent:</b> {}</p><hr/><p><a href=\"servo:newtab\">New Tab</a> &bull; <a href=\"servo:history\">History</a> &bull; <a href=\"servo:downloads\">Downloads</a> &bull; <a href=\"servo:preferences\">Preferences</a> &bull; <a href=\"servo:config\">Config</a> &bull; <a href=\"servo:license\">License</a></p></div></body></html>",
                    user_agent
                );
                html_response(request, body)
            },

            "downloads" => {
                let body = format!(
                    "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>Downloads</title><link rel=\"stylesheet\" href=\"resource:///newtab.css\" /></head><body><div style=\"max-width: 600px; margin: 40px auto; font-family: sans-serif;\"><h1>Downloads</h1><p>Active and completed downloads will appear here.</p><hr/><p><a href=\"servo:newtab\">New Tab</a> &bull; <a href=\"servo:history\">History</a> &bull; <a href=\"servo:about\">About</a></p></div></body></html>"
                );
                html_response(request, body)
            },

            "history" => {
                let items = crate::prefs::default_config_dir()
                    .map(|mut p| {
                        p.push("history.json");
                        p
                    })
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .and_then(|data| serde_json::from_str::<Vec<serde_json::Value>>(&data).ok())
                    .unwrap_or_default();

                let mut list_html = String::new();
                for item in items.iter().rev() {
                    if let Some(url) = item.get("url").and_then(|u| u.as_str()) {
                        list_html.push_str(&format!(
                            "<li><a href=\"{url}\">{url}</a></li>"
                        ));
                    }
                }
                if list_html.is_empty() {
                    list_html = "<p>No history entries recorded yet.</p>".to_string();
                } else {
                    list_html = format!("<ul>{list_html}</ul>");
                }

                let body = format!(
                    "<!DOCTYPE html><html><head><meta charset=\"UTF-8\"><title>Browsing History</title><link rel=\"stylesheet\" href=\"resource:///newtab.css\" /></head><body><div style=\"max-width: 600px; margin: 40px auto; font-family: sans-serif;\"><h1>Browsing History</h1>{}<hr/><p><a href=\"servo:newtab\">New Tab</a> &bull; <a href=\"servo:about\">About</a></p></div></body></html>",
                    list_html
                );
                html_response(request, body)
            },

            _ => Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid shortcut".to_owned()),
            ))),
        }
    }
}

fn json_response(
    request: &Request,
    body: String,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    let mut response = Response::new(
        request.current_url(),
        ResourceFetchTiming::new(request.timing_type()),
    );
    response.headers.typed_insert(ContentType::json());
    *response.body.lock() = ResponseBody::Done(body.into_bytes());
    Box::pin(std::future::ready(response))
}

fn html_response(
    request: &Request,
    body: String,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    let mut response = Response::new(
        request.current_url(),
        ResourceFetchTiming::new(request.timing_type()),
    );
    response.headers.typed_insert(ContentType::html());
    *response.body.lock() = ResponseBody::Done(body.into_bytes());
    Box::pin(std::future::ready(response))
}
