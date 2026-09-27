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
//! - servo:processes
//! - servo:process-list
//! - servo:history

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use headers::{ContentType, HeaderMapExt};
use servo::UserAgentPlatform;
use servo::protocol_handler::{
    DoneChannel, FetchContext, NetworkError, Origin, ProtocolHandler, Referrer, Request,
    ResourceFetchTiming, Response, ResponseBody,
};

use crate::desktop::protocols::processes::ProcessSampler;
use crate::desktop::protocols::resource::ResourceProtocolHandler;
use crate::prefs::EXPERIMENTAL_PREFS;

pub struct ServoProtocolHandler {
    process_sampler: Mutex<ProcessSampler>,
}

impl Default for ServoProtocolHandler {
    fn default() -> Self {
        Self {
            process_sampler: Mutex::new(ProcessSampler::default()),
        }
    }
}

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

            "processes" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/processes.html",
            ),

            "history" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/history.html",
            ),

            "process-list" => {
                if request_is_from_web_content(request) {
                    return Box::pin(std::future::ready(Response::network_error(
                        NetworkError::ResourceLoadError("Forbidden".to_owned()),
                    )));
                }
                let body = match self.process_sampler.lock() {
                    Ok(mut sampler) => sampler.snapshot_json(),
                    Err(poisoned) => poisoned.into_inner().snapshot_json(),
                };
                json_response(request, body)
            },

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

            _ => Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid shortcut".to_owned()),
            ))),
        }
    }
}

fn request_is_from_web_content(request: &Request) -> bool {
    fn is_web_scheme(scheme: &str) -> bool {
        matches!(scheme, "http" | "https" | "ftp" | "ws" | "wss")
    }

    if let Origin::Origin(origin) = &request.origin &&
        origin.scheme().is_some_and(is_web_scheme)
    {
        return true;
    }
    match &request.referrer {
        Referrer::Client(url) | Referrer::ReferrerUrl(url) => is_web_scheme(url.scheme()),
        Referrer::NoReferrer => false,
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
