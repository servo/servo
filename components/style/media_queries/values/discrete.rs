/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::super::MediaType;
use super::{DeviceFeatureContext, EvaluateMediaFeatureValue};

use ::FromCss;
use ::cssparser::{Parser, ToCss};
use ::geom::size::Size2D;
use ::util::geometry::Au;

macro_rules! discrete_values {
    ($name:ident { $($css:expr => $variant:ident),+ }) => {
        discrete_values!($name { $($css => $variant),+ },
                         context -> $name,
                         match context {
                             boolean(context_value) => context_value != $name::None,
                             normal(ref feature_value, context_value) => *feature_value == context_value
                         });
    };

    ($name:ident { $($css:expr => $variant:ident),+ },
     match $context:ident {
         boolean => $boolean_context:expr
    }) => {
        discrete_values!($name { $($css => $variant),+ },
                         context -> $name,
                         match $context {
                             boolean(unused) => $boolean_context,
                             normal(ref feature_value, context_value) => *feature_value == context_value
                         });
    };

    ($name:ident { $($css:expr => $variant:ident),+ },
     context -> $context_type:ty,
     match $context:ident {
         boolean($boolean_context_value:ident) => $boolean_context:expr,
         normal(ref $feature_value:ident, $normal_context_value:ident) => $normal_context:expr
    }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, FromPrimitive)]
        pub enum $name {
            $($variant),+
        }

        derive_display_using_to_css!($name);

        impl<C> EvaluateMediaFeatureValue<C> for Option<$name>
            where C: DeviceFeatureContext
        {
            type Context = $context_type;

            fn evaluate(&self, $context: &C, $boolean_context_value: $context_type) -> bool {
                match *self {
                    Some(ref feature_value) => feature_value.evaluate($context, $boolean_context_value),
                    None => $boolean_context,
                }
            }
        }

        impl<C> EvaluateMediaFeatureValue<C> for $name
            where C: DeviceFeatureContext
        {
            type Context = $context_type;

            #[allow(unused_variables)]
            #[inline]
            fn evaluate(&self, $context: &C, $normal_context_value: $context_type) -> bool {
                let $feature_value = self;
                $normal_context
            }
        }

        impl FromCss for $name {
            type Err = ();

            fn from_css(input: &mut Parser) -> Result<$name, ()> {
                use ::std::ascii::AsciiExt;

                match &try!(input.expect_ident())[] {
                    $(s if s.eq_ignore_ascii_case($css) => Ok($name::$variant)),+,
                    _ => Err(())
                }
            }
        }

        impl ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                match self {
                    $(&$name::$variant => dest.write_str($css)),+
                }
            }
        }
    }
}

// MQ 4 § 4.4
discrete_values!(Orientation {
    "portrait" => Portrait,
    "landscape" => Landscape
}, match context {
    boolean => context.MediaType() != MediaType::Speech
});

impl Orientation {
    pub fn from_viewport_size(viewport_size: Size2D<Au>) -> Orientation {
        if viewport_size.height >= viewport_size.width {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }
}

// MQ 4 § 4.3
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Grid(pub bool);

derive_display_using_to_css!(Grid);

impl<C> EvaluateMediaFeatureValue<C> for Option<Grid>
    where C: DeviceFeatureContext
{
    type Context = bool;

    #[inline]
    fn evaluate(&self, _: &C, context_value: bool) -> bool {
        match *self {
            Some(ref feature_value) => feature_value.0 == context_value,
            None => context_value,
        }
    }
}

impl<C> EvaluateMediaFeatureValue<C> for Grid
    where C: DeviceFeatureContext
{
    type Context = bool;

    #[inline]
    fn evaluate(&self, _: &C, context_value: bool) -> bool {
        self.0 == context_value
    }
}

impl FromCss for Grid {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<Grid, ()> {
        match try!(input.expect_integer()) {
            0 => Ok(Grid(false)),
            1 => Ok(Grid(true)),
            _ => Err(())
        }
    }
}

impl ToCss for Grid {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match self.0 {
            false => dest.write_str("0"),
            true => dest.write_str("1"),
        }
    }
}

// MQ 4 § 4.4
discrete_values!(Scan {
    "interlace" => Interlace,
    "progressive" => Progressive
},
context -> Option<Scan>,
match context {
    boolean(scan) => context.MediaType() == MediaType::Screen,
    normal(ref specified, scan) =>
        scan.map_or(false, |scan| *specified == scan)
});

// MQ 4 § 5.4
discrete_values!(UpdateFrequency {
    "none" => None,
    "slow" => Slow,
    "normal" => Normal
});

// MQ 4 § 5.5
discrete_values!(OverflowBlock {
    "none" => None,
    "scroll" => Scroll,
    "optional-paged" => OptionalPaged,
    "paged" => Paged
});

// MQ 4 § 5.6
discrete_values!(OverflowInline {
    "none" => None,
    "scroll" => Scroll
});

// MQ 4 § 6.4
discrete_values!(InvertedColors {
    "none" => None,
    "inverted" => Inverted
});

// MQ 4 § 7.1
discrete_values!(Pointer {
    "none" => None,
    "coarse" => Coarse,
    "fine" => Fine
});

// MQ 4 § 7.2
discrete_values!(Hover {
    "none" => None,
    "on-demand" => OnDemand,
    "hover" => Hover
});

// MQ 4 § 7.3
pub type AnyPointer = Pointer;

// MQ 4 § 7.4
pub type AnyHover = Hover;

// MQ 4 § 8.1
discrete_values!(LightLevel {
    "dim" => Dim,
    "normal" => Normal,
    "washed" => Washed
},
context -> Option<LightLevel>,
match context {
    boolean(light_level) => light_level.is_some(),
    normal(ref specified, light_level) =>
        light_level.map_or(false, |light_level| *specified == light_level)
});

// MQ 4 § 9.1
discrete_values!(Scripting {
    "none" => None,
    "initial-only" => InitialOnly,
    "enabled" => Enabled
});
