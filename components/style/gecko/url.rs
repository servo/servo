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
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::mem::ManuallyDrop;
use std::sync::RwLock;
use style_traits::{CssWriter, ParseError, ToCss};
use to_shmem::{SharedMemoryBuilder, ToShmem};

/// A CSS url() value for gecko.
#[css(function = "url")]
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct CssUrl(pub Arc<CssUrlData>);

/// Data shared between CssUrls.
#[derive(Clone, Debug, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
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

#[cfg(debug_assertions)]
impl Drop for CssUrlData {
    fn drop(&mut self) {
        assert!(
            !URL_VALUE_TABLE
                .read()
                .unwrap()
                .contains_key(&CssUrlDataKey(self as *mut _ as *const _)),
            "All CssUrlData objects used as keys in URL_VALUE_TABLE should be \
             from shared memory style sheets, and so should never be dropped",
        );
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

/// A key type for URL_VALUE_TABLE.
#[derive(Eq, Hash, PartialEq)]
struct CssUrlDataKey(*const CssUrlData);

unsafe impl Sync for CssUrlDataKey {}
unsafe impl Send for CssUrlDataKey {}

/// The source of a Gecko URLValue object for a SpecifiedUrl.
#[derive(Clone, Debug)]
pub enum URLValueSource {
    /// A strong reference to a Gecko URLValue object.
    URLValue(RefPtr<URLValue>),
    /// A CORSMode value used to lazily construct a Gecko URLValue object.
    ///
    /// The lazily created object will be stored in URL_VALUE_TABLE.
    CORSMode(CORSMode),
}

impl ToShmem for URLValueSource {
    fn to_shmem(&self, _builder: &mut SharedMemoryBuilder) -> ManuallyDrop<Self> {
        ManuallyDrop::new(match self {
            URLValueSource::URLValue(r) => URLValueSource::CORSMode(r.mCORSMode),
            URLValueSource::CORSMode(c) => URLValueSource::CORSMode(*c),
        })
    }
}

/// A specified non-image `url()` value.
#[derive(Clone, Debug, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct SpecifiedUrl {
    /// The specified url value.
    pub url: CssUrl,
    /// Gecko's URLValue so that we can reuse it while rematching a
    /// property with this specified value.
    ///
    /// Box this to avoid SpecifiedUrl getting bigger than two words,
    /// and increasing the size of PropertyDeclaration.
    #[css(skip)]
    url_value: Box<URLValueSource>,
}

fn make_url_value(url: &CssUrl, cors_mode: CORSMode) -> RefPtr<URLValue> {
    unsafe {
        let ptr = bindings::Gecko_URLValue_Create(url.0.clone().into_strong(), cors_mode);
        // We do not expect Gecko_URLValue_Create returns null.
        debug_assert!(!ptr.is_null());
        RefPtr::from_addrefed(ptr)
    }
}

impl SpecifiedUrl {
    /// Parse a URL from a string value.
    pub fn parse_from_string(url: String, context: &ParserContext) -> Self {
        Self::from_css_url(CssUrl::parse_from_string(url, context))
    }

    fn from_css_url_with_cors(url: CssUrl, cors: CORSMode) -> Self {
        let url_value = Box::new(URLValueSource::URLValue(make_url_value(&url, cors)));
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

    fn with_url_value<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&RefPtr<URLValue>) -> T,
    {
        match *self.url_value {
            URLValueSource::URLValue(ref r) => f(r),
            URLValueSource::CORSMode(cors_mode) => {
                {
                    let guard = URL_VALUE_TABLE.read().unwrap();
                    if let Some(r) = guard.get(&(CssUrlDataKey(&*self.url.0 as *const _))) {
                        return f(r);
                    }
                }
                let mut guard = URL_VALUE_TABLE.write().unwrap();
                let r = guard
                    .entry(CssUrlDataKey(&*self.url.0 as *const _))
                    .or_insert_with(|| make_url_value(&self.url, cors_mode));
                f(r)
            },
        }
    }

    /// Clone a new, strong reference to the Gecko URLValue.
    pub fn clone_url_value(&self) -> RefPtr<URLValue> {
        self.with_url_value(RefPtr::clone)
    }

    /// Get a raw pointer to the URLValue held by this SpecifiedUrl, for FFI.
    pub fn url_value_ptr(&self) -> *mut URLValue {
        self.with_url_value(RefPtr::get)
    }
}

/// Clears URL_VALUE_TABLE.  Entries in this table, which are for specified URL
/// values that come from shared memory style sheets, would otherwise persist
/// until the end of the process and be reported as leaks.
pub fn shutdown() {
    URL_VALUE_TABLE.write().unwrap().clear();
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
        n += unsafe { bindings::Gecko_URLValue_SizeOfIncludingThis(self.url_value_ptr()) };
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
#[derive(Clone, Debug, Eq, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
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
        self.0
            .with_url_value(|r| serialize_computed_url(r, dest, bindings::Gecko_GetComputedURLSpec))
    }
}

impl ComputedUrl {
    /// Convert from RefPtr<URLValue> to ComputedUrl.
    pub unsafe fn from_url_value(url_value: RefPtr<URLValue>) -> Self {
        let css_url = &*url_value.mCssUrl.mRawPtr;
        let url = CssUrl(CssUrlData::as_arc(&css_url).clone_arc());
        ComputedUrl(SpecifiedUrl {
            url,
            url_value: Box::new(URLValueSource::URLValue(url_value)),
        })
    }

    /// Clone a new, strong reference to the Gecko URLValue.
    pub fn clone_url_value(&self) -> RefPtr<URLValue> {
        self.0.clone_url_value()
    }

    /// Get a raw pointer to the URLValue held by this ComputedUrl, for FFI.
    pub fn url_value_ptr(&self) -> *mut URLValue {
        self.0.url_value_ptr()
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
        (self.0).0.with_url_value(|r| {
            serialize_computed_url(r, dest, bindings::Gecko_GetComputedImageURLSpec)
        })
    }
}

impl ComputedImageUrl {
    /// Convert from nsStyleImageReques to ComputedImageUrl.
    pub unsafe fn from_image_request(image_request: &nsStyleImageRequest) -> Self {
        let url_value = image_request.mImageValue.to_safe();
        let css_url = &*url_value.mCssUrl.mRawPtr;
        let url = CssUrl(CssUrlData::as_arc(&css_url).clone_arc());
        ComputedImageUrl(SpecifiedImageUrl(SpecifiedUrl {
            url,
            url_value: Box::new(URLValueSource::URLValue(url_value)),
        }))
    }

    /// Clone a new, strong reference to the Gecko URLValue.
    pub fn clone_url_value(&self) -> RefPtr<URLValue> {
        (self.0).0.clone_url_value()
    }

    /// Get a raw pointer to the URLValue held by this ComputedImageUrl, for FFI.
    pub fn url_value_ptr(&self) -> *mut URLValue {
        (self.0).0.url_value_ptr()
    }
}

lazy_static! {
    /// A table mapping CssUrlData objects to their lazily created Gecko
    /// URLValue objects.
    static ref URL_VALUE_TABLE: RwLock<HashMap<CssUrlDataKey, RefPtr<URLValue>>> = {
        Default::default()
    };
}
