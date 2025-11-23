/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::Entry;
use std::future;
use std::future::Future;
use std::ops::Bound;
use std::pin::Pin;

use headers::Range;
use http::StatusCode;
use log::error;
use net_traits::NetworkError;
use net_traits::filemanager_thread::RelativePos;
use net_traits::request::Request;
use net_traits::response::Response;
use rustc_hash::FxHashMap;
use servo_url::ServoUrl;

use crate::fetch::methods::{
    DoneChannel, FetchContext, FetchResponseCollector, RangeRequestBounds, fetch,
};

mod blob;
mod data;
mod file;

use blob::BlobProtocolHander;
use data::DataProtocolHander;
use file::FileProtocolHander;

// The set of schemes that can't be registered.
static FORBIDDEN_SCHEMES: [&str; 4] = ["http", "https", "chrome", "about"];

pub trait ProtocolHandler: Send + Sync {
    /// A list of schema-less URLs that can be resolved against this handler's
    /// scheme. These URLs will be granted access to the `navigator.servo`
    /// interface to perform privileged operations that manipulate Servo internals.
    fn privileged_paths(&self) -> &'static [&'static str] {
        &[]
    }

    /// Triggers the load of a resource for this protocol and returns a future
    /// that will produce a Response. Even if the protocol is not backed by a
    /// http endpoint, it is recommended to a least provide:
    /// - A relevant status code.
    /// - A Content Type.
    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>>;

    /// Specify if resources served by that protocol can be retrieved
    /// with `fetch()` without no-cors mode to allow the caller direct
    /// access to the resource content.
    fn is_fetchable(&self) -> bool {
        false
    }

    /// Specify if this custom protocol can be used in a [secure context]
    ///
    /// Note: this only works for bypassing mixed content checks right now
    ///
    /// [secure context]: https://developer.mozilla.org/en-US/docs/Web/Security/Secure_Contexts
    fn is_secure(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn ProtocolHandler>;
}

#[derive(Default)]
pub struct ProtocolRegistry {
    pub(crate) handlers: FxHashMap<String, Box<dyn ProtocolHandler>>, // Maps scheme -> handler
}

impl Clone for ProtocolRegistry {
    fn clone(&self) -> ProtocolRegistry {
        let mut handlers: FxHashMap<String, Box<dyn ProtocolHandler>> = FxHashMap::default();
        for (scheme, handler) in self.handlers.iter() {
            handlers.insert(scheme.clone(), handler.clone_box());
        }
        Self { handlers }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ProtocolRegisterError {
    ForbiddenScheme,
    SchemeAlreadyRegistered,
}

impl ProtocolRegistry {
    pub fn with_internal_protocols() -> Self {
        let mut registry = Self::default();
        // We just created a new registry, and know that we aren't using
        // any forbidden schemes, so this should never panic.
        registry
            .register("data", DataProtocolHander::default())
            .expect("Infallible");
        registry
            .register("blob", BlobProtocolHander::default())
            .expect("Infallible");
        registry
            .register("file", FileProtocolHander::default())
            .expect("Infallible");
        registry
    }

    /// Do not allow users to enter an arbitrary protocol as this can lead to slowdowns.
    pub fn register(
        &mut self,
        scheme: &str,
        handler: impl ProtocolHandler + 'static,
    ) -> Result<(), ProtocolRegisterError> {
        if FORBIDDEN_SCHEMES.contains(&scheme) {
            error!("Protocol handler for '{scheme}' is not allowed to be registered.");
            return Err(ProtocolRegisterError::ForbiddenScheme);
        }

        if let Entry::Vacant(entry) = self.handlers.entry(scheme.into()) {
            entry.insert(Box::new(handler));
            Ok(())
        } else {
            error!("Protocol handler for '{scheme}' is already registered.");
            Err(ProtocolRegisterError::SchemeAlreadyRegistered)
        }
    }

    pub fn register_page_content_handler(
        &mut self,
        scheme: String,
        url: String,
    ) -> Result<(), ProtocolRegisterError> {
        self.register(
            &scheme.clone(),
            WebpageContentProtocolHandler { url, scheme },
        )
    }

    pub fn get(&self, scheme: &str) -> Option<&dyn ProtocolHandler> {
        println!("Looking up for scheme {}", scheme);
        self.handlers.get(scheme).map(|e| e.as_ref())
    }

    pub fn merge(&mut self, mut other: ProtocolRegistry) {
        for (scheme, handler) in other.handlers.drain() {
            if FORBIDDEN_SCHEMES.contains(&scheme.as_str()) {
                error!("Protocol handler for '{scheme}' is not allowed to be registered.");
                continue;
            }

            self.handlers.entry(scheme).or_insert(handler);
        }
    }

    pub fn is_fetchable(&self, scheme: &str) -> bool {
        self.handlers
            .get(scheme)
            .is_some_and(|handler| handler.is_fetchable())
    }

    pub fn is_secure(&self, scheme: &str) -> bool {
        self.handlers
            .get(scheme)
            .is_some_and(|handler| handler.is_secure())
    }

    pub fn privileged_urls(&self) -> Vec<ServoUrl> {
        self.handlers
            .iter()
            .flat_map(|(scheme, handler)| {
                let paths = handler.privileged_paths();
                paths
                    .iter()
                    .filter_map(move |path| ServoUrl::parse(&format!("{scheme}:{path}")).ok())
            })
            .collect()
    }
}

struct WebpageContentProtocolHandler {
    url: String,
    scheme: String,
}

impl ProtocolHandler for WebpageContentProtocolHandler {
    /// <https://html.spec.whatwg.org/multipage/#protocol-handler-invocation>
    fn load(
        &self,
        request: &mut Request,
        _done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let mut url = request.current_url();
        // Step 1. Assert: inputURL's scheme is normalizedScheme.
        assert!(url.scheme() == self.scheme);
        // Step 2. Set the username given inputURL and the empty string.
        //
        // Ignore errors if no username can be set, which depending on the
        // URL in the scheme handler might be bogus. For example, with a
        // `mailto:` handler, it doesn't have a base. See the
        // documentation at [`url::Url::set_username`]
        let _ = url.set_username("");

        // Step 3. Set the password given inputURL and the empty string.
        //
        // Ignore errors if no password can be set, which depending on the
        // URL in the scheme handler might be bogus. For example, with a
        // `mailto:` handler, it doesn't have a base. See the
        // documentation at [`url::Url::set_password`]
        let _ = url.set_password(None);

        // Step 4. Let inputURLString be the serialization of inputURL.
        // Step 5. Let encodedURL be the result of running UTF-8 percent-encode on inputURLString using the component percent-encode set.
        //
        // Url is already UTF-8, so encoding isn't required
        let encoded_url = &url.as_str()[(self.scheme.len() + 1)..];
        // Step 6. Let handlerURLString be normalizedURLString.
        // Step 7. Replace the first instance of "%s" in handlerURLString with encodedURL.
        let handler_url_string = self.url.replacen("%s", encoded_url, 1);
        // Step 8. Let resultURL be the result of parsing handlerURLString.
        let Ok(result_url) = ServoUrl::parse(&handler_url_string) else {
            return Box::pin(future::ready(Response::network_error(
                NetworkError::Internal("Failed to parse substituted protocol handler url".into()),
            )));
        };
        println!("Final url {}", result_url);
        // Step 9. Navigate an appropriate navigable to resultURL.
        request.url_list.push(result_url);
        let request2 = request.clone();
        let context2 = context.clone();
        Box::pin(async move {
            let (sender, receiver) = tokio::sync::oneshot::channel();
            let mut collector = FetchResponseCollector {
                sender: Some(sender),
            };
            fetch(request2, &mut collector, &context2).await;
            receiver.await.unwrap()
        })
    }

    fn clone_box(&self) -> Box<dyn ProtocolHandler> {
        Box::new(WebpageContentProtocolHandler {
            scheme: self.scheme.clone(),
            url: self.url.clone(),
        })
    }
}

/// Test if the URL is potentially trustworthy or the custom protocol is registered as secure
pub fn is_url_potentially_trustworthy(
    protocol_registry: &ProtocolRegistry,
    url: &ServoUrl,
) -> bool {
    url.is_potentially_trustworthy() || protocol_registry.is_secure(url.scheme())
}

pub fn range_not_satisfiable_error(response: &mut Response) {
    response.status = StatusCode::RANGE_NOT_SATISFIABLE.into();
}

/// Get the range bounds if the `Range` header is present.
pub fn get_range_request_bounds(range: Option<Range>, len: u64) -> RangeRequestBounds {
    if let Some(ref range) = range {
        let (start, end) = match range.satisfiable_ranges(len).next() {
            Some((Bound::Included(start), Bound::Unbounded)) => (start, None),
            Some((Bound::Included(start), Bound::Included(end))) => {
                // `end` should be less or equal to `start`.
                (start, Some(i64::max(start as i64, end as i64)))
            },
            Some((Bound::Unbounded, Bound::Included(offset))) => {
                return RangeRequestBounds::Pending(offset);
            },
            _ => (0, None),
        };
        RangeRequestBounds::Final(RelativePos::from_opts(Some(start as i64), end))
    } else {
        RangeRequestBounds::Final(RelativePos::from_opts(Some(0), None))
    }
}

pub fn partial_content(response: &mut Response) {
    response.status = StatusCode::PARTIAL_CONTENT.into();
}
