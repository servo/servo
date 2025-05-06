/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::future::Future;
use std::ops::Bound;
use std::pin::Pin;

use headers::Range;
use http::StatusCode;
use log::error;
use net_traits::filemanager_thread::RelativePos;
use net_traits::request::Request;
use net_traits::response::Response;
use servo_url::ServoUrl;

use crate::fetch::methods::{DoneChannel, FetchContext, RangeRequestBounds};

mod blob;
mod data;
mod file;

use blob::BlobProtocolHander;
use data::DataProtocolHander;
use file::FileProtocolHander;

// The set of schemes that can't be registered.
static FORBIDDEN_SCHEMES: [&str; 4] = ["http", "https", "chrome", "about"];

pub trait ProtocolHandler: Send + Sync {
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
}

#[derive(Default)]
pub struct ProtocolRegistry {
    pub(crate) handlers: HashMap<String, Box<dyn ProtocolHandler>>, // Maps scheme -> handler
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

    pub fn get(&self, scheme: &str) -> Option<&dyn ProtocolHandler> {
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
