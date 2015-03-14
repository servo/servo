/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::Range;

use ::FromCss;
use ::cssparser::Parser;
use ::values::specified::{Length, Ratio};

// TODO: refactor values.rs to implement FromCss directly
impl FromCss for Length {
    type Err = ();

    #[inline]
    fn from_css(input: &mut Parser) -> Result<Length, ()> {
        Length::parse_non_negative(input)
    }
}

// TODO: refactor values.rs to implement FromCss directly
impl<T> FromCss for T where T: ::std::num::Int {
    type Err = ();

    fn from_css(input: &mut ::cssparser::Parser) -> Result<T, ()> {
        ::std::num::NumCast::from(try!(input.expect_integer())).ok_or(())
    }
}

macro_rules! range_value {
    ($name:ident($value_type:ident)) => {
        #[derive(Copy, Debug, PartialEq)]
        pub struct $name(pub Range<$value_type>);

        impl FromCss for $name {
            type Err = ();

            #[inline]
            fn from_css(input: &mut Parser) -> Result<$name, ()> {
                Ok($name(try!(FromCss::from_css(input))))
            }
        }

        impl $name {
            pub fn to_css<'a, W>(&self, dest: &mut W, name: &'a str) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                self.0.to_css(dest, name)
            }
        }
    }
}

// MQ 4 § 4.1
range_value!(Width(Length));
// MQ 4 § 4.2
range_value!(Height(Length));
// MQ 4 § 4.3
range_value!(AspectRatio(Ratio));

// MQ 4 § 6.1
range_value!(Color(u8));
// MQ 4 § 6.2
range_value!(ColorIndex(u32));
// MQ 4 § 6.3
range_value!(Monochrome(u32));

// MQ 4 § 11.1
range_value!(DeviceWidth(Length));
// MQ 4 § 11.2
range_value!(DeviceHeight(Length));
// MQ 4 § 11.3
range_value!(DeviceAspectRatio(Ratio));
