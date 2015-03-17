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

use ::FromCss;
use ::cssparser::{Parser, ToCss};
use ::geom::size::TypedSize2D;
use ::util::geometry::ViewportPx;

pub use self::feature::{DeviceFeatureContext, MediaFeature};
pub use self::condition::MediaCondition;

pub use self::query::MediaQuery;
// external users should only be able to use the defined media types
pub use self::query::DefinedMediaType as MediaType;

pub trait EvaluateUsingContext<C: DeviceFeatureContext>
{
    fn evaluate(&self, context: &C) -> bool;
}

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
    queries: Vec<MediaQuery>
}

impl<C> EvaluateUsingContext<C> for MediaQueryList
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        self.queries.iter().any(|query| query.evaluate(context))
    }
}

impl FromCss for MediaQueryList {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaQueryList, ()> {
        let queries = if input.is_exhausted() {
            // MQ 4 § 2.1
            // An empty media query list evaluates to true.
            vec![query::ALL_MEDIA_QUERY]
        } else {
            match input.parse_comma_separated(FromCss::from_css) {
                Ok(queries) => queries,
                // MediaQuery::from_css returns `not all` (and consumes any
                // remaining input of the query) on error
                Err(_) => unreachable!()
            }
        };

        Ok(MediaQueryList { queries: queries })
    }
}

impl ToCss for MediaQueryList {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        if !self.queries.is_empty() {
            try!(self.queries[0].to_css(dest));

            for query in &self.queries[1..] {
                try!(write!(dest, ", "));
                try!(query.to_css(dest));
            }
        }
        Ok(())
    }
}

impl MediaQueryList {
    pub fn evaluate(&self, device: &Device) -> bool {
        false
    }
}

pub fn parse_media_query_list(input: &mut Parser) -> MediaQueryList {
    FromCss::from_css(input).unwrap()
}
