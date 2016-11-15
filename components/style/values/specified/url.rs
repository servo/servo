/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use cssparser::{CssStringWriter, Parser};
#[cfg(feature = "gecko")]
use gecko_bindings::sugar::refptr::{GeckoArcPrincipal, GeckoArcURI};
use parser::ParserContext;
#[cfg(feature = "gecko")]
use parser::ParserContextExtraData;
use servo_url::ServoUrl;
use std::fmt::{self, Write};
use std::ptr;
use std::sync::Arc;
use style_traits::ToCss;
use values::computed::ComputedValueAsSpecified;

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Serialize, Deserialize, Eq))]
pub struct UrlExtraData {
    #[cfg(feature = "gecko")]
    pub base: GeckoArcURI,
    #[cfg(feature = "gecko")]
    pub referrer: GeckoArcURI,
    #[cfg(feature = "gecko")]
    pub principal: GeckoArcPrincipal,
}

impl UrlExtraData {
    #[cfg(feature = "servo")]
    pub fn make_from(_: &ParserContext) -> Option<UrlExtraData> {
        Some(UrlExtraData { })
    }

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

impl SpecifiedUrl {
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let url = try!(input.expect_url());

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

    pub fn extra_data(&self) -> &UrlExtraData {
        &self.extra_data
    }

    pub fn url(&self) -> Option<&ServoUrl> {
        self.resolved.as_ref()
    }

    pub fn as_str(&self) -> &str {
        match self.resolved {
            Some(ref url) => url.as_str(),
            None => "",
        }
    }

    /// Little helper for Gecko's ffi.
    pub fn as_slice_components(&self) -> (*const u8, usize) {
        match self.resolved {
            Some(ref url) => (url.as_str().as_ptr(), url.as_str().len()),
            None => (ptr::null(), 0),
        }
    }

    /// Creates an already specified url value from an already resolved URL
    /// for insertion in the cascade.
    pub fn for_cascade(url: Option<ServoUrl>, extra_data: UrlExtraData) -> Self {
        SpecifiedUrl {
            original: None,
            resolved: url,
            extra_data: extra_data,
        }
    }

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
