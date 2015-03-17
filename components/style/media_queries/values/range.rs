/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{DeviceFeatureContext, EvaluateMediaFeatureValue, Range};

use ::FromCss;
use ::cssparser::Parser;
use ::util::geometry::Au;
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
    ($name:ident($value_type:ty), zero = $zero:expr) => {
        range_value!($name($value_type),
                     context -> $value_type,
                     zero = $zero);
    };
    ($name:ident($value_type:ty),
     context -> $context_type:ty,
     zero = $zero:expr) => {
        def_range_value_type!($name($value_type),
                              context -> $context_type,
                              zero = $zero);
        impl_evaluate_media_feature_value!($name, $context_type);
    };
    ($name:ident($value_type:ty),
     context -> $context_type:ty,
     zero = $zero:expr,
     compute = $compute_value_fn:ident) => {
        def_range_value_type!($name($value_type),
                              context -> $context_type,
                              zero = $zero);
        impl_evaluate_media_feature_value!($name, $context_type, $compute_value_fn);
    }
}

macro_rules! def_range_value_type {
    ($name:ident($value_type:ty),
     context -> $context_type:ty,
     zero = $zero:expr) => {
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

        impl<C> EvaluateMediaFeatureValue<C> for Option<$name>
            where C: DeviceFeatureContext
        {
            type Context = $context_type;

            fn evaluate(&self, context: &C, context_value: $context_type) -> bool {
                match *self {
                    Some(ref range) => range.evaluate(context, context_value),
                    None => context_value != $zero
                }
            }
        }
    }
}

macro_rules! impl_evaluate_media_feature_value {
    ($name:ty, $context_type:ty) => {
        impl<C> EvaluateMediaFeatureValue<C> for $name
            where C: DeviceFeatureContext
        {
            type Context = $context_type;

            #[inline]
            fn evaluate(&self, _: &C, context_value: $context_type) -> bool {
                self.0.evaluate(context_value)
            }
        }
    };
    ($name:ty, $context_type:ty, $compute_value_fn:ident) => {
        impl<C> EvaluateMediaFeatureValue<C> for $name
            where C: DeviceFeatureContext
        {
            type Context = $context_type;

            #[inline]
            fn evaluate(&self, context: &C, context_value: $context_type) -> bool {
                self.0
                    .map(|value| $compute_value_fn(value, context))
                    .evaluate(context_value)
            }
        }
    }
}

fn compute_length<C>(specified: &Length, context: &C) -> Au
    where C: DeviceFeatureContext
{
    use ::properties::longhands::font_size;

    match *specified {
        Length::Absolute(value) => value,
        Length::FontRelative(value) => {
            // MQ 4 § 1.3
            // Relative units in media queries are based on the initial value,
            // which means that units are never based on results of declarations.
            let initial_font_size = font_size::get_initial_value();
            value.to_computed_value(initial_font_size, initial_font_size)
        }
        Length::ViewportPercentage(value) =>
            value.to_computed_value(context.ViewportSize()),
        _ => unreachable!()
    }
}

// MQ 4 § 4.1
range_value!(Width(Length),
             context -> Au,
             zero = Au(0),
             compute = compute_length);
// MQ 4 § 4.2
range_value!(Height(Length),
             context -> Au,
             zero = Au(0),
             compute = compute_length);
// MQ 4 § 4.3
range_value!(AspectRatio(Ratio),
             context -> f32,
             zero = 0.);

// MQ 4 § 6.1
range_value!(Color(u8), zero = 0);
// MQ 4 § 6.2
range_value!(ColorIndex(u32), zero = 0);
// MQ 4 § 6.3
range_value!(Monochrome(u32), zero = 0);

// MQ 4 § 11.1
range_value!(DeviceWidth(Length),
             context -> Au,
             zero = Au(0),
             compute = compute_length);
// MQ 4 § 11.2
range_value!(DeviceHeight(Length),
             context -> Au,
             zero = Au(0),
             compute = compute_length);
// MQ 4 § 11.3
range_value!(DeviceAspectRatio(Ratio),
             context -> f32,
             zero = 0.);
