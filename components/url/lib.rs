/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#![crate_name = "servo_url"]
#![crate_type = "rlib"]

#[cfg(feature = "servo")] #[macro_use] extern crate heapsize;
#[cfg(feature = "servo")] #[macro_use] extern crate heapsize_derive;
#[cfg(feature = "servo")] extern crate serde;
#[cfg(feature = "servo")] #[macro_use] extern crate serde_derive;
#[cfg(feature = "servo")] extern crate url_serde;

extern crate servo_rand;
extern crate url;
extern crate uuid;

pub mod origin;

pub use origin::{OpaqueOrigin, ImmutableOrigin, MutableOrigin};

use std::fmt;
use std::net::IpAddr;
use std::ops::{Range, RangeFrom, RangeTo, RangeFull, Index};
use std::path::Path;
use std::sync::Arc;
use url::{Url, Position};

pub use url::Host;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ServoUrl(Arc<Url>);

impl ServoUrl {
    pub fn from_url(url: Url) -> Self {
        ServoUrl(Arc::new(url))
    }

    pub fn parse_with_base(base: Option<&Self>, input: &str) -> Result<Self, url::ParseError> {
        Url::options().base_url(base.map(|b| &*b.0)).parse(input).map(Self::from_url)
    }

    pub fn into_string(self) -> String {
        Arc::try_unwrap(self.0).unwrap_or_else(|s| (*s).clone()).into_string()
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
        Ok(Self::from_url(try!(Url::from_file_path(path))))
    }
}

impl fmt::Display for ServoUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Debug for ServoUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
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

#[cfg(feature = "servo")]
impl serde::Serialize for ServoUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        url_serde::serialize(&*self.0, serializer)
    }
}

#[cfg(feature = "servo")]
impl serde::Deserialize for ServoUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer,
    {
        url_serde::deserialize(deserializer).map(Self::from_url)
    }
}
