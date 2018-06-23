/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use cssparser::Parser;
use gecko_bindings::bindings;
use gecko_bindings::structs::ServoBundledURI;
use gecko_bindings::structs::mozilla::css::URLValueData;
use gecko_bindings::structs::root::{RustString, nsStyleImageRequest};
use gecko_bindings::structs::root::mozilla::css::{ImageValue, URLValue};
use gecko_bindings::sugar::refptr::RefPtr;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use nsstring::nsCString;
use parser::{Parse, ParserContext};
use servo_arc::{Arc, RawOffsetArc};
use std::fmt::{self, Write};
use std::mem;
use style_traits::{CssWriter, ParseError, ToCss};
use stylesheets::UrlExtraData;
use values::computed::{Context, ToComputedValue};

/// A CSS url() value for gecko.
#[css(function = "url")]
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss)]
pub struct CssUrl {
    /// The URL in unresolved string form.
    ///
    /// Refcounted since cloning this should be cheap and data: uris can be
    /// really large.
    serialization: Arc<String>,

    /// The URL extra data.
    #[css(skip)]
    pub extra_data: UrlExtraData,
}

impl CssUrl {
    /// Try to parse a URL from a string value that is a valid CSS token for a
    /// URL.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        CssUrl {
            serialization: Arc::new(url),
            extra_data: context.url_data.clone(),
        }
    }

    /// Returns true if the URL is definitely invalid. We don't eagerly resolve
    /// URLs in gecko, so we just return false here.
    /// use its |resolved| status.
    pub fn is_invalid(&self) -> bool {
        false
    }

    /// Convert from URLValueData to SpecifiedUrl.
    unsafe fn from_url_value_data(url: &URLValueData) -> Self {
        let arc_type =
            &url.mString as *const _ as *const RawOffsetArc<String>;
        CssUrl {
            serialization: Arc::from_raw_offset((*arc_type).clone()),
            extra_data: UrlExtraData(url.mExtraData.to_safe()),
        }
    }

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

    /// Little helper for Gecko's ffi.
    pub fn as_slice_components(&self) -> (*const u8, usize) {
        (
            self.serialization.as_str().as_ptr(),
            self.serialization.as_str().len(),
        )
    }

    /// Create a bundled URI suitable for sending to Gecko
    /// to be constructed into a css::URLValue
    pub fn for_ffi(&self) -> ServoBundledURI {
        let arc_offset = Arc::into_raw_offset(self.serialization.clone());
        ServoBundledURI {
            mURLString: unsafe { mem::transmute::<_, RawOffsetArc<RustString>>(arc_offset) },
            mExtraData: self.extra_data.0.get(),
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

impl Eq for CssUrl {}

impl MallocSizeOf for CssUrl {
    fn size_of(&self, _ops: &mut MallocSizeOfOps) -> usize {
        // XXX: measure `serialization` once bug 1397971 lands

        // We ignore `extra_data`, because RefPtr is tricky, and there aren't
        // many of them in practise (sharing is common).

        0
    }
}

/// A specified url() value for general usage.
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
    fn from_css_url(url: CssUrl) -> Self {
        let url_value = unsafe {
            let ptr = bindings::Gecko_NewURLValue(url.for_ffi());
            // We do not expect Gecko_NewURLValue returns null.
            debug_assert!(!ptr.is_null());
            RefPtr::from_addrefed(ptr)
        };
        Self { url, url_value }
    }
}


impl PartialEq for SpecifiedUrl {
    fn eq(&self, other: &Self) -> bool {
        self.url.eq(&other.url)
    }
}

impl Eq for SpecifiedUrl {}

impl Parse for SpecifiedUrl {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        CssUrl::parse(context, input).map(Self::from_css_url)
    }
}

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

/// A specified url() value for image.
///
/// This exists so that we can construct `ImageValue` and reuse it.
#[derive(Clone, Debug, SpecifiedValueInfo, ToCss)]
pub struct SpecifiedImageUrl {
    /// The specified url value.
    pub url: CssUrl,
    /// Gecko's ImageValue so that we can reuse it while rematching a
    /// property with this specified value.
    #[css(skip)]
    pub image_value: RefPtr<ImageValue>,
}

impl SpecifiedImageUrl {
    /// Parse a URL from a string value. See SpecifiedUrl::parse_from_string.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        Self::from_css_url(CssUrl::parse_from_string(url, context))
    }

    fn from_css_url(url: CssUrl) -> Self {
        let image_value = unsafe {
            let ptr = bindings::Gecko_ImageValue_Create(url.for_ffi());
            // We do not expect Gecko_ImageValue_Create returns null.
            debug_assert!(!ptr.is_null());
            RefPtr::from_addrefed(ptr)
        };
        Self { url, image_value }
    }
}

impl Parse for SpecifiedImageUrl {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        CssUrl::parse(context, input).map(Self::from_css_url)
    }
}

impl PartialEq for SpecifiedImageUrl {
    fn eq(&self, other: &Self) -> bool {
        self.url.eq(&other.url)
    }
}

impl Eq for SpecifiedImageUrl {}

impl MallocSizeOf for SpecifiedImageUrl {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        let mut n = self.url.size_of(ops);
        // Although this is a RefPtr, this is the primary reference because
        // SpecifiedUrl is responsible for creating the image_value. So we
        // measure unconditionally here.
        n += unsafe { bindings::Gecko_ImageValue_SizeOfIncludingThis(self.image_value.get()) };
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
    url_value_data: &URLValueData,
    dest: &mut CssWriter<W>,
) -> fmt::Result
where
    W: Write,
{
    dest.write_str("url(")?;
    unsafe {
        let mut string = nsCString::new();
        bindings::Gecko_GetComputedURLSpec(url_value_data, &mut string);
        string.as_str_unchecked().to_css(dest)?;
    }
    dest.write_char(')')
}

/// The computed value of a CSS `url()`.
///
/// The only difference between specified and computed URLs is the
/// serialization.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct ComputedUrl(pub SpecifiedUrl);

impl ToCss for ComputedUrl {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write
    {
        serialize_computed_url(&self.0.url_value._base, dest)
    }
}

impl ComputedUrl {
    /// Convert from RefPtr<URLValue> to ComputedUrl.
    pub unsafe fn from_url_value(url_value: RefPtr<URLValue>) -> Self {
        let url = CssUrl::from_url_value_data(&url_value._base);
        ComputedUrl(SpecifiedUrl { url, url_value })
    }
}

/// The computed value of a CSS `url()` for image.
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq)]
pub struct ComputedImageUrl(pub SpecifiedImageUrl);

impl ToCss for ComputedImageUrl {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write
    {
        serialize_computed_url(&self.0.image_value._base, dest)
    }
}

impl ComputedImageUrl {
    /// Convert from nsStyleImageReques to ComputedImageUrl.
    pub unsafe fn from_image_request(image_request: &nsStyleImageRequest) -> Self {
        let image_value = image_request.mImageValue.to_safe();
        let url = CssUrl::from_url_value_data(&image_value._base);
        ComputedImageUrl(SpecifiedImageUrl { url, image_value })
    }
}
