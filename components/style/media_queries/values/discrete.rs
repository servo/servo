/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ::FromCss;
use ::cssparser::{Parser, ToCss};

macro_rules! discrete_values {
    ($name:ident { $($css:expr => $variant:ident),+ }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, FromPrimitive)]
        pub enum $name {
            $($variant),+
        }

        derive_display_using_to_css!($name);

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
    };
}

// MQ 4 § 4.4
discrete_values!(Orientation {
    "portrait" => Portrait,
    "landscape" => Landscape
});

// MQ 4 § 4.3
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Grid(pub bool);

derive_display_using_to_css!(Grid);

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
discrete_values!(AnyPointer {
    "none" => None,
    "coarse" => Coarse,
    "fine" => Fine
});

// MQ 4 § 7.4
discrete_values!(AnyHover {
    "none" => None,
    "on-demand" => OnDemand,
    "hover" => Hover
});

// MQ 4 § 8.1
discrete_values!(LightLevel {
    "dim" => Dim,
    "normal" => Normal,
    "washed" => Washed
});

// MQ 4 § 9.1
discrete_values!(Scripting {
    "none" => None,
    "initial-only" => InitialOnly,
    "enabled" => Enabled
});
