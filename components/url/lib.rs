/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![crate_name = "servo_url"]
#![crate_type = "rlib"]

#[macro_use]
extern crate malloc_size_of;
#[macro_use]
extern crate malloc_size_of_derive;
#[macro_use]
extern crate serde;

pub mod origin;

pub use crate::origin::{ImmutableOrigin, MutableOrigin, OpaqueOrigin};

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::Hasher;
use std::net::IpAddr;
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};
use std::path::Path;
use std::sync::Arc;
use to_shmem::{SharedMemoryBuilder, ToShmem};
use url::{Position, Url};

pub use url::Host;

#[derive(Clone, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ServoUrl(#[ignore_malloc_size_of = "Arc"] Arc<Url>);

impl ToShmem for ServoUrl {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> to_shmem::Result<Self> {
        unimplemented!("If servo wants to share stylesheets across processes, ToShmem for Url must be implemented")
    }
}

impl ServoUrl {
    pub fn from_url(url: Url) -> Self {
        ServoUrl(Arc::new(url))
    }

    pub fn parse_with_base(base: Option<&Self>, input: &str) -> Result<Self, url::ParseError> {
        Url::options()
            .base_url(base.map(|b| &*b.0))
            .parse(input)
            .map(Self::from_url)
    }

    pub fn into_string(self) -> String {
        Arc::try_unwrap(self.0)
            .unwrap_or_else(|s| (*s).clone())
            .into_string()
    }

    pub fn into_url(self) -> Url {
        Arc::try_unwrap(self.0).unwrap_or_else(|s| (*s).clone())
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn parse(input: &str) -> Result<Self, url::ParseError> {
        Url::parse(input).map(Self::from_url)
    }

    pub fn cannot_be_a_base(&self) -> bool {
        self.0.cannot_be_a_base()
    }

    pub fn domain(&self) -> Option<&str> {
        self.0.domain()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.0.fragment()
    }

    pub fn path(&self) -> &str {
        self.0.path()
    }

    pub fn origin(&self) -> ImmutableOrigin {
        ImmutableOrigin::new(self.0.origin())
    }

    pub fn scheme(&self) -> &str {
        self.0.scheme()
    }

    pub fn is_secure_scheme(&self) -> bool {
        let scheme = self.scheme();
        scheme == "https" || scheme == "wss"
    }

    /// <https://fetch.spec.whatwg.org/#local-scheme>
    pub fn is_local_scheme(&self) -> bool {
        let scheme = self.scheme();
        scheme == "about" || scheme == "blob" || scheme == "data"
    }

    pub fn chrome_rules_enabled(&self) -> bool {
        self.is_chrome()
    }

    pub fn is_chrome(&self) -> bool {
        self.scheme() == "chrome"
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_mut_url(&mut self) -> &mut Url {
        Arc::make_mut(&mut self.0)
    }

    pub fn set_username(&mut self, user: &str) -> Result<(), ()> {
        self.as_mut_url().set_username(user)
    }

    pub fn set_ip_host(&mut self, addr: IpAddr) -> Result<(), ()> {
        self.as_mut_url().set_ip_host(addr)
    }

    pub fn set_password(&mut self, pass: Option<&str>) -> Result<(), ()> {
        self.as_mut_url().set_password(pass)
    }

    pub fn set_fragment(&mut self, fragment: Option<&str>) {
        self.as_mut_url().set_fragment(fragment)
    }

    pub fn username(&self) -> &str {
        self.0.username()
    }

    pub fn password(&self) -> Option<&str> {
        self.0.password()
    }

    pub fn to_file_path(&self) -> Result<::std::path::PathBuf, ()> {
        self.0.to_file_path()
    }

    pub fn host(&self) -> Option<url::Host<&str>> {
        self.0.host()
    }

    pub fn host_str(&self) -> Option<&str> {
        self.0.host_str()
    }

    pub fn port(&self) -> Option<u16> {
        self.0.port()
    }

    pub fn port_or_known_default(&self) -> Option<u16> {
        self.0.port_or_known_default()
    }

    pub fn join(&self, input: &str) -> Result<ServoUrl, url::ParseError> {
        self.0.join(input).map(Self::from_url)
    }

    pub fn path_segments(&self) -> Option<::std::str::Split<char>> {
        self.0.path_segments()
    }

    pub fn query(&self) -> Option<&str> {
        self.0.query()
    }

    pub fn from_file_path<P: AsRef<Path>>(path: P) -> Result<Self, ()> {
        Ok(Self::from_url(Url::from_file_path(path)?))
    }

    /// Return a non-standard shortened form of the URL. Mainly intended to be
    /// used for debug printing in a constrained space (e.g., thread names).
    pub fn debug_compact(&self) -> impl std::fmt::Display + '_ {
        match self.scheme() {
            "http" | "https" => {
                // Strip `scheme://`, which is hardly useful for identifying websites
                let mut st = self.as_str();
                st = st.strip_prefix(self.scheme()).unwrap_or(st);
                st = st.strip_prefix(":").unwrap_or(st);
                st = st.trim_start_matches('/');

                // Don't want to return an empty string
                if st.is_empty() {
                    st = self.as_str();
                }

                st
            },
            "file" => {
                // The only useful part in a `file` URL is usually only the last
                // few components
                let path = self.path();
                let i = path.rfind('/');
                let i = i.map(|i| path[..i].rfind('/').unwrap_or(i));
                match i {
                    None | Some(0) => path,
                    Some(i) => &path[i + 1..],
                }
            },
            _ => self.as_str(),
        }
    }

    /// <https://w3c.github.io/webappsec-secure-contexts/#potentially-trustworthy-url>
    pub fn is_potentially_trustworthy(&self) -> bool {
        // Step 1
        if self.as_str() == "about:blank" || self.as_str() == "about:srcdoc" {
            return true;
        }
        // Step 2
        if self.scheme() == "data" {
            return true;
        }
        // Step 3
        self.is_origin_trustworthy()
    }

    /// <https://w3c.github.io/webappsec-secure-contexts/#is-origin-trustworthy>
    pub fn is_origin_trustworthy(&self) -> bool {
        // Step 1
        if !self.origin().is_tuple() {
            return false;
        }

        // Step 3
        if self.scheme() == "https" || self.scheme() == "wss" {
            true
        // Steps 4-5
        } else if self.host().is_some() {
            let host = self.host_str().unwrap();
            // Step 4
            if let Ok(ip_addr) = host.parse::<IpAddr>() {
                ip_addr.is_loopback()
            // Step 5
            } else {
                host == "localhost" || host.ends_with(".localhost")
            }
        // Step 6
        } else {
            self.scheme() == "file"
        }
    }
}

impl fmt::Display for ServoUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Debug for ServoUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0.as_str().len() > 40 {
            let mut hasher = DefaultHasher::new();
            hasher.write(self.0.as_str().as_bytes());
            let truncated: String = self.0.as_str().chars().take(40).collect();
            let result = format!("{}... ({:x})", truncated, hasher.finish());
            return result.fmt(formatter);
        }
        self.0.fmt(formatter)
    }
}

impl Index<RangeFull> for ServoUrl {
    type Output = str;
    fn index(&self, _: RangeFull) -> &str {
        &self.0[..]
    }
}

impl Index<RangeFrom<Position>> for ServoUrl {
    type Output = str;
    fn index(&self, range: RangeFrom<Position>) -> &str {
        &self.0[range]
    }
}

impl Index<RangeTo<Position>> for ServoUrl {
    type Output = str;
    fn index(&self, range: RangeTo<Position>) -> &str {
        &self.0[range]
    }
}

impl Index<Range<Position>> for ServoUrl {
    type Output = str;
    fn index(&self, range: Range<Position>) -> &str {
        &self.0[range]
    }
}

impl From<Url> for ServoUrl {
    fn from(url: Url) -> Self {
        ServoUrl::from_url(url)
    }
}
