/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::root::mozilla::css::URLValue;
use crate::gecko_bindings::structs::root::mozilla::CORSMode;
use crate::gecko_bindings::structs::root::nsStyleImageRequest;
use crate::gecko_bindings::sugar::ownership::{FFIArcHelpers, HasArcFFI};
use crate::gecko_bindings::sugar::refptr::RefPtr;
use crate::parser::{Parse, ParserContext};
use crate::stylesheets::UrlExtraData;
use crate::values::computed::{Context, ToComputedValue};
use cssparser::Parser;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use nsstring::nsCString;
use servo_arc::Arc;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};

/// A CSS url() value for gecko.
#[css(function = "url")]
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss)]
pub struct CssUrl(pub Arc<CssUrlData>);

/// Data shared between CssUrls.
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss)]
pub struct CssUrlData {
    /// The URL in unresolved string form.
    serialization: String,

    /// The URL extra data.
    #[css(skip)]
    pub extra_data: UrlExtraData,
}

impl CssUrl {
    /// Parse a URL from a string value that is a valid CSS token for a URL.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        CssUrl(Arc::new(CssUrlData {
            serialization: url,
            extra_data: context.url_data.clone(),
        }))
    }

    /// Returns true if the URL is definitely invalid. We don't eagerly resolve
    /// URLs in gecko, so we just return false here.
    /// use its |resolved| status.
    pub fn is_invalid(&self) -> bool {
        false
    }

    /// Returns true if this URL looks like a fragment.
    /// See https://drafts.csswg.org/css-values/#local-urls
    #[inline]
    pub fn is_fragment(&self) -> bool {
        self.0.is_fragment()
    }

    /// Return the unresolved url as string, or the empty string if it's
    /// invalid.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl CssUrlData {
    /// Returns true if this URL looks like a fragment.
    /// See https://drafts.csswg.org/css-values/#local-urls
    pub fn is_fragment(&self) -> bool {
        self.as_str().chars().next().map_or(false, |c| c == '#')
    }

    /// Return the unresolved url as string, or the empty string if it's
    /// invalid.
    pub fn as_str(&self) -> &str {
        &*self.serialization
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

impl Eq for CssUrl {}

impl MallocSizeOf for CssUrl {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // XXX: measure `serialization` once bug 1397971 lands

        // We ignore `extra_data`, because RefPtr is tricky, and there aren't
        // many of them in practise (sharing is common).

        0
    }
}

/// A specified non-image `url()` value.
#[derive(Clone, Debug, SpecifiedValueInfo, ToCss)]
pub struct SpecifiedUrl {
    /// The specified url value.
    pub url: CssUrl,
    /// Gecko's URLValue so that we can reuse it while rematching a
    /// property with this specified value.
    #[css(skip)]
    pub url_value: RefPtr<URLValue>,
}

impl SpecifiedUrl {
    /// Parse a URL from a string value.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        Self::from_css_url(CssUrl::parse_from_string(url, context))
    }

    fn from_css_url_with_cors(url: CssUrl, cors: CORSMode) -> Self {
        let url_value = unsafe {
            let ptr = bindings::Gecko_URLValue_Create(url.0.clone().into_strong(), cors);
            // We do not expect Gecko_URLValue_Create returns null.
            debug_assert!(!ptr.is_null());
            RefPtr::from_addrefed(ptr)
        };
        Self { url, url_value }
    }

    fn from_css_url(url: CssUrl) -> Self {
        use crate::gecko_bindings::structs::root::mozilla::CORSMode_CORS_NONE;
        Self::from_css_url_with_cors(url, CORSMode_CORS_NONE)
    }

    fn from_css_url_with_cors_anonymous(url: CssUrl) -> Self {
        use crate::gecko_bindings::structs::root::mozilla::CORSMode_CORS_ANONYMOUS;
        Self::from_css_url_with_cors(url, CORSMode_CORS_ANONYMOUS)
    }
}

impl Parse for SpecifiedUrl {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        CssUrl::parse(context, input).map(Self::from_css_url)
    }
}

impl PartialEq for SpecifiedUrl {
    fn eq(&self, other: &Self) -> bool {
        self.url.eq(&other.url)
    }
}

impl Eq for SpecifiedUrl {}

impl MallocSizeOf for SpecifiedUrl {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.url.size_of(ops);
        // Although this is a RefPtr, this is the primary reference because
        // SpecifiedUrl is responsible for creating the url_value. So we
        // measure unconditionally here.
        n += unsafe { bindings::Gecko_URLValue_SizeOfIncludingThis(self.url_value.get()) };
        n
    }
}

impl ToComputedValue for SpecifiedUrl {
    type ComputedValue = ComputedUrl;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        ComputedUrl(self.clone())
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.0.clone()
    }
}

/// A specified image `url()` value.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
pub struct SpecifiedImageUrl(pub SpecifiedUrl);

impl SpecifiedImageUrl {
    /// Parse a URL from a string value that is a valid CSS token for a URL.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        SpecifiedImageUrl(SpecifiedUrl::parse_from_string(url, context))
    }

    /// Provides an alternate method for parsing that associates the URL
    /// with anonymous CORS headers.
    pub fn parse_with_cors_anonymous<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        CssUrl::parse(context, input)
            .map(SpecifiedUrl::from_css_url_with_cors_anonymous)
            .map(SpecifiedImageUrl)
    }
}

impl Parse for SpecifiedImageUrl {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        SpecifiedUrl::parse(context, input).map(SpecifiedImageUrl)
    }
}

impl ToComputedValue for SpecifiedImageUrl {
    type ComputedValue = ComputedImageUrl;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        ComputedImageUrl(self.clone())
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        computed.0.clone()
    }
}

fn serialize_computed_url<W>(
    url_value: &URLValue,
    dest: &mut CssWriter<W>,
    get_url: unsafe extern "C" fn(*const URLValue, *mut nsCString),
) -> fmt::Result
where
    W: Write,
{
    dest.write_str("url(")?;
    unsafe {
        let mut string = nsCString::new();
        get_url(url_value, &mut string);
        string.as_str_unchecked().to_css(dest)?;
    }
    dest.write_char(')')
}

/// The computed value of a CSS non-image `url()`.
///
/// The only difference between specified and computed URLs is the
/// serialization.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct ComputedUrl(pub SpecifiedUrl);

impl ToCss for ComputedUrl {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        serialize_computed_url(&self.0.url_value, dest, bindings::Gecko_GetComputedURLSpec)
    }
}

impl ComputedUrl {
    /// Convert from RefPtr<URLValue> to ComputedUrl.
    pub unsafe fn from_url_value(url_value: RefPtr<URLValue>) -> Self {
        let css_url = &*url_value.mCssUrl.mRawPtr;
        let url = CssUrl(CssUrlData::as_arc(&css_url).clone_arc());
        ComputedUrl(SpecifiedUrl { url, url_value })
    }

    /// Get a raw pointer to the URLValue held by this ComputedUrl, for FFI.
    pub fn url_value_ptr(&self) -> *mut URLValue {
        self.0.url_value.get()
    }
}

/// The computed value of a CSS image `url()`.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct ComputedImageUrl(pub SpecifiedImageUrl);

impl ToCss for ComputedImageUrl {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        serialize_computed_url(
            &(self.0).0.url_value,
            dest,
            bindings::Gecko_GetComputedImageURLSpec,
        )
    }
}

impl ComputedImageUrl {
    /// Convert from nsStyleImageReques to ComputedImageUrl.
    pub unsafe fn from_image_request(image_request: &nsStyleImageRequest) -> Self {
        let url_value = image_request.mImageValue.to_safe();
        let css_url = &*url_value.mCssUrl.mRawPtr;
        let url = CssUrl(CssUrlData::as_arc(&css_url).clone_arc());
        ComputedImageUrl(SpecifiedImageUrl(SpecifiedUrl { url, url_value }))
    }

    /// Get a raw pointer to the URLValue held by this ComputedImageUrl, for FFI.
    pub fn url_value_ptr(&self) -> *mut URLValue {
        (self.0).0.url_value.get()
    }
}
