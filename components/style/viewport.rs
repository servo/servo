/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@viewport`][at] at-rule and [`meta`][meta] element.
//!
//! [at]: https://drafts.csswg.org/css-device-adapt/#atviewport-rule
//! [meta]: https://drafts.csswg.org/css-device-adapt/#viewport-meta

use app_units::Au;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser, parse_important};
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use media_queries::Device;
use parser::{ParserContext, log_css_error};
use properties::ComputedValues;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt;
use std::iter::Enumerate;
use std::str::Chars;
use style_traits::{ToCss, ViewportPx};
use style_traits::viewport::{Orientation, UserZoom, ViewportConstraints, Zoom};
use stylesheets::{Stylesheet, Origin};
use values::computed::{Context, ToComputedValue};
use values::specified::{Length, LengthOrPercentageOrAuto, ViewportPercentageLength};

macro_rules! declare_viewport_descriptor {
    ( $( $variant: ident($data: ident), )+ ) => {
        declare_viewport_descriptor_inner!([] [ $( $variant($data), )+ ] 0);
    };
}

macro_rules! declare_viewport_descriptor_inner {
    (
        [ $( $assigned_variant: ident($assigned_data: ident) = $assigned_discriminant: expr, )* ]
        [
            $next_variant: ident($next_data: ident),
            $( $variant: ident($data: ident), )*
        ]
        $next_discriminant: expr
    ) => {
        declare_viewport_descriptor_inner! {
            [
                $( $assigned_variant($assigned_data) = $assigned_discriminant, )*
                $next_variant($next_data) = $next_discriminant,
            ]
            [ $( $variant($data), )* ]
            $next_discriminant + 1
        }
    };

    (
        [ $( $assigned_variant: ident($assigned_data: ident) = $assigned_discriminant: expr, )* ]
        [ ]
        $number_of_variants: expr
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum ViewportDescriptor {
            $(
                $assigned_variant($assigned_data),
            )+
        }

        const VIEWPORT_DESCRIPTOR_VARIANTS: usize = $number_of_variants;

        impl ViewportDescriptor {
            fn discriminant_value(&self) -> usize {
                match *self {
                    $(
                        ViewportDescriptor::$assigned_variant(..) => $assigned_discriminant,
                    )*
                }
            }
        }
    };
}

declare_viewport_descriptor! {
    MinWidth(ViewportLength),
    MaxWidth(ViewportLength),

    MinHeight(ViewportLength),
    MaxHeight(ViewportLength),

    Zoom(Zoom),
    MinZoom(Zoom),
    MaxZoom(Zoom),

    UserZoom(UserZoom),
    Orientation(Orientation),
}

trait FromMeta: Sized {
    fn from_meta(value: &str) -> Option<Self>;
}

// ViewportLength is a length | percentage | auto | extend-to-zoom
// See:
// * http://dev.w3.org/csswg/css-device-adapt/#min-max-width-desc
// * http://dev.w3.org/csswg/css-device-adapt/#extend-to-zoom
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ViewportLength {
    Specified(LengthOrPercentageOrAuto),
    ExtendToZoom
}

impl ToCss for ViewportLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            ViewportLength::Specified(length) => length.to_css(dest),
            ViewportLength::ExtendToZoom => write!(dest, "extend-to-zoom"),
        }
    }
}

impl FromMeta for ViewportLength {
    fn from_meta(value: &str) -> Option<ViewportLength> {
        macro_rules! specified {
            ($value:expr) => {
                ViewportLength::Specified(LengthOrPercentageOrAuto::Length($value))
            }
        }

        Some(match value {
            v if v.eq_ignore_ascii_case("device-width") =>
                specified!(Length::ViewportPercentage(ViewportPercentageLength::Vw(100.))),
            v if v.eq_ignore_ascii_case("device-height") =>
                specified!(Length::ViewportPercentage(ViewportPercentageLength::Vh(100.))),
            _ => {
                match value.parse::<f32>() {
                    Ok(n) if n >= 0. => specified!(Length::from_px(n.max(1.).min(10000.))),
                    Ok(_) => return None,
                    Err(_) => specified!(Length::from_px(1.))
                }
            }
        })
    }
}

impl ViewportLength {
    fn parse(input: &mut Parser) -> Result<ViewportLength, ()> {
        // we explicitly do not accept 'extend-to-zoom', since it is a UA internal value
        // for <META> viewport translation
        LengthOrPercentageOrAuto::parse_non_negative(input).map(ViewportLength::Specified)
    }
}

impl FromMeta for Zoom {
    fn from_meta(value: &str) -> Option<Zoom> {
        Some(match value {
            v if v.eq_ignore_ascii_case("yes") => Zoom::Number(1.),
            v if v.eq_ignore_ascii_case("no") => Zoom::Number(0.1),
            v if v.eq_ignore_ascii_case("device-width") => Zoom::Number(10.),
            v if v.eq_ignore_ascii_case("device-height") => Zoom::Number(10.),
            _ => {
                match value.parse::<f32>() {
                    Ok(n) if n >= 0. => Zoom::Number(n.max(0.1).min(10.)),
                    Ok(_) => return None,
                    Err(_) => Zoom::Number(0.1),
                }
            }
        })
    }
}

impl FromMeta for UserZoom {
    fn from_meta(value: &str) -> Option<UserZoom> {
        Some(match value {
            v if v.eq_ignore_ascii_case("yes") => UserZoom::Zoom,
            v if v.eq_ignore_ascii_case("no") => UserZoom::Fixed,
            v if v.eq_ignore_ascii_case("device-width") => UserZoom::Zoom,
            v if v.eq_ignore_ascii_case("device-height") => UserZoom::Zoom,
            _ => {
                match value.parse::<f32>() {
                    Ok(n) if n >= 1. || n <= -1. => UserZoom::Zoom,
                    _ => UserZoom::Fixed
                }
            }
        })
    }
}

struct ViewportRuleParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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
}

fn parse_shorthand(input: &mut Parser) -> Result<[ViewportLength; 2], ()> {
    let min = try!(ViewportLength::parse(input));
    match input.try(|input| ViewportLength::parse(input)) {
        Err(()) => Ok([min, min]),
        Ok(max) => Ok([min, max])
    }
}

impl<'a, 'b> AtRuleParser for ViewportRuleParser<'a, 'b> {
    type Prelude = ();
    type AtRule = Vec<ViewportDescriptorDeclaration>;
}

impl<'a, 'b> DeclarationParser for ViewportRuleParser<'a, 'b> {
    type Declaration = Vec<ViewportDescriptorDeclaration>;

    fn parse_value(&mut self, name: &str, input: &mut Parser) -> Result<Vec<ViewportDescriptorDeclaration>, ()> {
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
                ok!(MinWidth(ViewportLength::parse)),
            n if n.eq_ignore_ascii_case("max-width") =>
                ok!(MaxWidth(ViewportLength::parse)),
            n if n.eq_ignore_ascii_case("width") =>
                ok!(shorthand -> [MinWidth, MaxWidth]),

            n if n.eq_ignore_ascii_case("min-height") =>
                ok!(MinHeight(ViewportLength::parse)),
            n if n.eq_ignore_ascii_case("max-height") =>
                ok!(MaxHeight(ViewportLength::parse)),
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
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ViewportRule {
    pub declarations: Vec<ViewportDescriptorDeclaration>
}

/// Whitespace as defined by DEVICE-ADAPT § 9.2
// TODO: should we just use whitespace as defined by HTML5?
const WHITESPACE: &'static [char] = &['\t', '\n', '\r', ' '];

/// Separators as defined by DEVICE-ADAPT § 9.2
// need to use \x2c instead of ',' due to test-tidy
const SEPARATOR: &'static [char] = &['\x2c', ';'];

#[inline]
fn is_whitespace_separator_or_equals(c: &char) -> bool {
    WHITESPACE.contains(c) || SEPARATOR.contains(c) || *c == '='
}

impl ViewportRule {
    pub fn parse(input: &mut Parser, context: &ParserContext)
                     -> Result<ViewportRule, ()>
    {
        let parser = ViewportRuleParser { context: context };

        let mut cascade = Cascade::new();
        let mut parser = DeclarationListParser::new(input, parser);
        while let Some(result) = parser.next() {
            match result {
                Ok(declarations) => {
                    for declarations in declarations {
                        cascade.add(Cow::Owned(declarations))
                    }
                }
                Err(range) => {
                    let pos = range.start;
                    let message = format!("Unsupported @viewport descriptor declaration: '{}'",
                                          parser.input.slice(range));
                    log_css_error(parser.input, pos, &*message, &context);
                }
            }
        }
        Ok(ViewportRule { declarations: cascade.finish() })
    }

    pub fn from_meta(content: &str) -> Option<ViewportRule> {
        let mut declarations = vec![None; VIEWPORT_DESCRIPTOR_VARIANTS];
        macro_rules! push_descriptor {
            ($descriptor:ident($value:expr)) => {{
                let descriptor = ViewportDescriptor::$descriptor($value);
                let discriminant = descriptor.discriminant_value();
                declarations[discriminant] = Some(ViewportDescriptorDeclaration::new(
                    Origin::Author,
                    descriptor,
                    false));
            }
        }}

        let mut has_width = false;
        let mut has_height = false;
        let mut has_zoom = false;

        let mut iter = content.chars().enumerate();

        macro_rules! start_of_name {
            ($iter:ident) => {
                $iter.by_ref()
                    .skip_while(|&(_, c)| is_whitespace_separator_or_equals(&c))
                    .next()
            }
        }

        while let Some((start, _)) = start_of_name!(iter) {
            let property = ViewportRule::parse_meta_property(content,
                                                             &mut iter,
                                                             start);

            if let Some((name, value)) = property {
                macro_rules! push {
                    ($descriptor:ident($translate:path)) => {
                        if let Some(value) = $translate(value) {
                            push_descriptor!($descriptor(value));
                        }
                    }
                }

                match name {
                    n if n.eq_ignore_ascii_case("width") => {
                        if let Some(value) = ViewportLength::from_meta(value) {
                            push_descriptor!(MinWidth(ViewportLength::ExtendToZoom));
                            push_descriptor!(MaxWidth(value));
                            has_width = true;
                        }
                    }
                    n if n.eq_ignore_ascii_case("height") => {
                        if let Some(value) = ViewportLength::from_meta(value) {
                            push_descriptor!(MinHeight(ViewportLength::ExtendToZoom));
                            push_descriptor!(MaxHeight(value));
                            has_height = true;
                        }
                    }
                    n if n.eq_ignore_ascii_case("initial-scale") => {
                        if let Some(value) = Zoom::from_meta(value) {
                            push_descriptor!(Zoom(value));
                            has_zoom = true;
                        }
                    }
                    n if n.eq_ignore_ascii_case("minimum-scale") =>
                        push!(MinZoom(Zoom::from_meta)),
                    n if n.eq_ignore_ascii_case("maximum-scale") =>
                        push!(MaxZoom(Zoom::from_meta)),
                    n if n.eq_ignore_ascii_case("user-scalable") =>
                        push!(UserZoom(UserZoom::from_meta)),
                    _ => {}
                }
            }
        }

        // DEVICE-ADAPT § 9.4 - The 'width' and 'height' properties
        // http://dev.w3.org/csswg/css-device-adapt/#width-and-height-properties
        if !has_width && has_zoom {
            if has_height {
                push_descriptor!(MinWidth(ViewportLength::Specified(LengthOrPercentageOrAuto::Auto)));
                push_descriptor!(MaxWidth(ViewportLength::Specified(LengthOrPercentageOrAuto::Auto)));
            } else {
                push_descriptor!(MinWidth(ViewportLength::ExtendToZoom));
                push_descriptor!(MaxWidth(ViewportLength::ExtendToZoom));
            }
        }

        let declarations: Vec<_> = declarations.into_iter().filter_map(|entry| entry).collect();
        if !declarations.is_empty() {
            Some(ViewportRule { declarations: declarations })
        } else {
            None
        }
    }

    fn parse_meta_property<'a>(content: &'a str,
                               iter: &mut Enumerate<Chars<'a>>,
                               start: usize)
                               -> Option<(&'a str, &'a str)>
    {
        fn end_of_token(iter: &mut Enumerate<Chars>) -> Option<(usize, char)> {
            iter.by_ref()
                .skip_while(|&(_, c)| !is_whitespace_separator_or_equals(&c))
                .next()
        }

        fn skip_whitespace(iter: &mut Enumerate<Chars>) -> Option<(usize, char)> {
            iter.by_ref()
                .skip_while(|&(_, c)| WHITESPACE.contains(&c))
                .next()
        }

        // <name> <whitespace>* '='
        let end = match end_of_token(iter) {
            Some((end, c)) if WHITESPACE.contains(&c) => {
                match skip_whitespace(iter) {
                    Some((_, c)) if c == '=' => end,
                    _ => return None
                }
            }
            Some((end, c)) if c == '=' => end,
            _ => return None
        };
        let name = &content[start..end];

        // <whitespace>* <value>
        let start = match skip_whitespace(iter) {
            Some((start, c)) if !SEPARATOR.contains(&c) => start,
            _ => return None
        };
        let value = match end_of_token(iter) {
            Some((end, _)) => &content[start..end],
            _ => &content[start..]
        };

        Some((name, value))
    }
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

pub struct Cascade {
    declarations: Vec<Option<(usize, ViewportDescriptorDeclaration)>>,
    count_so_far: usize,
}

impl Cascade {
    pub fn new() -> Self {
        Cascade {
            declarations: vec![None; VIEWPORT_DESCRIPTOR_VARIANTS],
            count_so_far: 0,
        }
    }

    pub fn from_stylesheets<'a, I>(stylesheets: I, device: &Device) -> Self
    where I: IntoIterator, I::Item: AsRef<Stylesheet> {
        let mut cascade = Self::new();
        for stylesheet in stylesheets {
            stylesheet.as_ref().effective_viewport_rules(device, |rule| {
                for declaration in &rule.declarations {
                    cascade.add(Cow::Borrowed(declaration))
                }
            })
        }
        cascade
    }

    pub fn add(&mut self, declaration: Cow<ViewportDescriptorDeclaration>)  {
        let descriptor = declaration.descriptor.discriminant_value();

        match self.declarations[descriptor] {
            Some((ref mut order_of_appearance, ref mut entry_declaration)) => {
                if declaration.higher_or_equal_precendence(entry_declaration) {
                    *entry_declaration = declaration.into_owned();
                    *order_of_appearance = self.count_so_far;
                }
            }
            ref mut entry @ None => {
                *entry = Some((self.count_so_far, declaration.into_owned()));
            }
        }
        self.count_so_far += 1;
    }

    pub fn finish(mut self) -> Vec<ViewportDescriptorDeclaration> {
        // sort the descriptors by order of appearance
        self.declarations.sort_by_key(|entry| entry.as_ref().map(|&(index, _)| index));
        self.declarations.into_iter().filter_map(|entry| entry.map(|(_, decl)| decl)).collect()
    }
}

pub trait MaybeNew {
    fn maybe_new(initial_viewport: TypedSize2D<f32, ViewportPx>,
                     rule: &ViewportRule)
                     -> Option<ViewportConstraints>;
}

impl MaybeNew for ViewportConstraints {
    fn maybe_new(initial_viewport: TypedSize2D<f32, ViewportPx>,
                 rule: &ViewportRule)
                 -> Option<ViewportConstraints>
    {
        use std::cmp;

        if rule.declarations.is_empty() {
            return None
        }

        let mut min_width = None;
        let mut max_width = None;

        let mut min_height = None;
        let mut max_height = None;

        let mut initial_zoom = None;
        let mut min_zoom = None;
        let mut max_zoom = None;

        let mut user_zoom = UserZoom::Zoom;
        let mut orientation = Orientation::Auto;

        // collapse the list of declarations into descriptor values
        for declaration in &rule.declarations {
            match declaration.descriptor {
                ViewportDescriptor::MinWidth(value) => min_width = Some(value),
                ViewportDescriptor::MaxWidth(value) => max_width = Some(value),

                ViewportDescriptor::MinHeight(value) => min_height = Some(value),
                ViewportDescriptor::MaxHeight(value) => max_height = Some(value),

                ViewportDescriptor::Zoom(value) => initial_zoom = value.to_f32(),
                ViewportDescriptor::MinZoom(value) => min_zoom = value.to_f32(),
                ViewportDescriptor::MaxZoom(value) => max_zoom = value.to_f32(),

                ViewportDescriptor::UserZoom(value) => user_zoom = value,
                ViewportDescriptor::Orientation(value) => orientation = value
            }
        }

        // TODO: return `None` if all descriptors are either absent or initial value

        macro_rules! choose {
            ($op:ident, $opta:expr, $optb:expr) => {
                match ($opta, $optb) {
                    (None, None) => None,
                    (a, None) => a,
                    (None, b) => b,
                    (a, b) => Some(a.unwrap().$op(b.unwrap())),
                }
            }
        }
        macro_rules! min {
            ($opta:expr, $optb:expr) => {
                choose!(min, $opta, $optb)
            }
        }
        macro_rules! max {
            ($opta:expr, $optb:expr) => {
                choose!(max, $opta, $optb)
            }
        }

        // DEVICE-ADAPT § 6.2.1 Resolve min-zoom and max-zoom values
        if min_zoom.is_some() && max_zoom.is_some() {
            max_zoom = Some(min_zoom.unwrap().max(max_zoom.unwrap()))
        }

        // DEVICE-ADAPT § 6.2.2 Constrain zoom value to the [min-zoom, max-zoom] range
        if initial_zoom.is_some() {
            initial_zoom = max!(min_zoom, min!(max_zoom, initial_zoom));
        }

        // DEVICE-ADAPT § 6.2.3 Resolve non-auto lengths to pixel lengths
        //
        // Note: DEVICE-ADAPT § 5. states that relative length values are
        // resolved against initial values
        let initial_viewport = Size2D::new(Au::from_f32_px(initial_viewport.width),
                                           Au::from_f32_px(initial_viewport.height));


        let context = Context {
            is_root_element: false,
            viewport_size: initial_viewport,
            inherited_style: ComputedValues::initial_values(),
            style: ComputedValues::initial_values().clone(),
            font_metrics_provider: None, // TODO: Should have!
        };

        // DEVICE-ADAPT § 9.3 Resolving 'extend-to-zoom'
        let extend_width;
        let extend_height;
        if let Some(extend_zoom) = max!(initial_zoom, max_zoom) {
            let scale_factor = 1. / extend_zoom;
            extend_width = Some(initial_viewport.width.scale_by(scale_factor));
            extend_height = Some(initial_viewport.height.scale_by(scale_factor));
        } else {
            extend_width = None;
            extend_height = None;
        }

        macro_rules! to_pixel_length {
            ($value:ident, $dimension:ident, $extend_to:ident => $auto_extend_to:expr) => {
                if let Some($value) = $value {
                    match $value {
                        ViewportLength::Specified(length) => match length {
                            LengthOrPercentageOrAuto::Length(value) =>
                                Some(value.to_computed_value(&context)),
                            LengthOrPercentageOrAuto::Percentage(value) =>
                                Some(initial_viewport.$dimension.scale_by(value.0)),
                            LengthOrPercentageOrAuto::Auto => None,
                            LengthOrPercentageOrAuto::Calc(calc) => {
                                let calc = calc.to_computed_value(&context);
                                Some(initial_viewport.$dimension.scale_by(calc.percentage()) + calc.length())
                            }
                        },
                        ViewportLength::ExtendToZoom => {
                            // $extend_to will be 'None' if 'extend-to-zoom' is 'auto'
                            match ($extend_to, $auto_extend_to) {
                                (None, None) => None,
                                (a, None) => a,
                                (None, b) => b,
                                (a, b) => cmp::max(a, b)
                            }
                        }
                    }
                } else {
                    None
                }
            }
        }

        // DEVICE-ADAPT § 9.3 states that max-descriptors need to be resolved
        // before min-descriptors.
        // http://dev.w3.org/csswg/css-device-adapt/#resolve-extend-to-zoom
        let max_width = to_pixel_length!(max_width, width, extend_width => None);
        let max_height = to_pixel_length!(max_height, height, extend_height => None);

        let min_width = to_pixel_length!(min_width, width, extend_width => max_width);
        let min_height = to_pixel_length!(min_height, height, extend_height => max_height);

        // DEVICE-ADAPT § 6.2.4 Resolve initial width and height from min/max descriptors
        macro_rules! resolve {
            ($min:ident, $max:ident, $initial:expr) => {
                if $min.is_some() || $max.is_some() {
                    let max = match $max {
                        Some(max) => cmp::min(max, $initial),
                        None => $initial
                    };

                    Some(match $min {
                        Some(min) => cmp::max(min, max),
                        None => max
                    })
                } else {
                    None
                };
            }
        }

        let width = resolve!(min_width, max_width, initial_viewport.width);
        let height = resolve!(min_height, max_height, initial_viewport.height);

        // DEVICE-ADAPT § 6.2.5 Resolve width value
        let width = if width.is_none() && height.is_none() {
             Some(initial_viewport.width)
        } else {
            width
        };

        let width = width.unwrap_or_else(|| match initial_viewport.height {
            Au(0) => initial_viewport.width,
            initial_height => {
                let ratio = initial_viewport.width.to_f32_px() / initial_height.to_f32_px();
                Au::from_f32_px(height.unwrap().to_f32_px() * ratio)
            }
        });

        // DEVICE-ADAPT § 6.2.6 Resolve height value
        let height = height.unwrap_or_else(|| match initial_viewport.width {
            Au(0) => initial_viewport.height,
            initial_width => {
                let ratio = initial_viewport.height.to_f32_px() / initial_width.to_f32_px();
                Au::from_f32_px(width.to_f32_px() * ratio)
            }
        });

        Some(ViewportConstraints {
            size: TypedSize2D::new(width.to_f32_px(), height.to_f32_px()),

            // TODO: compute a zoom factor for 'auto' as suggested by DEVICE-ADAPT § 10.
            initial_zoom: ScaleFactor::new(initial_zoom.unwrap_or(1.)),
            min_zoom: min_zoom.map(ScaleFactor::new),
            max_zoom: max_zoom.map(ScaleFactor::new),

            user_zoom: user_zoom,
            orientation: orientation
        })
    }
}
