/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::MediaCondition;

use ::FromCss;
use ::cssparser::{Parser, ToCss};

use std::ascii::AsciiExt;

macro_rules! media_types {
    (enum $name:ident { $($css:expr => $variant:ident),+ }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($variant),*
        }

        derive_display_using_to_css!($name);

        impl FromCss for $name {
            type Err = ();

            fn from_css(input: &mut Parser) -> Result<$name, ()> {
                match &try!(input.expect_ident())[] {
                    $(t if $css.eq_ignore_ascii_case(t) => Ok($name::$variant)),+,
                    _ => Err(())
                }
            }
        }

        impl ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
                where W: ::text_writer::TextWriter
            {
                match self {
                    $(&$name::$variant => write!(dest, $css)),+
                }
            }
        }
    }
}

media_types!(enum DefinedMediaType {
    "print" => Print,
    "screen" => Screen,
    "speech" => Speech
});

media_types!(enum DeprecatedMediaType {
    "tty" => TTY,
    "tv" => TV,
    "projection" => Projection,
    "handheld" => Handheld,
    "braille" => Braille,
    "embossed" => Embossed,
    "aural" => Aural
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaType {
    All,
    Defined(DefinedMediaType),
    Deprecated(DeprecatedMediaType),
    Unknown(String)
}

derive_display_using_to_css!(MediaType);

impl FromCss for MediaType {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaType, ()> {
        if let Ok(keyword_result) = input.try(|input| -> Result<Result<MediaType, ()>, ()> {
            match &try!(input.expect_ident())[] {
                t if "all".eq_ignore_ascii_case(t) => Ok(Ok(MediaType::All)),
                // MQ 4 § 3
                // The <media-type> production does not include the keywords
                // `only`, `not`, `and`, and `or`.
                t if "only".eq_ignore_ascii_case(t) => Ok(Err(())),
                t if "not".eq_ignore_ascii_case(t) => Ok(Err(())),
                t if "and".eq_ignore_ascii_case(t) => Ok(Err(())),
                t if "or".eq_ignore_ascii_case(t) => Ok(Err(())),
                _ => Err(())
            }
        }) {
            keyword_result
        } else if let Ok(defined) = input.try(FromCss::from_css) {
            Ok(MediaType::Defined(defined))
        } else if let Ok(deprecated) = input.try(FromCss::from_css) {
            Ok(MediaType::Deprecated(deprecated))
        } else {
            input.expect_ident().map(|name| MediaType::Unknown(name.into_owned()
                                                                   .to_ascii_lowercase()))
        }
    }
}

impl ToCss for MediaType {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        match self {
            &MediaType::All => write!(dest, "all"),
            &MediaType::Defined(ref type_) => type_.to_css(dest),
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

    fn from_css(input: &mut Parser) -> Result<Qualifier, ()> {
        match &try!(input.expect_ident())[] {
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

#[derive(Copy, Debug, PartialEq, Eq)]
struct QualifiedMediaType(Option<Qualifier>, MediaType);

derive_display_using_to_css!(QualifiedMediaType);

impl FromCss for QualifiedMediaType {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<QualifiedMediaType, ()> {
        // [ only | not ] <media-type>
        let qualifier = input.try(FromCss::from_css).ok();
        let media_type = try!(FromCss::from_css(input));

        Ok(QualifiedMediaType(qualifier, media_type))
    }
}

impl ToCss for QualifiedMediaType {
    fn to_css<W>(&self, dest: &mut W) -> ::text_writer::Result
        where W: ::text_writer::TextWriter
    {
        if let Some(q) = self.0 {
            try!(q.to_css(dest));
            try!(write!(dest, " "));
        }
        self.1.to_css(dest)
    }
}

#[derive(Debug, PartialEq)]
pub struct MediaQuery {
    media_type: Option<QualifiedMediaType>,
    condition: Option<MediaCondition>
}

derive_display_using_to_css!(MediaQuery);

const NOT_ALL_MEDIA_QUERY: MediaQuery =
    MediaQuery {
        media_type: Some(QualifiedMediaType(None, MediaType::All)),
        condition: None
    };

impl FromCss for MediaQuery {
    type Err = ();

    fn from_css(input: &mut Parser) -> Result<MediaQuery, ()> {
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
        if let Ok(condition) = input.try(FromCss::from_css) {
            Ok(MediaQuery {
                media_type: None,
                condition: Some(condition)
            })
        } else {
            // [ only | not ] <media-type>
            let media_type = Some(try!(FromCss::from_css(input)));

            // [ and <media-condition> ]?
            let condition = if !input.is_exhausted() {
                try!(input.expect_ident_matching("and"));
                Some(try!(FromCss::from_css(input)))
            } else {
                None
            };

            Ok(MediaQuery {
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
            &MediaQuery { media_type: None, condition: Some(ref c ) } =>
                c.to_css(dest),
            &MediaQuery { media_type: Some(ref qt), condition: None } =>
                qt.to_css(dest),
            &MediaQuery { media_type: Some(ref qt), condition: Some(ref c ) } => {
                try!(qt.to_css(dest));
                try!(write!(dest, " and "));
                c.to_css(dest)
            }
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
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
                let query: MediaQuery = FromCss::from_css(&mut Parser::new($from)).unwrap();
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
