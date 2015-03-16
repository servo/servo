/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

macro_rules! derive_display_using_to_css {
    ($item_:ty) => {
        impl ::std::fmt::Display for $item_ {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                use ::cssparser::ToCss;

                self.fmt_to_css(f)
            }
        }
    };
}

pub mod values;
mod feature;
mod condition;
mod query;

use ::cssparser::Parser;
use ::geom::size::TypedSize2D;
use ::util::geometry::ViewportPx;

pub use self::feature::MediaFeature;
pub use self::condition::MediaCondition;
pub use self::query::{MediaType, MediaQuery};

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Device {
    pub media_type: MediaType,
    pub viewport_size: TypedSize2D<ViewportPx, f32>,
}

impl Device {
    pub fn new(media_type: MediaType, viewport_size: TypedSize2D<ViewportPx, f32>) -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MediaQueryList {
    media_queries: Vec<MediaQuery>
}

impl MediaQueryList {
    pub fn evaluate(&self, device: &Device) -> bool {
        false
    }
}

pub fn parse_media_query_list(input: &mut Parser) -> MediaQueryList {
    MediaQueryList { media_queries: vec![] }
}
