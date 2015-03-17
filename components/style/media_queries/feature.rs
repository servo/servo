/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{EvaluateUsingContext, MediaType};
use super::values::{discrete, EvaluateMediaFeatureValue, Range, range};

use ::FromCss;
use ::cssparser::{Parser, SourcePosition, ToCss, Token};
use ::geom::size::Size2D;
use ::util::geometry::Au;

use std::ascii::AsciiExt;
use std::borrow::Cow;

macro_rules! media_features {
    ($($css:expr => $feature:ident($feature_type:ident)),*) => {
        #[derive(Debug, PartialEq)]
        pub enum MediaFeature {
            $($feature(Option<$feature_type::$feature>)),*
        }

        #[allow(non_snake_case)]
        pub trait DeviceFeatureContext {
            fn MediaType(&self) -> MediaType;
            fn ViewportSize(&self) -> Size2D<Au>;

            $(fn $feature(&self) -> <$feature_type::$feature as EvaluateMediaFeatureValue<Self>>::Context;)*
        }

        impl<C> EvaluateUsingContext<C> for MediaFeature
            where C: DeviceFeatureContext
        {
            fn evaluate(&self, context: &C) -> bool {
                use super::values::EvaluateMediaFeatureValue;

                match *self {
                    $(MediaFeature::$feature(ref feature) =>
                          feature.evaluate(context, context.$feature())),*
                }
            }
        }

        impl ToCss for MediaFeature {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                match *self {
                    $(MediaFeature::$feature(ref value) =>
                          $feature_type!(ToCss, dest, $css, value)),*
                }
            }
        }

        fn dispatch_ident_first_form<'a>(input: &mut Parser,
                                         prefix: Option<RangePrefix>,
                                         name: &'a str)
                                         -> Result<MediaFeature, ()>
        {
            match name {
                $(n if $css.eq_ignore_ascii_case(n) =>
                      $feature_type!(MediaFeatureForm::IdentFirst,
                                     input, prefix, $feature)),*,
                _ => Err(())
            }
        }

        fn dispatch_value_first_form<'a>(input: &mut Parser,
                                         name: &'a str,
                                         after_name: SourcePosition)
                                         -> Result<MediaFeature, ()>
        {
            match name {
                $(n if $css.eq_ignore_ascii_case(n) =>
                      $feature_type!(MediaFeatureForm::ValueFirst,
                                     input, $feature, after_name)),*,
                _ => Err(())
            }
        }
    }
}

macro_rules! discrete {
    (ToCss, $dest:ident, $css:expr, $value:ident) => {
        match $value {
            &Some(ref value) => {
                try!(write!($dest, "({}: ", $css));
                try!(value.to_css($dest));
                write!($dest, ")")
            }
            &None => write!($dest, "({})", $css),
        }
    };
    (MediaFeatureForm::IdentFirst, $input:ident, $prefix:ident, $feature:ident) => {
        parse_discrete_value($input, $prefix).map(MediaFeature::$feature)
    };
    (MediaFeatureForm::ValueFirst, $input:ident, $feature:ident, $after_name:ident) => {
        Err(())
    }
}

macro_rules! range {
    (ToCss, $dest:ident, $css:expr, $value:ident) => {
        match $value {
            &Some(ref value) => {
                try!(write!($dest, "("));
                try!(value.to_css($dest, $css));
                write!($dest, ")")
            }
            &None => write!($dest, "({})", $css),
        }
    };
    (MediaFeatureForm::IdentFirst, $input:ident, $prefix:ident, $feature:ident) => {
        parse_boolean_or_normal_range_value($input, $prefix).map(|r| match r {
            Some(value) => MediaFeature::$feature(Some(range::$feature(value))),
            None => MediaFeature::$feature(None)
        })
    };
    (MediaFeatureForm::ValueFirst, $input:ident, $feature:ident, $after_name:ident) => {
        parse_range_form_value($input, $after_name)
            .map(|value| value.map(range::$feature))
            .map(MediaFeature::$feature)
    }
}

media_features! {
    // MQ 4 § 4. Screen/Device Dimensions
    "width" => Width(range),
    "height" => Height(range),
    "aspect-ratio" => AspectRatio(range),
    "orientation" => Orientation(discrete),

    // MQ 4 § 5. Display Quality
    //"resolution" => Resolution(range),
    "scan" => Scan(discrete),
    "grid" => Grid(discrete),
    "update-frequency" => UpdateFrequency(discrete),
    "overflow-block" => OverflowBlock(discrete),
    "overflow-inline" => OverflowInline(discrete),

    // MQ 4 § 6. Color
    "color" => Color(range),
    "color-index" => ColorIndex(range),
    "monochrome" => Monochrome(range),
    "inverted-colors" => InvertedColors(discrete),

    // MQ 4 § 7. Interaction
    "pointer" => Pointer(discrete),
    "hover" => Hover(discrete),
    "any-pointer" => AnyPointer(discrete),
    "any-hover" => AnyHover(discrete),

    // MQ 4 § 8. Environment
    "light-level" => LightLevel(discrete),

    // MQ 4 § 9. Scripting
    "scripting" => Scripting(discrete),

    // MQ 4 § 11. Deprecated
    "device-width" => DeviceWidth(range),
    "device-height" => DeviceHeight(range),
    "device-aspect-ratio" => DeviceAspectRatio(range)
}

derive_display_using_to_css!(MediaFeature);

enum RangePrefix {
    Min,
    Max
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

fn parse_ident_first_form<'a>(input: &mut Parser, name: Cow<'a, String, str>)
                              -> Result<MediaFeature, ()>
{
    if name.len() >= 4 {
        match &name[..4] {
            p if "min-".eq_ignore_ascii_case(p) =>
                dispatch_ident_first_form(input,
                                          Some(RangePrefix::Min),
                                          &name[4..]),
            p if "max-".eq_ignore_ascii_case(p) =>
                dispatch_ident_first_form(input,
                                          Some(RangePrefix::Max),
                                          &name[4..]),
            _ => dispatch_ident_first_form(input, None, &name[])
        }
    } else {
        dispatch_ident_first_form(input, None, &name[])
    }
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
    dispatch_value_first_form(input, &name[], after_name)
}

fn parse_discrete_value<T>(input: &mut Parser,
                           prefix: Option<RangePrefix>) -> Result<Option<T>, ()>
    where T: FromCss<Err=()>
{
    // MQ 4 § 2.4.4.
    // “Discrete” type properties do not accept “min-” or “max-” prefixes.
    if prefix.is_some() {
        return Err(())
    }

    if !input.is_exhausted() {
        try!(input.expect_colon());
        FromCss::from_css(input).map(|value| Some(value))
    } else {
        Ok(None)
    }
}

fn parse_boolean_or_normal_range_value<T>(input: &mut Parser,
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

fn parse_range_form_value<T>(input: &mut Parser,
                             after_name: SourcePosition)  -> Result<Option<Range<T>>, ()>
    where T: FromCss<Err=()>
{
    let first = try!(FromCss::from_css(input));
    input.reset(after_name);

    // we've parsed <value> <op> <ident>; now need to check if there's
    // another constraint (<op> <value>) to parse
    if input.is_exhausted() {
        Ok(Some(first))
    } else {
        let second = try!(FromCss::from_css(input));
        if let Some(interval) = Range::interval(first, second) {
            Ok(Some(interval))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::FromCss;
    use ::cssparser::Parser;

    #[test]
    fn parse_discrete() {
        use ::media_queries::values::discrete;

        macro_rules! assert_from_css_eq {
            ($css:expr, $feature:ident(None)) => {
                assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                           Ok(MediaFeature::$feature(None)))
            };
            ($css:expr, $feature:ident(Some($value:ident))) => {
                assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                           Ok(MediaFeature::$feature(Some(discrete::$feature::$value))))
            };
            ($css:expr, $feature:ident(Some(value = $value:expr))) => {
                assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                           Ok(MediaFeature::$feature(Some(discrete::$feature($value)))))
            }
        }

        // boolean context
        assert_from_css_eq!("(orientation)", Orientation(None));

        // normal context
        assert_from_css_eq!("(orientation: portrait)", Orientation(Some(Portrait)));
        assert_from_css_eq!("(orientation: landscape)", Orientation(Some(Landscape)));

        // 'grid' is a special case, due to the numeric booleans
        assert_from_css_eq!("(grid)", Grid(None));
        assert_from_css_eq!("(grid: 0)", Grid(Some(value = false)));
        assert_from_css_eq!("(grid: 1)", Grid(Some(value = true)));
    }

    #[test]
    fn parse_range() {
        use ::media_queries::values::{Range, range};
        use ::values::specified::Length;

        macro_rules! assert_from_css_eq {
            ($css:expr, Err) => {
                assert_eq!(<MediaFeature as FromCss>::from_css(&mut Parser::new($css)),
                           Err(()))
            };
            ($css:expr, $feature:ident(None)) => {
                assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                           Ok(MediaFeature::$feature(None)))
            };
            ($css:expr, $feature:ident($op:ident($value:expr))) => {
                assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                           Ok(MediaFeature::$feature(Some(range::$feature(Range::$op($value))))))
            };
            ($css:expr, $feature:ident(Interval($a:expr,$ac:expr,$b:expr,$bc:expr))) => {
                assert_eq!(FromCss::from_css(&mut Parser::new($css)),
                           Ok(MediaFeature::$feature(Some(range::$feature(Range::Interval($a,$ac,$b,$bc))))))
            };
        }

        // boolean context
        assert_from_css_eq!("(width", Width(None));
        assert_from_css_eq!("(width)", Width(None));

        // normal context
        assert_from_css_eq!("(width: 200px)", Width(Eq(Length::from_px(200.))));
        assert_from_css_eq!("(min-width: 200px)", Width(Ge(Length::from_px(200.))));
        assert_from_css_eq!("(max-width: 200px)", Width(Le(Length::from_px(200.))));

        // range context (name first)
        assert_from_css_eq!("(width  = 200px)", Width(Eq(Length::from_px(200.))));
        assert_from_css_eq!("(width <  200px)", Width(Lt(Length::from_px(200.))));
        assert_from_css_eq!("(width <= 200px)", Width(Le(Length::from_px(200.))));
        assert_from_css_eq!("(width >  200px)", Width(Gt(Length::from_px(200.))));
        assert_from_css_eq!("(width >= 200px)", Width(Ge(Length::from_px(200.))));

        // range context (value first)
        assert_from_css_eq!("(200px  = width)", Width(Eq(Length::from_px(200.))));
        assert_from_css_eq!("(200px <  width)", Width(Gt(Length::from_px(200.))));
        assert_from_css_eq!("(200px <= width)", Width(Ge(Length::from_px(200.))));
        assert_from_css_eq!("(200px >  width)", Width(Lt(Length::from_px(200.))));
        assert_from_css_eq!("(200px >= width)", Width(Le(Length::from_px(200.))));

        // range context (interval)
        assert_from_css_eq!("(0px <  width <  200px)", Width(Interval(Length::from_px(0.), false,
                                                                      Length::from_px(200.), false)));
        assert_from_css_eq!("(0px <= width <  200px)", Width(Interval(Length::from_px(0.), true,
                                                                      Length::from_px(200.), false)));
        assert_from_css_eq!("(0px <  width <= 200px)", Width(Interval(Length::from_px(0.), false,
                                                                      Length::from_px(200.), true)));
        assert_from_css_eq!("(0px <= width <= 200px)", Width(Interval(Length::from_px(0.), true,
                                                                      Length::from_px(200.), true)));

        assert_from_css_eq!("(200px >  width >  0px)", Width(Interval(Length::from_px(0.), false,
                                                                      Length::from_px(200.), false)));
        assert_from_css_eq!("(200px >= width >  0px)", Width(Interval(Length::from_px(0.), false,
                                                                      Length::from_px(200.), true)));
        assert_from_css_eq!("(200px >  width >= 0px)", Width(Interval(Length::from_px(0.), true,
                                                                      Length::from_px(200.), false)));
        assert_from_css_eq!("(200px >= width >= 0px)", Width(Interval(Length::from_px(0.), true,
                                                                      Length::from_px(200.), true)));

        // invalid
        assert_from_css_eq!("width", Err);
        assert_from_css_eq!("width)", Err);
    }
}
