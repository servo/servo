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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecificationLevel {
    MEDIAQ3,
    MEDIAQ4
}

impl SpecificationLevel {
    fn from_opts() -> SpecificationLevel {
        if ::util::opts::experimental_enabled() {
            SpecificationLevel::MEDIAQ4
        } else {
            SpecificationLevel::MEDIAQ3
        }
    }
}

#[macro_use]
mod util;
mod feature;
mod condition;
mod query;
mod device;

use ::FromCss;
use ::cssparser::Parser;

pub use self::feature::DeviceFeatureContext;
pub use self::query::{MediaQuery, MediaQueryList};
// external users should only be able to use the known media types
pub use self::query::KnownMediaType as MediaType;
pub use self::device::Device;

pub trait EvaluateUsingContext<C: DeviceFeatureContext>
{
    fn evaluate(&self, context: &C) -> bool;
}

pub fn parse_media_query_list(input: &mut Parser) -> MediaQueryList {
    FromCss::from_css(input, &SpecificationLevel::from_opts()).unwrap()
}

#[cfg(test)]
mod tests;
