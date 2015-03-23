/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, DeclarationListParser, AtRuleParser, DeclarationParser, ToCss, parse_important};
use parser::{ParserContext, log_css_error};
use stylesheets::Origin;
use values::specified::{AllowedNumericType, Length, LengthOrPercentageOrAuto};

use std::ascii::AsciiExt;
use std::collections::hash_map::{Entry, HashMap};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ViewportDescriptor {
    MinWidth(LengthOrPercentageOrAuto),
    MaxWidth(LengthOrPercentageOrAuto),

    MinHeight(LengthOrPercentageOrAuto),
    MaxHeight(LengthOrPercentageOrAuto),

    Zoom(Zoom),
    MinZoom(Zoom),
    MaxZoom(Zoom),

    UserZoom(UserZoom),
    Orientation(Orientation)
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, FromPrimitive)]
enum ViewportDescriptorKind {
    MinWidth,
    MaxWidth,

    MinHeight,
    MaxHeight,

    Zoom,
    MinZoom,
    MaxZoom,

    UserZoom,
    Orientation
}

impl ViewportDescriptor {
    fn kind(&self) -> ViewportDescriptorKind {
        match self {
            &ViewportDescriptor::MinWidth(_) => ViewportDescriptorKind::MinWidth,
            &ViewportDescriptor::MaxWidth(_) => ViewportDescriptorKind::MaxWidth,

            &ViewportDescriptor::MinHeight(_) => ViewportDescriptorKind::MinHeight,
            &ViewportDescriptor::MaxHeight(_) => ViewportDescriptorKind::MaxHeight,

            &ViewportDescriptor::Zoom(_) => ViewportDescriptorKind::Zoom,
            &ViewportDescriptor::MinZoom(_) => ViewportDescriptorKind::MinZoom,
            &ViewportDescriptor::MaxZoom(_) => ViewportDescriptorKind::MaxZoom,

            &ViewportDescriptor::UserZoom(_) => ViewportDescriptorKind::UserZoom,
            &ViewportDescriptor::Orientation(_) => ViewportDescriptorKind::Orientation
        }
    }
}

/// Zoom is a number | percentage | auto
/// See http://dev.w3.org/csswg/css-device-adapt/#descdef-viewport-zoom
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Zoom {
    Number(f64),
    Percentage(f64),
    Auto,
}

impl fmt::Display for Zoom {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_to_css(f)
    }
}

impl ToCss for Zoom {
    fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
        match self {
            &Zoom::Number(number) => write!(dest, "{}", number),
            &Zoom::Percentage(percentage) => write!(dest, "{}%", percentage * 100.),
            &Zoom::Auto => write!(dest, "auto")
        }
    }
}

impl Zoom {
    pub fn parse(input: &mut Parser) -> Result<Zoom, ()> {
        use cssparser::Token;

        match try!(input.next()) {
            Token::Percentage(ref value) if AllowedNumericType::NonNegative.is_ok(value.unit_value) =>
                Ok(Zoom::Percentage(value.unit_value)),
            Token::Number(ref value) if AllowedNumericType::NonNegative.is_ok(value.value) =>
                Ok(Zoom::Number(value.value)),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(Zoom::Auto),
            _ => Err(())
        }
    }

    #[inline]
    pub fn to_f32(&self) -> Option<f32> {
        match self {
            &Zoom::Number(number) => Some(number as f32),
            &Zoom::Percentage(percentage) => Some(percentage as f32),
            &Zoom::Auto => None
        }
    }
}

define_css_keyword_enum!(UserZoom:
                         "zoom" => Zoom,
                         "fixed" => Fixed);

impl fmt::Display for UserZoom {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_to_css(f)
    }
}

define_css_keyword_enum!(Orientation:
                         "auto" => Auto,
                         "portrait" => Portrait,
                         "landscape" => Landscape);

impl fmt::Display for Orientation {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_to_css(f)
    }
}

struct ViewportRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ViewportDescriptorDeclaration {
    pub origin: Origin,
    pub descriptor: ViewportDescriptor,
    pub important: bool
}

impl ViewportDescriptorDeclaration {
    pub fn new(origin: Origin,
               descriptor: ViewportDescriptor,
               important: bool) -> ViewportDescriptorDeclaration
    {
        ViewportDescriptorDeclaration {
            origin: origin,
            descriptor: descriptor,
            important: important
        }
    }

    #[inline]
    fn kind(&self) -> ViewportDescriptorKind {
        self.descriptor.kind()
    }
}

fn parse_shorthand(input: &mut Parser) -> Result<[LengthOrPercentageOrAuto; 2], ()> {
    let min = try!(LengthOrPercentageOrAuto::parse_non_negative(input));
    match input.try(|input| LengthOrPercentageOrAuto::parse_non_negative(input)) {
        Err(()) => Ok([min.clone(), min]),
        Ok(max) => Ok([min, max])
    }
}

impl<'a, 'b> AtRuleParser for ViewportRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = Vec<ViewportDescriptorDeclaration>;
}

impl<'a, 'b> DeclarationParser for ViewportRuleParser<'a, 'b> {
    type Declaration = Vec<ViewportDescriptorDeclaration>;

    fn parse_value(&self, name: &str, input: &mut Parser) -> Result<Vec<ViewportDescriptorDeclaration>, ()> {
        macro_rules! declaration {
            ($declaration:ident($parse:path)) => {
                declaration!($declaration(value: try!($parse(input)),
                                          important: input.try(parse_important).is_ok()))
            };
            ($declaration:ident(value: $value:expr, important: $important:expr)) => {
                ViewportDescriptorDeclaration::new(
                    self.context.stylesheet_origin,
                    ViewportDescriptor::$declaration($value),
                    $important)
            }
        }

        macro_rules! ok {
            ($declaration:ident($parse:path)) => {
                Ok(vec![declaration!($declaration($parse))])
            };
            (shorthand -> [$min:ident, $max:ident]) => {{
                let shorthand = try!(parse_shorthand(input));
                let important = input.try(parse_important).is_ok();

                Ok(vec![declaration!($min(value: shorthand[0], important: important)),
                        declaration!($max(value: shorthand[1], important: important))])
            }}
        }

        match name {
            n if n.eq_ignore_ascii_case("min-width") =>
                ok!(MinWidth(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("max-width") =>
                ok!(MaxWidth(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("width") =>
                ok!(shorthand -> [MinWidth, MaxWidth]),

            n if n.eq_ignore_ascii_case("min-height") =>
                ok!(MinHeight(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("max-height") =>
                ok!(MaxHeight(LengthOrPercentageOrAuto::parse_non_negative)),
            n if n.eq_ignore_ascii_case("height") =>
                ok!(shorthand -> [MinHeight, MaxHeight]),

            n if n.eq_ignore_ascii_case("zoom") =>
                ok!(Zoom(Zoom::parse)),
            n if n.eq_ignore_ascii_case("min-zoom") =>
                ok!(MinZoom(Zoom::parse)),
            n if n.eq_ignore_ascii_case("max-zoom") =>
                ok!(MaxZoom(Zoom::parse)),

            n if n.eq_ignore_ascii_case("user-zoom") =>
                ok!(UserZoom(UserZoom::parse)),
            n if n.eq_ignore_ascii_case("orientation") =>
                ok!(Orientation(Orientation::parse)),

            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ViewportRule {
    pub declarations: Vec<ViewportDescriptorDeclaration>
}

impl ViewportRule {
    pub fn parse<'a>(input: &mut Parser, context: &'a ParserContext)
                     -> Result<ViewportRule, ()>
    {
        let parser = ViewportRuleParser { context: context };

        let mut errors = vec![];
        let valid_declarations = DeclarationListParser::new(input, parser)
            .filter_map(|result| {
                match result {
                    Ok(declarations) => Some(declarations),
                    Err(range) => {
                        errors.push(range);
                        None
                    }
                }
            })
            .flat_map(|declarations| declarations.into_iter())
            .collect::<Vec<_>>();

        for range in errors {
            let pos = range.start;
            let message = format!("Unsupported @viewport descriptor declaration: '{}'",
                                  input.slice(range));
            log_css_error(input, pos, &*message);
        }

        Ok(ViewportRule { declarations: valid_declarations.iter().cascade() })
    }
}

pub trait ViewportRuleCascade: Iterator + Sized {
    fn cascade(self) -> ViewportRule;
}

impl<'a, I> ViewportRuleCascade for I
    where I: Iterator<Item=&'a ViewportRule>
{
    #[inline]
    fn cascade(self) -> ViewportRule {
        ViewportRule {
            declarations: self.flat_map(|r| r.declarations.iter()).cascade()
        }
    }
}

trait ViewportDescriptorDeclarationCascade: Iterator + Sized {
    fn cascade(self) -> Vec<ViewportDescriptorDeclaration>;
}

/// Computes the cascade precedence as according to
/// http://dev.w3.org/csswg/css-cascade/#cascade-origin
fn cascade_precendence(origin: Origin, important: bool) -> u8 {
    match (origin, important) {
        (Origin::UserAgent, true) => 1,
        (Origin::User, true) => 2,
        (Origin::Author, true) => 3,
        (Origin::Author, false) => 4,
        (Origin::User, false) => 5,
        (Origin::UserAgent, false) => 6,
    }
}

impl ViewportDescriptorDeclaration {
    fn higher_or_equal_precendence(&self, other: &ViewportDescriptorDeclaration) -> bool {
        let self_precedence = cascade_precendence(self.origin, self.important);
        let other_precedence = cascade_precendence(other.origin, other.important);

        self_precedence <= other_precedence
    }
}

fn cascade<'a, I>(iter: I) -> Vec<ViewportDescriptorDeclaration>
    where I: Iterator<Item=&'a ViewportDescriptorDeclaration>
{
    let mut declarations: HashMap<ViewportDescriptorKind, (usize, &'a ViewportDescriptorDeclaration)> = HashMap::new();

    // index is used to reconstruct order of appearance after all declarations
    // have been added to the map
    let mut index = 0;
    for declaration in iter {
        match declarations.entry(declaration.kind()) {
            Entry::Occupied(mut entry) => {
                if declaration.higher_or_equal_precendence(entry.get().1) {
                    entry.insert((index, declaration));
                    index += 1;
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((index, declaration));
                index += 1;
            }
        }
    }

    // convert to a list and sort the descriptors by order of appearance
    let mut declarations: Vec<_> = declarations.into_iter().map(|kv| kv.1).collect();
    declarations.sort_by(|a, b| a.0.cmp(&b.0));
    declarations.into_iter().map(|id| *id.1).collect::<Vec<_>>()
}

impl<'a, I> ViewportDescriptorDeclarationCascade for I
    where I: Iterator<Item=&'a ViewportDescriptorDeclaration>
{
    #[inline]
    fn cascade(self) -> Vec<ViewportDescriptorDeclaration> {
        cascade(self)
    }
}
