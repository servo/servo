/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use cssparser::CssStringWriter;
use gecko_bindings::structs::{ServoBundledURI, URLExtraData};
use gecko_bindings::sugar::refptr::RefPtr;
use parser::ParserContext;
use std::borrow::Cow;
use std::fmt::{self, Write};
use style_traits::ToCss;
use stylearc::Arc;

/// A specified url() value for gecko. Gecko does not eagerly resolve SpecifiedUrls.
#[derive(Clone, Debug, PartialEq)]
pub struct SpecifiedUrl {
    /// The URL in unresolved string form.
    ///
    /// Refcounted since cloning this should be cheap and data: uris can be
    /// really large.
    serialization: Arc<String>,

    /// The URL extra data.
    pub extra_data: RefPtr<URLExtraData>,
}

impl SpecifiedUrl {
    /// Try to parse a URL from a string value that is a valid CSS token for a
    /// URL.
    ///
    /// Returns `Err` in the case that extra_data is incomplete.
    pub fn parse_from_string<'a>(url: Cow<'a, str>,
                                 context: &ParserContext)
                                 -> Result<Self, ()> {
        Ok(SpecifiedUrl {
            serialization: Arc::new(url.into_owned()),
            extra_data: context.url_data.clone(),
        })
    }

    /// Returns true if the URL is definitely invalid. We don't eagerly resolve
    /// URLs in gecko, so we just return false here.
    /// use its |resolved| status.
    pub fn is_invalid(&self) -> bool {
        false
    }

    /// Returns true if this URL looks like a fragment.
    /// See https://drafts.csswg.org/css-values/#local-urls
    pub fn is_fragment(&self) -> bool {
        self.as_str().chars().next().map_or(false, |c| c == '#')
    }

    /// Return the resolved url as string, or the empty string if it's invalid.
    ///
    /// FIXME(bholley): This returns the unresolved URL while the servo version
    /// returns the resolved URL.
    pub fn as_str(&self) -> &str {
        &*self.serialization
    }

    /// Little helper for Gecko's ffi.
    pub fn as_slice_components(&self) -> (*const u8, usize) {
        (self.serialization.as_str().as_ptr(), self.serialization.as_str().len())
    }

    /// Create a bundled URI suitable for sending to Gecko
    /// to be constructed into a css::URLValue
    pub fn for_ffi(&self) -> ServoBundledURI {
        let (ptr, len) = self.as_slice_components();
        ServoBundledURI {
            mURLString: ptr,
            mURLStringLength: len as u32,
            mExtraData: self.extra_data.get(),
        }
    }
}

impl ToCss for SpecifiedUrl {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("url(\""));
        try!(CssStringWriter::new(dest).write_str(&*self.serialization));
        dest.write_str("\")")
    }
}
