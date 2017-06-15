/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use cssparser::CssStringWriter;
use parser::ParserContext;
use servo_url::ServoUrl;
use std::fmt::{self, Write};
// Note: We use std::sync::Arc rather than stylearc::Arc here because the
// nonzero optimization is important in keeping the size of SpecifiedUrl below
// the threshold.
use std::sync::Arc;
use style_traits::{ToCss, ParseError};

/// A specified url() value for servo.
///
/// Servo eagerly resolves SpecifiedUrls, which it can then take advantage of
/// when computing values. In contrast, Gecko uses a different URL backend, so
/// eagerly resolving with rust-url would be duplicated work.
///
/// However, this approach is still not necessarily optimal: See
/// https://bugzilla.mozilla.org/show_bug.cgi?id=1347435#c6
#[derive(Clone, Debug, HeapSizeOf, Serialize, Deserialize)]
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
}

impl SpecifiedUrl {
    /// Try to parse a URL from a string value that is a valid CSS token for a
    /// URL. Never fails - the API is only fallible to be compatible with the
    /// gecko version.
    pub fn parse_from_string<'a>(url: String,
                                 context: &ParserContext)
                                 -> Result<Self, ParseError<'a>> {
        let serialization = Arc::new(url);
        let resolved = context.url_data.join(&serialization).ok();
        Ok(SpecifiedUrl {
            original: Some(serialization),
            resolved: resolved,
        })
    }

    /// Returns true if the URL is definitely invalid. For Servo URLs, we can
    /// use its |resolved| status.
    pub fn is_invalid(&self) -> bool {
        self.resolved.is_none()
    }

    /// Returns true if this URL looks like a fragment.
    /// See https://drafts.csswg.org/css-values/#local-urls
    ///
    /// Since Servo currently stores resolved URLs, this is hard to implement. We
    /// either need to change servo to lazily resolve (like Gecko), or note this
    /// information in the tokenizer.
    pub fn is_fragment(&self) -> bool {
        error!("Can't determine whether the url is a fragment.");
        false
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

    /// Creates an already specified url value from an already resolved URL
    /// for insertion in the cascade.
    pub fn for_cascade(url: ServoUrl) -> Self {
        SpecifiedUrl {
            original: None,
            resolved: Some(url),
        }
    }

    /// Gets a new url from a string for unit tests.
    pub fn new_for_testing(url: &str) -> Self {
        SpecifiedUrl {
            original: Some(Arc::new(url.into())),
            resolved: ServoUrl::parse(url).ok(),
        }
    }
}

impl PartialEq for SpecifiedUrl {
    fn eq(&self, other: &Self) -> bool {
        // TODO(emilio): maybe we care about equality of the specified values if
        // present? Seems not.
        self.resolved == other.resolved
    }
}

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
