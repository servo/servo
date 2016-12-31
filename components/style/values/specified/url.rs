/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use cssparser::{CssStringWriter, Parser};
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use parser::ParserContextExtraData;
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::fmt::{self, Write};
use std::sync::Arc;
use style_traits::ToCss;
use values::NoViewportPercentage;
use values::computed::ComputedValueAsSpecified;

/// A set of data needed in Gecko to represent a URL.
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Serialize, Deserialize, Eq))]
pub struct UrlExtraData {
    /// The base URI.
    #[cfg(feature = "gecko")]
    pub base: GeckoArcURI,
    /// The referrer.
    #[cfg(feature = "gecko")]
    pub referrer: GeckoArcURI,
    /// The principal that originated this URI.
    #[cfg(feature = "gecko")]
    pub principal: GeckoArcPrincipal,
}

impl UrlExtraData {
    /// Constructs a `UrlExtraData`.
    #[cfg(feature = "servo")]
    pub fn make_from(_: &ParserContext) -> Option<UrlExtraData> {
        Some(UrlExtraData { })
    }

    /// Constructs a `UrlExtraData`.
    #[cfg(feature = "gecko")]
    pub fn make_from(context: &ParserContext) -> Option<UrlExtraData> {
        match context.extra_data {
            ParserContextExtraData {
                base: Some(ref base),
                referrer: Some(ref referrer),
                principal: Some(ref principal),
            } => {
                Some(UrlExtraData {
                    base: base.clone(),
                    referrer: referrer.clone(),
                    principal: principal.clone(),
                })
            },
            _ => None,
        }
    }
}

/// A specified url() value.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Serialize, Deserialize))]
pub struct SpecifiedUrl {
    /// The original URI. This might be optional since we may insert computed
    /// values of images into the cascade directly, and we don't bother to
    /// convert their serialization.
    ///
    /// Refcounted since cloning this should be cheap and data: uris can be
    /// really large.
    original: Option<Arc<String>>,

    /// The resolved value for the url, if valid.
    resolved: Option<ServoUrl>,

    /// Extra data used for Stylo.
    extra_data: UrlExtraData,
}

impl Parse for SpecifiedUrl {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let url = try!(input.expect_url());
        Self::parse_from_string(url, context)
    }
}

impl SpecifiedUrl {
    /// Try to parse a URL from a string value that is a valid CSS token for a
    /// URL.
    ///
    /// Only returns `Err` for Gecko, in the case we can't construct a
    /// `URLExtraData`.
    pub fn parse_from_string<'a>(url: Cow<'a, str>,
                                 context: &ParserContext)
                                 -> Result<Self, ()> {
        let extra_data = match UrlExtraData::make_from(context) {
            Some(extra_data) => extra_data,
            None => {
                // FIXME(heycam) should ensure we always have a principal, etc.,
                // when parsing style attributes and re-parsing due to CSS
                // Variables.
                println!("stylo: skipping declaration without ParserContextExtraData");
                return Err(())
            },
        };

        let serialization = Arc::new(url.into_owned());
        let resolved = context.base_url.join(&serialization).ok();
        Ok(SpecifiedUrl {
            original: Some(serialization),
            resolved: resolved,
            extra_data: extra_data,
        })
    }

    /// Get this URL's extra data.
    pub fn extra_data(&self) -> &UrlExtraData {
        &self.extra_data
    }

    /// Returns the resolved url if it was valid.
    pub fn url(&self) -> Option<&ServoUrl> {
        self.resolved.as_ref()
    }

    /// Return the resolved url as string, or the empty string if it's invalid.
    ///
    /// TODO(emilio): Should we return the original one if needed?
    pub fn as_str(&self) -> &str {
        match self.resolved {
            Some(ref url) => url.as_str(),
            None => "",
        }
    }

    /// Little helper for Gecko's ffi.
    #[cfg(feature = "gecko")]
    pub fn as_slice_components(&self) -> Result<(*const u8, usize), (*const u8, usize)> {
        match self.resolved {
            Some(ref url) => Ok((url.as_str().as_ptr(), url.as_str().len())),
            None => {
                let url = self.original.as_ref()
                    .expect("We should always have either the original or the resolved value");
                Err((url.as_str().as_ptr(), url.as_str().len()))
            }
        }
    }

    /// Creates an already specified url value from an already resolved URL
    /// for insertion in the cascade.
    pub fn for_cascade(url: ServoUrl, extra_data: UrlExtraData) -> Self {
        SpecifiedUrl {
            original: None,
            resolved: Some(url),
            extra_data: extra_data,
        }
    }

    /// Gets a new url from a string for unit tests.
    #[cfg(feature = "servo")]
    pub fn new_for_testing(url: &str) -> Self {
        SpecifiedUrl {
            original: Some(Arc::new(url.into())),
            resolved: ServoUrl::parse(url).ok(),
            extra_data: UrlExtraData {}
        }
    }
}

impl PartialEq for SpecifiedUrl {
    fn eq(&self, other: &Self) -> bool {
        // TODO(emilio): maybe we care about equality of the specified values if
        // present? Seems not.
        self.resolved == other.resolved &&
            self.extra_data == other.extra_data
    }
}

impl Eq for SpecifiedUrl {}

impl ToCss for SpecifiedUrl {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("url(\""));
        let string = match self.original {
            Some(ref original) => &**original,
            None => match self.resolved {
                Some(ref url) => url.as_str(),
                // This can only happen if the url wasn't specified by the
                // user *and* it's an invalid url that has been transformed
                // back to specified value via the "uncompute" functionality.
                None => "about:invalid",
            }
        };

        try!(CssStringWriter::new(dest).write_str(string));
        dest.write_str("\")")
    }
}

// TODO(emilio): Maybe consider ComputedUrl to save a word in style structs?
impl ComputedValueAsSpecified for SpecifiedUrl {}

impl NoViewportPercentage for SpecifiedUrl {}
