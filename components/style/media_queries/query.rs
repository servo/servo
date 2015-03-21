/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::{DeviceFeatureContext, EvaluateUsingContext, SpecificationLevel};
use super::condition::MediaCondition;

use ::FromCss;
use ::cssparser::{Parser, ToCss};

use std::ascii::AsciiExt;
use std::borrow::ToOwned;

macro_rules! known_media_types {
    ($($css:expr => $variant:ident($($availability:tt)+)),+) =>
    {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum KnownMediaType {
            $($variant),*
        }

        derive_display_using_to_css!(KnownMediaType);

        impl KnownMediaType {
            fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaType, ()> {
                match &try!(input.expect_ident()) {
                    $(t if $css.eq_ignore_ascii_case(t) =>
                          known_media_types!(derive ToCss(level) for $variant($css),
                                             $($availability)+)),+,
                    t => Ok(MediaType::Unknown(t.to_ascii_lowercase()))
                }
            }
        }

        impl ToCss for KnownMediaType {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                match self {
                    $(&KnownMediaType::$variant => write!(dest, $css)),+
                }
            }
        }
    };

    (derive ToCss($level:ident) for $variant:ident($css:expr),
     since: $since:expr) =>
    {
        Ok(if *$level >= $since {
            MediaType::Known(KnownMediaType::$variant)
        } else {
            MediaType::Unknown($css.to_owned())
        })
    };
    (derive ToCss($level:ident) for $variant:ident($css:expr),
     since: $since:expr, deprecated: $deprecated:expr) =>
    {
        Ok(if *$level >= $since {
            if *$level < $deprecated {
                MediaType::Known(KnownMediaType::$variant)
            } else {
                MediaType::Deprecated(KnownMediaType::$variant)
            }
        } else {
            MediaType::Unknown($css.to_owned())
        })
    }
}

known_media_types! {
    "print" => Print(since: SpecificationLevel::MEDIAQ3),
    "screen" => Screen(since: SpecificationLevel::MEDIAQ3),
    "speech" => Speech(since: SpecificationLevel::MEDIAQ3),
    "tty" => TTY(since: SpecificationLevel::MEDIAQ3,
                 deprecated: SpecificationLevel::MEDIAQ4),
    "tv" => TV(since: SpecificationLevel::MEDIAQ3,
               deprecated: SpecificationLevel::MEDIAQ4),
    "projection" => Projection(since: SpecificationLevel::MEDIAQ3,
                               deprecated: SpecificationLevel::MEDIAQ4),
    "handheld" => Handheld(since: SpecificationLevel::MEDIAQ3,
                           deprecated: SpecificationLevel::MEDIAQ4),
    "braille" => Braille(since: SpecificationLevel::MEDIAQ3,
                         deprecated: SpecificationLevel::MEDIAQ4),
    "embossed" => Embossed(since: SpecificationLevel::MEDIAQ3,
                           deprecated: SpecificationLevel::MEDIAQ4),
    "aural" => Aural(since: SpecificationLevel::MEDIAQ3,
                     deprecated: SpecificationLevel::MEDIAQ3)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    All,
    Known(KnownMediaType),
    Deprecated(KnownMediaType),
    Unknown(String)
}

derive_display_using_to_css!(MediaType);

impl<C> EvaluateUsingContext<C> for MediaType
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        match self {
            &MediaType::All => true,
            &MediaType::Known(ref media_type) => context.MediaType() == *media_type,
            &MediaType::Deprecated(_) |
            &MediaType::Unknown(_) => false,
        }
    }
}

impl FromCss for MediaType {
    type Err = ();
    type Context = SpecificationLevel;

    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaType, ()> {
        match level {
            &SpecificationLevel::MEDIAQ3 => {
                if input.try(|input| input.expect_ident_matching("all")).is_ok() {
                    Ok(MediaType::All)
                } else {
                    KnownMediaType::from_css(input, level)
                }
            }
            &SpecificationLevel::MEDIAQ4 => {
                if input.try(|input| input.expect_ident_matching("all")).is_ok() {
                    Ok(MediaType::All)
                } else if input.try(|input| match &try!(input.expect_ident()) {
                    // MQ 4 § 3
                    // The <media-type> production does not include the keywords
                    // `only`, `not`, `and`, and `or`.
                    t if "only".eq_ignore_ascii_case(t) => Ok(()),
                    t if "not".eq_ignore_ascii_case(t) => Ok(()),
                    t if "and".eq_ignore_ascii_case(t) => Ok(()),
                    t if "or".eq_ignore_ascii_case(t) => Ok(()),
                    _ => Err(())
                }).is_ok() {
                    Err(())
                } else {
                    KnownMediaType::from_css(input, level)
                }
            }
        }
    }
}

impl ToCss for MediaType {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match self {
            &MediaType::All => write!(dest, "all"),
            &MediaType::Known(ref type_) |
            &MediaType::Deprecated(ref type_) => type_.to_css(dest),
            &MediaType::Unknown(ref name) => dest.write_str(name),
        }
    }
}

#[derive(Copy, Debug, PartialEq, Eq)]
pub enum Qualifier {
    Only,
    Not,
}

derive_display_using_to_css!(Qualifier);

impl FromCss for Qualifier {
    type Err = ();
    type Context = ();

    fn from_css(input: &mut Parser, _: &()) -> Result<Qualifier, ()> {
        match &try!(input.expect_ident()) {
            q if "only".eq_ignore_ascii_case(q) => Ok(Qualifier::Only),
            q if "not".eq_ignore_ascii_case(q) => Ok(Qualifier::Not),
            _ => Err(())
        }
    }
}

impl ToCss for Qualifier {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match self {
            &Qualifier::Only => write!(dest, "only"),
            &Qualifier::Not => write!(dest, "not")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MediaQuery {
    pub qualifier: Option<Qualifier>,
    pub media_type: MediaType,
    pub condition: Option<MediaCondition>
}

derive_display_using_to_css!(MediaQuery);

pub const ALL_MEDIA_QUERY: MediaQuery =
    MediaQuery {
        qualifier: None,
        media_type: MediaType::All,
        condition: None
    };

pub const NOT_ALL_MEDIA_QUERY: MediaQuery =
    MediaQuery {
        qualifier: Some(Qualifier::Not),
        media_type: MediaType::All,
        condition: None
    };

impl<C> EvaluateUsingContext<C> for MediaQuery
    where C: DeviceFeatureContext
{
    fn evaluate(&self, context: &C) -> bool {
        let result = if self.media_type.evaluate(context) {
            if let Some(ref condition) = self.condition {
                condition.evaluate(context)
            } else {
                true
            }
        } else {
            false
        };

        match self.qualifier {
            Some(Qualifier::Not) => !result,
            Some(Qualifier::Only) | None => result
        }
    }
}

impl FromCss for MediaQuery {
    type Err = ();
    type Context = SpecificationLevel;

    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaQuery, ()> {
        // MQ 4 § 3.1
        // A media query that does not match the grammar in the previous section
        // must be replaced by `not all` during parsing.
        macro_rules! try {
            ($expr:expr) => (match $expr {
                Result::Ok(val) => val,
                Result::Err(_) => {
                    // consume any remaining input
                    while input.next().is_ok() {}
                    return Ok(NOT_ALL_MEDIA_QUERY)
                }
            })
        }

        // <media-condition>
        if let Ok(condition) = input.try(|input| FromCss::from_css(input, level)) {
            Ok(MediaQuery {
                qualifier: None,
                media_type: MediaType::All,
                condition: Some(condition)
            })
        } else {
            // [ only | not ]?
            let qualifier = input.try(|input| FromCss::from_css(input, &())).ok();

            //  <media-type>
            let media_type = try!(FromCss::from_css(input, level));

            let condition = if !input.is_exhausted() {
                match level {
                    &SpecificationLevel::MEDIAQ3 => {
                        try!(expect_and!(input));
                        Some(try!(FromCss::from_css(input, level)))
                    }
                    &SpecificationLevel::MEDIAQ4 => {
                        // [ and <media-condition> ]?
                        try!(input.expect_ident_matching("and"));
                        Some(try!(FromCss::from_css(input, level)))
                    }
                }
            } else {
                None
            };

            Ok(MediaQuery {
                qualifier: qualifier,
                media_type: media_type,
                condition: condition,
            })
        }
    }
}

impl ToCss for MediaQuery {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match self {
            &MediaQuery { qualifier: Some(ref q), media_type: ref mt, condition: None } => {
                try!(q.to_css(dest));
                try!(write!(dest, " "));
                mt.to_css(dest)
            }
            &MediaQuery { qualifier: Some(ref q), media_type: ref mt, condition: Some(ref c) } => {
                try!(q.to_css(dest));
                try!(write!(dest, " "));
                try!(mt.to_css(dest));
                try!(write!(dest, " and "));
                c.to_css(dest)
            }
            &MediaQuery { qualifier: None, media_type: ref mt, condition: None } => {
                mt.to_css(dest)
            }
            &MediaQuery { qualifier: None, media_type: ref mt, condition: Some(ref c) } => {
                // omit 'all'
                if *mt != MediaType::All {
                    try!(mt.to_css(dest));
                    try!(write!(dest, " and "));
                }
                c.to_css(dest)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MediaQueryList {
    pub queries: Vec<MediaQuery>
}

derive_display_using_to_css!(MediaQueryList);

impl MediaQueryList {
    pub fn evaluate(&self, context: &super::Device) -> bool {
        self.queries.iter().any(|query| query.evaluate(context))
    }
}

impl FromCss for MediaQueryList {
    type Err = ();
    type Context = SpecificationLevel;

    fn from_css(input: &mut Parser, level: &SpecificationLevel) -> Result<MediaQueryList, ()> {
        let queries = if input.is_exhausted() {
            // MQ 4 § 2.1
            // An empty media query list evaluates to true.
            vec![ALL_MEDIA_QUERY]
        } else {
            match input.parse_comma_separated(|input| FromCss::from_css(input, level)) {
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

#[cfg(test)]
mod tests {
    use super::super::SpecificationLevel;
    use super::MediaQuery;

    use ::FromCss;
    use ::cssparser::{Parser, ToCss};

    #[test]
    fn parse_examples() {
        macro_rules! assert_roundtrip_eq {
            ($css:expr) => {
                assert_roundtrip_eq!($css => $css)
            };
            ($from:expr => $to:expr) => {{
                let query: MediaQuery = FromCss::from_css(&mut Parser::new($from), &SpecificationLevel::MEDIAQ4).unwrap();
                assert_eq!(query.to_css_string(),
                           $to)
            }}
        }

        assert_roundtrip_eq!("screen");
        assert_roundtrip_eq!("print");
        assert_roundtrip_eq!("screen and (color)");
        assert_roundtrip_eq!("projection and (color)");
        assert_roundtrip_eq!("not screen and (color)");
        assert_roundtrip_eq!("only screen and (color)");
        assert_roundtrip_eq!("speech and (device-aspect-ratio: 16/9)");
        assert_roundtrip_eq!("(width >= 600px)");
        assert_roundtrip_eq!("(width: 600px)");
        assert_roundtrip_eq!("(scripting)");
        assert_roundtrip_eq!("not (scripting)");
        assert_roundtrip_eq!("(scripting: enabled)");
        assert_roundtrip_eq!("(scripting: none)");
        assert_roundtrip_eq!("(color)");
        assert_roundtrip_eq!("(color > 0)");
        assert_roundtrip_eq!("(pointer)");
        assert_roundtrip_eq!("(scan)");
        assert_roundtrip_eq!("(height > 600px)");
        assert_roundtrip_eq!("(600px < height)" => "(height > 600px)");
        assert_roundtrip_eq!("(400px < width < 1000px)");
        assert_roundtrip_eq!("(min-grid: 1)" => "not all");
        assert_roundtrip_eq!("all");
        assert_roundtrip_eq!("example");
        assert_roundtrip_eq!("(example, all,)" => "not all");
        assert_roundtrip_eq!("&test" => "not all");
        assert_roundtrip_eq!("(example, speech { /* rules for speech devices */ }" => "not all");
        assert_roundtrip_eq!("or and (color)" => "not all");
        assert_roundtrip_eq!("screen and (max-weight: 3kg) and (color)" => "not all");
        assert_roundtrip_eq!("(min-orientation:portrait)" => "not all");
        assert_roundtrip_eq!("(min-width: -100px)" => "not all");
        assert_roundtrip_eq!("test;" => "not all");
        assert_roundtrip_eq!("print and (min-width: 25cm)" => "print and (width >= 944.866667px)");
        assert_roundtrip_eq!("(400px <= width <= 700px)");
        assert_roundtrip_eq!("(min-width: 20em)" => "(width >= 20em)");
        assert_roundtrip_eq!("(orientation:portrait)" => "(orientation: portrait)");
        //assert_roundtrip_eq!("(resolution >= 2dppx)");
        //assert_roundtrip_eq!("print and (min-resolution: 300dpi)");
        //assert_roundtrip_eq!("print and (min-resolution: 118dpcm)");
        assert_roundtrip_eq!("(scan: interlace)");
        assert_roundtrip_eq!("(grid) and (max-width: 15em)" => "(grid) and (width <= 15em)");
        assert_roundtrip_eq!("(update-frequency: none)");
        assert_roundtrip_eq!("(min-color: 1)" => "(color >= 1)");
        assert_roundtrip_eq!("(color >= 8)");
        assert_roundtrip_eq!("(color-index)");
        assert_roundtrip_eq!("(color-index >= 1)");
        assert_roundtrip_eq!("(min-color-index: 256)" => "(color-index >= 256)");
        assert_roundtrip_eq!("(monochrome)");
        assert_roundtrip_eq!("(monochrome >= 2)");
        assert_roundtrip_eq!("print and (color)");
        assert_roundtrip_eq!("print and (monochrome)");
        assert_roundtrip_eq!("(inverted-colors)");
        assert_roundtrip_eq!("(pointer:coarse)" => "(pointer: coarse)");
        assert_roundtrip_eq!("(hover)");
        assert_roundtrip_eq!("(light-level: normal)");
        assert_roundtrip_eq!("(light-level: dim)");
        assert_roundtrip_eq!("(light-level: washed)");
        assert_roundtrip_eq!("(device-width < 800px)");
        assert_roundtrip_eq!("(device-height > 600px)");
        assert_roundtrip_eq!("(device-aspect-ratio: 16/9)");
        assert_roundtrip_eq!("(device-aspect-ratio: 32/18)");
        assert_roundtrip_eq!("(device-aspect-ratio: 1280/720)");
        assert_roundtrip_eq!("(device-aspect-ratio: 2560/1440)");
    }
}
