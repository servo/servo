/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Bound;
use std::pin::Pin;

use headers::Range;
use http::StatusCode;
use log::error;
use net_traits::filemanager_thread::RelativePos;
use net_traits::request::Request;
use net_traits::response::Response;

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
}

#[derive(Default)]
pub struct ProtocolRegistry {
    pub(crate) handlers: HashMap<String, Box<dyn ProtocolHandler>>, // Maps scheme -> handler
}

impl ProtocolRegistry {
    pub fn with_internal_protocols() -> Self {
        let mut registry = Self::default();
        registry.register("data", DataProtocolHander::default());
        registry.register("blob", BlobProtocolHander::default());
        registry.register("file", FileProtocolHander::default());
        registry
    }

    pub fn register(&mut self, scheme: &str, handler: impl ProtocolHandler + 'static) -> bool {
        if FORBIDDEN_SCHEMES.contains(&scheme) {
            error!("Protocol handler for '{scheme}' is not allowed to be registered.");
            return false;
        }

        if let Entry::Vacant(entry) = self.handlers.entry(scheme.into()) {
            entry.insert(Box::new(handler));
            true
        } else {
            error!("Protocol handler for '{scheme}' is already registered.");
            false
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
            .map(|handler| handler.is_fetchable())
            .unwrap_or(false)
    }
}

pub fn range_not_satisfiable_error(response: &mut Response) {
    response.status = StatusCode::RANGE_NOT_SATISFIABLE.into();
}

/// Get the range bounds if the `Range` header is present.
pub fn get_range_request_bounds(range: Option<Range>) -> RangeRequestBounds {
    if let Some(ref range) = range {
        let (start, end) = match range
            .iter()
            .collect::<Vec<(Bound<u64>, Bound<u64>)>>()
            .first()
        {
            Some(&(Bound::Included(start), Bound::Unbounded)) => (start, None),
            Some(&(Bound::Included(start), Bound::Included(end))) => {
                // `end` should be less or equal to `start`.
                (start, Some(i64::max(start as i64, end as i64)))
            },
            Some(&(Bound::Unbounded, Bound::Included(offset))) => {
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
