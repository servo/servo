/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs;
use std::path::PathBuf;

use embedder_traits::resources::{self, Resource, ResourceReaderMethods};
use getopts::Options;
use headers::{ContentType, HeaderMapExt};
use net::protocols::{ProtocolHandler, ProtocolRegistry};
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::ResourceFetchTiming;
use servo_config::opts::{self, ArgumentParsingResult};
use winit::window::WindowAttributes;

/// Configuration of Servo instance.
#[derive(Clone, Debug)]
pub struct Config {
    /// URL to load initially.
    pub url: Option<url::Url>,
    /// Should launch without control panel
    pub no_panel: bool,
    /// Window settings for the initial winit window
    pub window_attributes: WindowAttributes,
    /// Override the user agent
    pub user_agent: Option<String>,
}

impl Config {
    /// Create a new configuration and set the options for creating Servo instance.
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut opts = Options::new();
        opts.optflag("", "no-panel", "Launch Servo without control panel");
        opts.optopt(
            "u",
            "user-agent",
            "Override the user agent",
            "'ServoView/1.0'",
        );

        let matches = match opts::from_cmdline_args(opts, &args) {
            ArgumentParsingResult::ChromeProcess(matches) => matches,
            ArgumentParsingResult::ContentProcess(_matches, _token) => {
                todo!("Multiprocess mode hasn't supported yet")
            },
        };

        if opts::get().is_printing_version {
            println!("Servo kun v0.0.1");
            std::process::exit(0);
        }

        let no_panel = matches.opt_present("no-panel");
        let user_agent = matches.opt_str("u");
        let url = if !matches.free.is_empty() {
            &matches.free[0][..]
        } else {
            ""
        };
        let url = match url::Url::parse(&url) {
            Ok(url_parsed) => Some(url_parsed),
            Err(e) => {
                let mut u = None;
                if e == url::ParseError::RelativeUrlWithoutBase {
                    if let Ok(url_parsed) = url::Url::parse(&format!("https://{url}")) {
                        u = Some(url_parsed);
                    }
                }
                log::error!("Invalid initial url: {url}");
                u
            },
        };
        let mut window_attributes = winit::window::Window::default_attributes();
        window_attributes.maximized = true;
        // TODO: get opts window attributes

        Self {
            url,
            no_panel,
            window_attributes,
            user_agent,
        }
    }

    /// Register URL scheme protocols
    pub fn create_protocols(&self) -> ProtocolRegistry {
        let handler = ResourceReader(resources_dir_path());
        let mut protocols = ProtocolRegistry::with_internal_protocols();
        protocols.register("servo", handler);
        protocols
    }

    /// Init options and preferences.
    pub fn init(self) {
        // Set the resource files and preferences of Servo.
        resources::set(Box::new(ResourceReader(resources_dir_path())));
    }
}

struct ResourceReader(PathBuf);

impl ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let path = self.0.join(file.filename());
        // Rigppy image is the only one needs to be valid bytes.
        // Others can be empty and Servo will set to default.
        if let Resource::RippyPNG = file {
            fs::read(path).unwrap_or(include_bytes!("../../resources/rippy.png").to_vec())
        } else {
            fs::read(path).unwrap_or_default()
        }
    }

    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }

    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![]
    }
}

impl ProtocolHandler for ResourceReader {
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut net::fetch::methods::DoneChannel,
        _context: &net::fetch::methods::FetchContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {
        let path = self.0.join(request.current_url().domain().unwrap());

        let response = if let Ok(file) = fs::read(path) {
            let mut response = Response::new(
                request.current_url(),
                ResourceFetchTiming::new(request.timing_type()),
            );

            // Set Content-Type header.
            // TODO: We assume it's HTML for now. This should be updated once we have IPC interface.
            response.headers.typed_insert(ContentType::html());

            *response.body.lock().unwrap() = ResponseBody::Done(file);

            response
        } else {
            Response::network_internal_error("Opening file failed")
        };

        Box::pin(std::future::ready(response))
    }
}

/// Helper function to get default resource directory if it's not provided.
fn resources_dir_path() -> PathBuf {
    // TODO: opts::get().resources_path
    let root_dir = std::env::current_dir();

    root_dir.ok().map(|dir| dir.join("resources")).unwrap()
}
