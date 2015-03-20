/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
mod macros;
mod range;

use ::FromCss;
use super::EvaluateUsingContext;
pub use self::range::Range;

use ::cssparser::{Parser, SourcePosition, ToCss, Token};
use ::geom::size::Size2D;
use ::util::geometry::Au;
use ::values::specified::{Length, Ratio};

use std::ascii::AsciiExt;
use std::borrow::Cow;

media_features! {
    // MQ 4 § 4. Screen/Device Dimensions
    // MQ 4 § 4.1
    Width(name: "width",
          value: Length,
          type: range,
          availability: {
              since: SpecificationLevel::MEDIAQ3
          },
          impl: {
              context: Au,
              zero: Au(0),
              compute: compute_length
          }),

    // MQ 4 § 4.2
    Height(name: "height",
           value: Length,
           type: range,
           availability: {
               since: SpecificationLevel::MEDIAQ3
           },
           impl: {
               context: Au,
               zero: Au(0),
               compute: compute_length
           }),

    // MQ 4 § 4.3
    AspectRatio(name: "aspect-ratio",
                value: Ratio,
                type: range,
                availability: {
                    since: SpecificationLevel::MEDIAQ3
                },
                impl: {
                    context: f32,
                    zero: 0.
                }),

    // MQ 4 § 4.5
    Orientation(name: "orientation",
                value: {
                    "portrait" => Portrait,
                    "landscape" => Landscape
                },
                type: discrete,
                availability: {
                    since: SpecificationLevel::MEDIAQ3
                },
                impl: {
                    no_none
                }),

    // MQ 4 § 5. Display Quality
    // MQ 4 § 5.1
    //Resolution(name: "resolution",
    //           value: Resolution,
    //           type: range,
    //           availability: {
    //               since: SpecificationLevel::MEDIAQ3
    //           },
    //           impl: {
    //               context: Resolution,
    //               zero: 0.,
    //               compute: compute_resolution
    //           }),

    // MQ 4 § 5.2
    Scan(name: "scan",
         value: {
             "interlace" => Interlace,
             "progressive" => Progressive
         },
         type: discrete,
         availability: {
             since: SpecificationLevel::MEDIAQ3
         },
         impl: {
             no_none
         }),

    // MQ 4 § 5.3
    Grid(name: "grid",
         value: mq_boolean,
         type: discrete,
         availability: {
             since: SpecificationLevel::MEDIAQ3
         }),

    // MQ 4 § 5.4
    UpdateFrequency(name: "update-frequency",
                    value: {
                        "none" => None,
                        "slow" => Slow,
                        "normal" => Normal
                    },
                    type: discrete,
                    availability: {
                        since: SpecificationLevel::MEDIAQ4
                    }),

    // MQ 4 § 5.5
    OverflowBlock(name: "overflow-block",
                  value: {
                      "none" => None,
                      "scroll" => Scroll,
                      "optional-paged" => OptionalPaged,
                      "paged" => Paged
                  },
                  type: discrete,
                  availability: {
                      since: SpecificationLevel::MEDIAQ4
                  }),

    // MQ 4 § 5.6
    OverflowInline(name: "overflow-inline",
                   value: {
                       "none" => None,
                       "scroll" => Scroll
                   },
                   type: discrete,
                   availability: {
                       since: SpecificationLevel::MEDIAQ4
                   }),

    // MQ 4 § 6. Color
    // MQ 4 § 6.1
    Color(name: "color",
          value: u8,
          type: range,
          availability: {
              since: SpecificationLevel::MEDIAQ3
          },
          impl: {
              context: u8,
              zero: 0
          }),

    // MQ 4 § 6.2
    ColorIndex(name: "color-index",
               value: u32,
               type: range,
               availability: {
                   since: SpecificationLevel::MEDIAQ3
               },
               impl: {
                   context: u32,
                   zero: 0
               }),

    // MQ 4 § 6.3
    Monochrome(name: "monochrome",
               value: u32,
               type: range,
               availability: {
                   since: SpecificationLevel::MEDIAQ3
               },
               impl: {
                   context: u32,
                   zero: 0
               }),

    // MQ 4 § 6.4
    InvertedColors(name: "inverted-colors",
                   value: {
                       "none" => None,
                       "inverted" => Inverted
                   },
                   type: discrete,
                   availability: {
                       since: SpecificationLevel::MEDIAQ3
                   }),

    // MQ 4 § 7. Interaction
    // MQ 4 § 7.1
    Pointer(name: "pointer",
            value: {
                "none" => None,
                "coarse" => Coarse,
                "fine" => Fine
            },
            type: discrete,
            availability: {
                since: SpecificationLevel::MEDIAQ4
            }),

    // MQ 4 § 7.2
    Hover(name: "hover",
          value: {
              "none" => None,
              "on-demand" => OnDemand,
              "hover" => Hover
          },
          type: discrete,
          availability: {
              since: SpecificationLevel::MEDIAQ4
          }),

    // MQ 4 § 7.3
    AnyPointer(name: "any-pointer",
               value: {
                   "none" => None,
                   "coarse" => Coarse,
                   "fine" => Fine
               },
               type: discrete,
               availability: {
                   since: SpecificationLevel::MEDIAQ4
               }),

    // MQ 4 § 7.4
    AnyHover(name: "hover",
             value: {
                 "none" => None,
                 "on-demand" => OnDemand,
                 "hover" => Hover
             },
             type: discrete,
             availability: {
                 since: SpecificationLevel::MEDIAQ4
             }),

    // MQ 4 § 8. Environment
    // MQ 4 § 8.1
    LightLevel(name: "light-level",
               value: {
                   "dim" => Dim,
                   "normal" => Normal,
                   "washed" => Washed
               },
               type: discrete,
               availability: {
                   since: SpecificationLevel::MEDIAQ4
               },
               impl: {
                   no_none
               }),

    // MQ 4 § 9. Scripting
    // MQ 4 § 9.1
    Scripting(name: "scripting",
              value: {
                  "none" => None,
                  "initial-only" => InitialOnly,
                  "enabled" => Enabled
              },
              type: discrete,
              availability: {
                  since: SpecificationLevel::MEDIAQ4
              }),

    // MQ 4 § 11. Deprecated
    DeviceWidth(name: "device-width",
                value: Length,
                type: range,
                availability: {
                    since: SpecificationLevel::MEDIAQ3,
                    since: SpecificationLevel::MEDIAQ4
                },
                impl: {
                    context: Au,
                    zero: Au(0),
                    compute: compute_length
                }),

    // MQ 4 § 4.2
    DeviceHeight(name: "device-height",
                 value: Length,
                 type: range,
                 availability: {
                     since: SpecificationLevel::MEDIAQ3,
                     since: SpecificationLevel::MEDIAQ4
                 },
                 impl: {
                     context: Au,
                     zero: Au(0),
                     compute: compute_length
                 }),

    // MQ 4 § 4.3
    DeviceAspectRatio(name: "device-aspect-ratio",
                      value: Ratio,
                      type: range,
                      availability: {
                          since: SpecificationLevel::MEDIAQ3,
                          since: SpecificationLevel::MEDIAQ4
                      },
                      impl: {
                          context: f32,
                          zero: 0.
                      }),

    // View-mode § 5.1
    // http://www.w3.org/TR/view-mode/#view-modes
    ViewMode(name: "view-mode",
             value: {
                 "windowed" => Windowed,
                 "floating" => Floating,
                 "fullscreen" => Fullscreen,
                 "maximized" => Maximized,
                 "minimized" => Minimized
             },
             type: discrete,
             availability: {
                 since: SpecificationLevel::MEDIAQ3
             },
             impl: {
                 no_none
             })
}

enum RangePrefix {
    Min,
    Max
}

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

    fn from_css(input: &mut Parser) -> Result<T, ()> {
        ::std::num::NumCast::from(try!(input.expect_integer())).ok_or(())
    }
}

impl FromCss for MediaFeature {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaFeature, ()> {
        try!(input.expect_parenthesis_block());
        input.parse_nested_block(|input| {
            if let Ok(name) = input.try(|input| input.expect_ident()) {
                // ident-first form is the boolean and normal contexts, and part
                // of the range context (e.g. "(width >= 200px)")
                parse_ident_first_form(input, name)
            } else {
                parse_value_first_form(input)
            }
        })
    }
}

impl ToCss for MediaFeature {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        dispatch_to_css(self, dest)
    }
}

//derive_display_using_to_css!(MediaFeature);

fn parse_ident_first_form<'a>(input: &mut Parser, name: Cow<'a, str>)
                              -> Result<MediaFeature, ()>
{
    let resolved_name;
    let prefix;

    if name.len() >= 4 {
        match &name[..4] {
            p if "min-".eq_ignore_ascii_case(p) => {
                resolved_name = &name[4..];
                prefix = Some(RangePrefix::Min);
            }
            p if "max-".eq_ignore_ascii_case(p) => {
                resolved_name = &name[4..];
                prefix = Some(RangePrefix::Max);
            }
            _ => {
                resolved_name = &name;
                prefix = None;
            }
        }
    } else {
        resolved_name = &name;
        prefix = None;
    }

    dispatch_parse_ident_first_form(input, prefix, resolved_name)
}

fn parse_value_first_form(input: &mut Parser) -> Result<MediaFeature, ()> {
    // look-ahead to after ['=' | '<' | '<=' | '>' | '>=' ]
    let start = input.position();
    loop {
        match input.next() {
            Ok(Token::Delim(c)) => match c {
                '=' => break,
                '<' | '>' => {
                    let after_op = input.position();
                    match input.next_including_whitespace_and_comments() {
                        Ok(Token::Delim('=')) => {},
                        _ => input.reset(after_op)
                    }
                    break;
                }
                _ => continue
            },
            Ok(_) => continue,
            Err(_) => return Err(())
        }
    }

    // parse the feature name
    let name = try!(input.expect_ident());
    let after_name = input.position();

    // we have our feature name; we can reset the parser to `start`
    // and use Range::from_css now that we can infer the value type
    input.reset(start);
    dispatch_parse_value_first_form(input, &name, after_name)
}

fn parse_discrete<T>(input: &mut Parser, prefix: Option<RangePrefix>) -> Result<Option<T>, ()>
    where T: FromCss<Err=()>
{
    if prefix.is_none() {
        // boolean context?
        if input.is_exhausted() {
            Ok(None)
        } else {
            try!(input.expect_colon());
            FromCss::from_css(input).map(|value| Some(value))
        }
    } else {
        // MQ 4 § 2.4.4.
        // “Discrete” type properties do not accept “min-” or “max-” prefixes.
        Err(())
    }
}

fn parse_boolean_or_normal_range<T>(input: &mut Parser,
                                    prefix: Option<RangePrefix>)
                                    -> Result<Option<Range<T>>, ()>
    where T: FromCss<Err=()>
{
    if let Some(prefix) = prefix {
        try!(input.expect_colon());
        FromCss::from_css(input).map(|value| match prefix {
            RangePrefix::Min => Some(Range::Ge(value)),
            RangePrefix::Max => Some(Range::Le(value))
        })
    } else {
        if !input.is_exhausted() {
            FromCss::from_css(input).map(|value| Some(value))
        } else {
            Ok(None)
        }
    }
}

fn parse_range_form<T>(input: &mut Parser, after_name: SourcePosition)  -> Result<Range<T>, ()>
    where T: FromCss<Err=()>
{
    let first = try!(FromCss::from_css(input));
    input.reset(after_name);

    // we've parsed <value> <op> <ident>; now need to check if there's
    // another constraint (<op> <value>) to parse
    if input.is_exhausted() {
        Ok(first)
    } else {
        let second = try!(FromCss::from_css(input));
        if let Some(interval) = Range::interval(first, second) {
            Ok(interval)
        } else {
            Err(())
        }
    }
}

impl<C> EvaluateUsingContext<C> for MediaFeature
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        dispatch_evaluate(self, context)
    }
}

trait EvaluateUsingContextValue<C>
    where C: DeviceFeatureContext
{
    type ContextValue;

    fn evaluate(feature_value: &Option<Self>,
                context: &C,
                context_value: Self::ContextValue) -> bool;
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

#[cfg(test)]
mod tests;
