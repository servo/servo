/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common handling for the specified value CSS url() values.

use gecko_bindings::structs::{ServoBundledURI, URLExtraData};
use gecko_bindings::structs::mozilla::css::URLValueData;
use gecko_bindings::structs::root::mozilla::css::ImageValue;
use gecko_bindings::structs::root::nsStyleImageRequest;
use gecko_bindings::sugar::refptr::RefPtr;
use parser::ParserContext;
use servo_arc::Arc;
use std::fmt;
use style_traits::{ToCss, ParseError};

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

    /// Cache ImageValue, if any, so that we can reuse it while rematching a
    /// a property with this specified url value.
    pub image_value: Option<RefPtr<ImageValue>>,
}

impl SpecifiedUrl {
    /// Try to parse a URL from a string value that is a valid CSS token for a
    /// URL.
    ///
    /// Returns `Err` in the case that extra_data is incomplete.
    pub fn parse_from_string<'a>(url: String,
                                 context: &ParserContext)
                                 -> Result<Self, ParseError<'a>> {
        Ok(SpecifiedUrl {
            serialization: Arc::new(url),
            extra_data: context.url_data.clone(),
            image_value: None,
        })
    }

    /// Returns true if the URL is definitely invalid. We don't eagerly resolve
    /// URLs in gecko, so we just return false here.
    /// use its |resolved| status.
    pub fn is_invalid(&self) -> bool {
        false
    }

    /// Convert from URLValueData to SpecifiedUrl.
    pub unsafe fn from_url_value_data(url: &URLValueData)
                                       -> Result<SpecifiedUrl, ()> {
        Ok(SpecifiedUrl {
            serialization: Arc::new(url.mString.to_string()),
            extra_data: url.mExtraData.to_safe(),
            image_value: None,
        })
    }

    /// Convert from nsStyleImageRequest to SpecifiedUrl.
    pub unsafe fn from_image_request(image_request: &nsStyleImageRequest) -> Result<SpecifiedUrl, ()> {
        if image_request.mImageValue.mRawPtr.is_null() {
            return Err(());
        }

        let image_value = image_request.mImageValue.mRawPtr.as_ref().unwrap();
        let ref url_value_data = image_value._base;
        let mut result = try!(Self::from_url_value_data(url_value_data));
        result.build_image_value();
        Ok(result)
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

    /// Build and carry an image value on request.
    pub fn build_image_value(&mut self) {
        use gecko_bindings::bindings::Gecko_ImageValue_Create;

        debug_assert_eq!(self.image_value, None);
        self.image_value = {
            unsafe {
                let ptr = Gecko_ImageValue_Create(self.for_ffi());
                // We do not expect Gecko_ImageValue_Create returns null.
                debug_assert!(!ptr.is_null());
                Some(RefPtr::from_addrefed(ptr))
            }
        }
    }
}

impl ToCss for SpecifiedUrl {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("url(")?;
        self.serialization.to_css(dest)?;
        dest.write_str(")")
    }
}
