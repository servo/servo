/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use servo_url::ServoUrl;
use std::fmt::{self, Write};
// Note: We use std::sync::Arc rather than servo_arc::Arc here because the
// nonzero optimization is important in keeping the size of SpecifiedUrl below
// the threshold.
use std::sync::Arc;
use style_traits::{CssWriter, ParseError, ToCss};
use values::computed::{Context, ToComputedValue};

/// A CSS url() value for servo.
///
/// Servo eagerly resolves SpecifiedUrls, which it can then take advantage of
/// when computing values. In contrast, Gecko uses a different URL backend, so
/// eagerly resolving with rust-url would be duplicated work.
///
/// However, this approach is still not necessarily optimal: See
/// <https://bugzilla.mozilla.org/show_bug.cgi?id=1347435#c6>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize, SpecifiedValueInfo)]
pub struct CssUrl {
    /// The original URI. This might be optional since we may insert computed
    /// values of images into the cascade directly, and we don't bother to
    /// convert their serialization.
    ///
    /// Refcounted since cloning this should be cheap and data: uris can be
    /// really large.
    #[ignore_malloc_size_of = "Arc"]
    original: Option<Arc<String>>,

    /// The resolved value for the url, if valid.
    resolved: Option<ServoUrl>,
}

impl CssUrl {
    /// Try to parse a URL from a string value that is a valid CSS token for a
    /// URL.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        let serialization = Arc::new(url);
        let resolved = context.url_data.join(&serialization).ok();
        CssUrl {
            original: Some(serialization),
            resolved: resolved,
        }
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
        CssUrl {
            original: None,
            resolved: Some(url),
        }
    }

    /// Gets a new url from a string for unit tests.
    pub fn new_for_testing(url: &str) -> Self {
        CssUrl {
            original: Some(Arc::new(url.into())),
            resolved: ServoUrl::parse(url).ok(),
        }
    }
}

impl Parse for CssUrl {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let url = input.expect_url()?;
        Ok(Self::parse_from_string(url.as_ref().to_owned(), context))
    }
}

impl PartialEq for CssUrl {
    fn eq(&self, other: &Self) -> bool {
        // TODO(emilio): maybe we care about equality of the specified values if
        // present? Seems not.
        self.resolved == other.resolved
    }
}

impl Eq for CssUrl {}

impl ToCss for CssUrl {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let string = match self.original {
            Some(ref original) => &**original,
            None => match self.resolved {
                Some(ref url) => url.as_str(),
                // This can only happen if the url wasn't specified by the
                // user *and* it's an invalid url that has been transformed
                // back to specified value via the "uncompute" functionality.
                None => "about:invalid",
            },
        };

        dest.write_str("url(")?;
        string.to_css(dest)?;
        dest.write_str(")")
    }
}

/// A specified url() value for servo.
pub type SpecifiedUrl = CssUrl;

impl ToComputedValue for SpecifiedUrl {
    type ComputedValue = ComputedUrl;

    // If we can't resolve the URL from the specified one, we fall back to the original
    // but still return it as a ComputedUrl::Invalid
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        match self.resolved {
            Some(ref url) => ComputedUrl::Valid(url.clone()),
            None => match self.original {
                Some(ref url) => ComputedUrl::Invalid(url.clone()),
                None => {
                    unreachable!("Found specified url with neither resolved or original URI!");
                },
            },
        }
    }

    fn from_computed_value(computed: &ComputedUrl) -> Self {
        match *computed {
            ComputedUrl::Valid(ref url) => SpecifiedUrl {
                original: None,
                resolved: Some(url.clone()),
            },
            ComputedUrl::Invalid(ref url) => SpecifiedUrl {
                original: Some(url.clone()),
                resolved: None,
            },
        }
    }
}

/// A specified image url() value for servo.
pub type SpecifiedImageUrl = CssUrl;

/// The computed value of a CSS `url()`, resolved relative to the stylesheet URL.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ComputedUrl {
    /// The `url()` was invalid or it wasn't specified by the user.
    Invalid(#[ignore_malloc_size_of = "Arc"] Arc<String>),
    /// The resolved `url()` relative to the stylesheet URL.
    Valid(ServoUrl),
}

impl ComputedUrl {
    /// Returns the resolved url if it was valid.
    pub fn url(&self) -> Option<&ServoUrl> {
        match *self {
            ComputedUrl::Valid(ref url) => Some(url),
            _ => None,
        }
    }
}

impl ToCss for ComputedUrl {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let string = match *self {
            ComputedUrl::Valid(ref url) => url.as_str(),
            ComputedUrl::Invalid(ref invalid_string) => invalid_string,
        };

        dest.write_str("url(")?;
        string.to_css(dest)?;
        dest.write_str(")")
    }
}

/// The computed value of a CSS `url()` for image.
pub type ComputedImageUrl = ComputedUrl;
