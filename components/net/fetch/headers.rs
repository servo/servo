/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use content_security_policy::Destination;
use headers::{Error, Header, HeaderMap};
use http::{HeaderName, HeaderValue};
use net_traits::fetch::headers::get_decode_and_split_header_name;
use net_traits::request::RequestMode;

static SEC_FETCH_DEST: HeaderName = HeaderName::from_static("sec-fetch-dest");

static SEC_FETCH_MODE: HeaderName = HeaderName::from_static("sec-fetch-mode");

static SEC_FETCH_SITE: HeaderName = HeaderName::from_static("sec-fetch-site");

static SEC_FETCH_USER: HeaderName = HeaderName::from_static("sec-fetch-user");

/// <https://fetch.spec.whatwg.org/#determine-nosniff>
pub fn determine_nosniff(headers: &HeaderMap) -> bool {
    let values = get_decode_and_split_header_name("x-content-type-options", headers);

    match values {
        None => false,
        Some(values) => !values.is_empty() && values[0].eq_ignore_ascii_case("nosniff"),
    }
}

/// The `sec-fetch-dest` header
pub struct SecFetchDest(pub Destination);

/// The `sec-fetch-mode` header
///
/// This is effectively the same as a [RequestMode], except
/// it doesn't keep track of the websocket protocols
pub enum SecFetchMode {
    SameOrigin,
    Cors,
    NoCors,
    Navigate,
    WebSocket,
}

/// The `sec-fetch-user` header
pub struct SecFetchUser;

/// The `sec-fetch-site` header
#[derive(Eq, PartialEq)]
pub enum SecFetchSite {
    None,
    SameOrigin,
    SameSite,
    CrossSite,
}

impl Header for SecFetchDest {
    fn name() -> &'static HeaderName {
        &SEC_FETCH_DEST
    }

    fn decode<'i, I>(_: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // TODO
        Err(Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static(destination_as_str(self.0));
        values.extend(std::iter::once(value));
    }
}

impl From<Destination> for SecFetchDest {
    fn from(value: Destination) -> Self {
        Self(value)
    }
}

impl Header for SecFetchMode {
    fn name() -> &'static HeaderName {
        &SEC_FETCH_MODE
    }

    fn decode<'i, I>(_: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // TODO
        Err(Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static(self.as_str());
        values.extend(std::iter::once(value));
    }
}

impl<'a> From<&'a RequestMode> for SecFetchMode {
    fn from(value: &'a RequestMode) -> Self {
        match value {
            RequestMode::SameOrigin => Self::SameOrigin,
            RequestMode::CorsMode => Self::Cors,
            RequestMode::NoCors => Self::NoCors,
            RequestMode::Navigate => Self::Navigate,
            RequestMode::WebSocket { .. } => Self::WebSocket,
        }
    }
}

impl Header for SecFetchSite {
    fn name() -> &'static HeaderName {
        &SEC_FETCH_SITE
    }

    fn decode<'i, I>(_: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // TODO
        Err(Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let s = match self {
            Self::None => "none",
            Self::SameSite => "same-site",
            Self::CrossSite => "cross-site",
            Self::SameOrigin => "same-origin",
        };
        let value = HeaderValue::from_static(s);
        values.extend(std::iter::once(value));
    }
}

impl Header for SecFetchUser {
    fn name() -> &'static HeaderName {
        &SEC_FETCH_USER
    }

    fn decode<'i, I>(_: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        // TODO
        Err(Error::invalid())
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static("?1");
        values.extend(std::iter::once(value));
    }
}

const fn destination_as_str(destination: Destination) -> &'static str {
    match destination {
        Destination::None => "empty",
        Destination::Audio => "audio",
        Destination::AudioWorklet => "audioworklet",
        Destination::Document => "document",
        Destination::Embed => "embed",
        Destination::Font => "font",
        Destination::Frame => "frame",
        Destination::IFrame => "iframe",
        Destination::Image => "image",
        Destination::Json => "json",
        Destination::Manifest => "manifest",
        Destination::Object => "object",
        Destination::PaintWorklet => "paintworklet",
        Destination::Report => "report",
        Destination::Script => "script",
        Destination::ServiceWorker => "serviceworker",
        Destination::SharedWorker => "sharedworker",
        Destination::Style => "style",
        Destination::Track => "track",
        Destination::Video => "video",
        Destination::WebIdentity => "webidentity",
        Destination::Worker => "worker",
        Destination::Xslt => "xslt",
    }
}

impl SecFetchMode {
    /// Converts to the spec representation of a [RequestMode]
    fn as_str(&self) -> &'static str {
        match self {
            Self::SameOrigin => "same-origin",
            Self::Cors => "cors",
            Self::NoCors => "no-cors",
            Self::Navigate => "navigate",
            Self::WebSocket => "websocket",
        }
    }
}
