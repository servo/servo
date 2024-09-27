/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::str::FromStr;

use content_security_policy::Destination;
use headers::{Error, Header, HeaderMap};
use http::{HeaderName, HeaderValue};
use net_traits::fetch::headers::get_decode_and_split_header_name;

static SEC_FETCH_DEST: HeaderName = HeaderName::from_static("Sec-Fetch-Dest");

/// <https://fetch.spec.whatwg.org/#determine-nosniff>
pub fn determine_nosniff(headers: &HeaderMap) -> bool {
    let values = get_decode_and_split_header_name("x-content-type-options", headers);

    match values {
        None => false,
        Some(values) => !values.is_empty() && values[0].eq_ignore_ascii_case("nosniff"),
    }
}

pub struct SecFetchDest(pub Destination);

impl Header for SecFetchDest {
    fn name() -> &'static HeaderName {
        &SEC_FETCH_DEST
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .next()
            .ok_or(Error::invalid())?
            .to_str()
            .map_err(|_| Error::invalid())?;

        // The empty destination serializes as the string "empty"
        if value == "empty" {
            return Ok(Destination::None.into());
        }

        let sec_fetch_dest = destination_from_str(value).map_err(|_| Error::invalid())?;

        Ok(sec_fetch_dest.into())
    }
    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = HeaderValue::from_static(destination_as_str(&self.0));
        values.extend(std::iter::once(value));
    }
}

impl From<Destination> for SecFetchDest {
    fn from(value: Destination) -> Self {
        Self(value)
    }
}

// FIXME: This functionality is in content-security-policy!
// (https://github.com/rust-ammonia/rust-content-security-policy/commit/ec258813f4dbacf1b601c99980e04d3dcbfaa6f1)
// But using it requires upgrading both servo and stylo (because of MallocSizeOf) to a newer version of the csp crate...

pub struct InvalidDestination;

fn destination_from_str(s: &str) -> Result<Destination, InvalidDestination> {
    let destination = match s {
        "" => Destination::None,
        "audio" => Destination::Audio,
        "audioworklet" => Destination::AudioWorklet,
        "document" => Destination::Document,
        "embed" => Destination::Embed,
        "font" => Destination::Font,
        "frame" => Destination::Frame,
        "iframe" => Destination::IFrame,
        "image" => Destination::Image,
        "json" => Destination::Json,
        "manifest" => Destination::Manifest,
        "object" => Destination::Object,
        "paintworklet" => Destination::PaintWorklet,
        "report" => Destination::Report,
        "script" => Destination::Script,
        "serviceworker" => Destination::ServiceWorker,
        "sharedworker" => Destination::SharedWorker,
        "style" => Destination::Style,
        "track" => Destination::Track,
        "video" => Destination::Video,
        "webidentity" => Destination::WebIdentity,
        "worker" => Destination::Worker,
        "xslt" => Destination::Xslt,
        _ => return Err(InvalidDestination),
    };

    Ok(destination)
}


const fn destination_as_str(destination: &Destination) -> &'static str {
    match destination {
        Destination::None => "",
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