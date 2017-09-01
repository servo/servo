/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The [`@viewport`][at] at-rule and [`meta`][meta] element.
//!
//! [at]: https://drafts.csswg.org/css-device-adapt/#atviewport-rule
//! [meta]: https://drafts.csswg.org/css-device-adapt/#viewport-meta

use app_units::Au;
use context::QuirksMode;
use cssparser::{AtRuleParser, DeclarationListParser, DeclarationParser, Parser, parse_important};
use cssparser::{CowRcStr, ToCss as ParserToCss};
use error_reporting::{ContextualParseError, ParseErrorReporter};
use euclid::TypedSize2D;
use font_metrics::get_metrics_provider_for_product;
use media_queries::Device;
use parser::{ParserContext, ParserErrorContext};
use properties::StyleBuilder;
use selectors::parser::SelectorParseError;
use shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt;
use std::iter::Enumerate;
use std::str::Chars;
use style_traits::{PinchZoomFactor, ToCss, ParseError, StyleParseError};
use style_traits::viewport::{Orientation, UserZoom, ViewportConstraints, Zoom};
use stylesheets::{StylesheetInDocument, Origin};
use values::computed::{Context, ToComputedValue};
use values::specified::{NoCalcLength, LengthOrPercentageOrAuto, ViewportPercentageLength};

/// Whether parsing and processing of `@viewport` rules is enabled.
#[cfg(feature = "servo")]
pub fn enabled() -> bool {
    use servo_config::prefs::PREFS;
    PREFS.get("layout.viewport.enabled").as_boolean().unwrap_or(false)
}

/// Whether parsing and processing of `@viewport` rules is enabled.
#[cfg(not(feature = "servo"))]
pub fn enabled() -> bool {
    false // Gecko doesn't support @viewport.
}

macro_rules! declare_viewport_descriptor {
    ( $( $variant_name: expr => $variant: ident($data: ident), )+ ) => {
         declare_viewport_descriptor_inner!([] [ $( $variant_name => $variant($data), )+ ] 0);
    };
}

macro_rules! declare_viewport_descriptor_inner {
    (
        [ $( $assigned_variant_name: expr =>
             $assigned_variant: ident($assigned_data: ident) = $assigned_discriminant: expr, )* ]
        [
            $next_variant_name: expr => $next_variant: ident($next_data: ident),
            $( $variant_name: expr => $variant: ident($data: ident), )*
        ]
        $next_discriminant: expr
    ) => {
        declare_viewport_descriptor_inner! {
            [
                $( $assigned_variant_name => $assigned_variant($assigned_data) = $assigned_discriminant, )*
                $next_variant_name => $next_variant($next_data) = $next_discriminant,
            ]
            [ $( $variant_name => $variant($data), )* ]
            $next_discriminant + 1
        }
    };

    (
        [ $( $assigned_variant_name: expr =>
             $assigned_variant: ident($assigned_data: ident) = $assigned_discriminant: expr, )* ]
        [ ]
        $number_of_variants: expr
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[allow(missing_docs)]
        pub enum ViewportDescriptor {
            $(
                $assigned_variant($assigned_data),
            )+
        }

        const VIEWPORT_DESCRIPTOR_VARIANTS: usize = $number_of_variants;

        impl ViewportDescriptor {
            #[allow(missing_docs)]
            pub fn discriminant_value(&self) -> usize {
                match *self {
                    $(
                        ViewportDescriptor::$assigned_variant(..) => $assigned_discriminant,
                    )*
                }
            }
        }

        impl ToCss for ViewportDescriptor {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    $(
                        ViewportDescriptor::$assigned_variant(ref val) => {
                            dest.write_str($assigned_variant_name)?;
                            dest.write_str(": ")?;
                            val.to_css(dest)?;
                        },
                    )*
                }
                dest.write_str(";")
            }
        }
    };
}

declare_viewport_descriptor! {
    "min-width" => MinWidth(ViewportLength),
    "max-width" => MaxWidth(ViewportLength),

    "min-height" => MinHeight(ViewportLength),
    "max-height" => MaxHeight(ViewportLength),

    "zoom" => Zoom(Zoom),
    "min-zoom" => MinZoom(Zoom),
    "max-zoom" => MaxZoom(Zoom),

    "user-zoom" => UserZoom(UserZoom),
    "orientation" => Orientation(Orientation),
}

trait FromMeta: Sized {
    fn from_meta(value: &str) -> Option<Self>;
}

/// ViewportLength is a length | percentage | auto | extend-to-zoom
/// See:
/// * http://dev.w3.org/csswg/css-device-adapt/#min-max-width-desc
/// * http://dev.w3.org/csswg/css-device-adapt/#extend-to-zoom
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum ViewportLength {
    Specified(LengthOrPercentageOrAuto),
    ExtendToZoom
}

impl ToCss for ViewportLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            ViewportLength::Specified(ref length) => length.to_css(dest),
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
                specified!(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(100.))),
            v if v.eq_ignore_ascii_case("device-height") =>
                specified!(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vh(100.))),
            _ => {
                match value.parse::<f32>() {
                    Ok(n) if n >= 0. => specified!(NoCalcLength::from_px(n.max(1.).min(10000.))),
                    Ok(_) => return None,
                    Err(_) => specified!(NoCalcLength::from_px(1.))
                }
            }
        })
    }
}

impl ViewportLength {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        // we explicitly do not accept 'extend-to-zoom', since it is a UA
        // internal value for <META> viewport translation
        LengthOrPercentageOrAuto::parse_non_negative(context, input).map(ViewportLength::Specified)
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
#[allow(missing_docs)]
pub struct ViewportDescriptorDeclaration {
    pub origin: Origin,
    pub descriptor: ViewportDescriptor,
    pub important: bool
}

impl ViewportDescriptorDeclaration {
    #[allow(missing_docs)]
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

impl ToCss for ViewportDescriptorDeclaration {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.descriptor.to_css(dest)?;
        if self.important {
            dest.write_str(" !important")?;
        }
        dest.write_str(";")
    }
}

fn parse_shorthand<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                           -> Result<(ViewportLength, ViewportLength), ParseError<'i>> {
    let min = ViewportLength::parse(context, input)?;
    match input.try(|i| ViewportLength::parse(context, i)) {
        Err(_) => Ok((min.clone(), min)),
        Ok(max) => Ok((min, max))
    }
}

impl<'a, 'b, 'i> AtRuleParser<'i> for ViewportRuleParser<'a, 'b> {
    type PreludeNoBlock = ();
    type PreludeBlock = ();
    type AtRule = Vec<ViewportDescriptorDeclaration>;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;
}

impl<'a, 'b, 'i> DeclarationParser<'i> for ViewportRuleParser<'a, 'b> {
    type Declaration = Vec<ViewportDescriptorDeclaration>;
    type Error = SelectorParseError<'i, StyleParseError<'i>>;

    fn parse_value<'t>(&mut self, name: CowRcStr<'i>, input: &mut Parser<'i, 't>)
                       -> Result<Vec<ViewportDescriptorDeclaration>, ParseError<'i>> {
        macro_rules! declaration {
            ($declaration:ident($parse:expr)) => {
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
            ($declaration:ident($parse:expr)) => {
                Ok(vec![declaration!($declaration($parse))])
            };
            (shorthand -> [$min:ident, $max:ident]) => {{
                let shorthand = parse_shorthand(self.context, input)?;
                let important = input.try(parse_important).is_ok();

                Ok(vec![declaration!($min(value: shorthand.0, important: important)),
                        declaration!($max(value: shorthand.1, important: important))])
            }}
        }

        match_ignore_ascii_case! { &*name,
            "min-width" => ok!(MinWidth(|i| ViewportLength::parse(self.context, i))),
            "max-width" => ok!(MaxWidth(|i| ViewportLength::parse(self.context, i))),
            "width" => ok!(shorthand -> [MinWidth, MaxWidth]),
            "min-height" => ok!(MinHeight(|i| ViewportLength::parse(self.context, i))),
            "max-height" => ok!(MaxHeight(|i| ViewportLength::parse(self.context, i))),
            "height" => ok!(shorthand -> [MinHeight, MaxHeight]),
            "zoom" => ok!(Zoom(Zoom::parse)),
            "min-zoom" => ok!(MinZoom(Zoom::parse)),
            "max-zoom" => ok!(MaxZoom(Zoom::parse)),
            "user-zoom" => ok!(UserZoom(UserZoom::parse)),
            "orientation" => ok!(Orientation(Orientation::parse)),
            _ => Err(SelectorParseError::UnexpectedIdent(name.clone()).into()),
        }
    }
}

/// A `@viewport` rule.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ViewportRule {
    /// The declarations contained in this @viewport rule.
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
    /// Parse a single @viewport rule.
    pub fn parse<'i, 't, R>(context: &ParserContext,
                            error_context: &ParserErrorContext<R>,
                            input: &mut Parser<'i, 't>)
                            -> Result<Self, ParseError<'i>>
        where R: ParseErrorReporter
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
                Err(err) => {
                    let error = ContextualParseError::UnsupportedViewportDescriptorDeclaration(err.slice, err.error);
                    context.log_css_error(error_context, err.location, error);
                }
            }
        }
        Ok(ViewportRule { declarations: cascade.finish() })
    }
}

impl ViewportRule {
    #[allow(missing_docs)]
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

impl ToCssWithGuard for ViewportRule {
    // Serialization of ViewportRule is not specced.
    fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        dest.write_str("@viewport { ")?;
        let mut iter = self.declarations.iter();
        iter.next().unwrap().to_css(dest)?;
        for declaration in iter {
            dest.write_str(" ")?;
            declaration.to_css(dest)?;
        }
        dest.write_str(" }")
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

#[allow(missing_docs)]
pub struct Cascade {
    declarations: Vec<Option<(usize, ViewportDescriptorDeclaration)>>,
    count_so_far: usize,
}

#[allow(missing_docs)]
impl Cascade {
    pub fn new() -> Self {
        Cascade {
            declarations: vec![None; VIEWPORT_DESCRIPTOR_VARIANTS],
            count_so_far: 0,
        }
    }

    pub fn from_stylesheets<'a, I, S>(
        stylesheets: I,
        guard: &SharedRwLockReadGuard,
        device: &Device
    ) -> Self
    where
        I: Iterator<Item = &'a S>,
        S: StylesheetInDocument + 'static,
    {
        let mut cascade = Self::new();
        for stylesheet in stylesheets {
            stylesheet.effective_viewport_rules(device, guard, |rule| {
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

/// Just a helper trait to be able to implement methods on ViewportConstraints.
pub trait MaybeNew {
    /// Create a ViewportConstraints from a viewport size and a `@viewport`
    /// rule.
    fn maybe_new(device: &Device,
                 rule: &ViewportRule,
                 quirks_mode: QuirksMode)
                 -> Option<ViewportConstraints>;
}

impl MaybeNew for ViewportConstraints {
    fn maybe_new(device: &Device,
                 rule: &ViewportRule,
                 quirks_mode: QuirksMode)
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
                ViewportDescriptor::MinWidth(ref value) => min_width = Some(value),
                ViewportDescriptor::MaxWidth(ref value) => max_width = Some(value),

                ViewportDescriptor::MinHeight(ref value) => min_height = Some(value),
                ViewportDescriptor::MaxHeight(ref value) => max_height = Some(value),

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
                    (Some(a), Some(b)) => Some(a.$op(b)),
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
        let initial_viewport = device.au_viewport_size();

        let provider = get_metrics_provider_for_product();

        let default_values = device.default_computed_values();

        let context = Context {
            is_root_element: false,
            builder: StyleBuilder::for_derived_style(device, default_values, None, None),
            font_metrics_provider: &provider,
            cached_system_font: None,
            in_media_query: false,
            quirks_mode: quirks_mode,
            for_smil_animation: false,
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
                    match *$value {
                        ViewportLength::Specified(ref length) => match *length {
                            LengthOrPercentageOrAuto::Length(ref value) =>
                                Some(value.to_computed_value(&context)),
                            LengthOrPercentageOrAuto::Percentage(value) =>
                                Some(initial_viewport.$dimension.scale_by(value.0)),
                            LengthOrPercentageOrAuto::Auto => None,
                            LengthOrPercentageOrAuto::Calc(ref calc) => {
                                calc.to_computed_value(&context).to_used_value(Some(initial_viewport.$dimension))
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
            initial_zoom: PinchZoomFactor::new(initial_zoom.unwrap_or(1.)),
            min_zoom: min_zoom.map(PinchZoomFactor::new),
            max_zoom: max_zoom.map(PinchZoomFactor::new),

            user_zoom: user_zoom,
            orientation: orientation
        })
    }
}
