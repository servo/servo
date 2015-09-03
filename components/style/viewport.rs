/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, DeclarationListParser, AtRuleParser, DeclarationParser, parse_important};
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use parser::{ParserContext, log_css_error};
use properties::longhands;
use style_traits::viewport::{UserZoom, Zoom, Orientation, ViewportConstraints};
use stylesheets::Origin;
use util::geometry::{Au, ViewportPx};
use values::computed::{Context, ToComputedValue};
use values::specified::LengthOrPercentageOrAuto;


use std::ascii::AsciiExt;
use std::collections::hash_map::{Entry, HashMap};
use std::intrinsics;

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
    pub fn parse(input: &mut Parser, context: &ParserContext)
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
    let mut declarations: HashMap<u64, (usize, &'a ViewportDescriptorDeclaration)> = HashMap::new();

    // index is used to reconstruct order of appearance after all declarations
    // have been added to the map
    let mut index = 0;
    for declaration in iter {
        let descriptor = unsafe {
            intrinsics::discriminant_value(&declaration.descriptor)
        };

        match declarations.entry(descriptor) {
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

pub trait MaybeNew {
    fn maybe_new(initial_viewport: TypedSize2D<ViewportPx, f32>,
                     rule: &ViewportRule)
                     -> Option<ViewportConstraints>;
}

impl MaybeNew for ViewportConstraints {
    fn maybe_new(initial_viewport: TypedSize2D<ViewportPx, f32>,
                     rule: &ViewportRule)
                     -> Option<ViewportConstraints>
    {
        use num::{Float, ToPrimitive};
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
                    (a, None) => a.clone(),
                    (None, b) => b.clone(),
                    (a, b) => Some(a.clone().unwrap().$op(b.clone().unwrap())),
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
            max_zoom = Some(min_zoom.clone().unwrap().max(max_zoom.unwrap()))
        }

        // DEVICE-ADAPT § 6.2.2 Constrain zoom value to the [min-zoom, max-zoom] range
        if initial_zoom.is_some() {
            initial_zoom = max!(min_zoom, min!(max_zoom, initial_zoom));
        }

        // DEVICE-ADAPT § 6.2.3 Resolve non-auto lengths to pixel lengths
        //
        // Note: DEVICE-ADAPT § 5. states that relative length values are
        // resolved against initial values
        let initial_viewport = Size2D::new(Au::from_f32_px(initial_viewport.width.get()),
                                           Au::from_f32_px(initial_viewport.height.get()));


        let context = Context {
            is_root_element: false,
            viewport_size: initial_viewport,
            inherited_font_weight: longhands::font_weight::get_initial_value(),
            inherited_font_size: longhands::font_size::get_initial_value(),
            inherited_text_decorations_in_effect: longhands::_servo_text_decorations_in_effect::get_initial_value(),
            font_size: longhands::font_size::get_initial_value(),
            root_font_size: longhands::font_size::get_initial_value(),
            display: longhands::display::get_initial_value(),
            color: longhands::color::get_initial_value(),
            text_decoration: longhands::text_decoration::get_initial_value(),
            overflow_x: longhands::overflow_x::get_initial_value(),
            overflow_y: longhands::overflow_y::get_initial_value(),
            positioned: false,
            floated: false,
            border_top_present: false,
            border_right_present: false,
            border_bottom_present: false,
            border_left_present: false,
            outline_style_present: false,
        };

        macro_rules! to_pixel_length {
            ($value:ident, $dimension:ident) => {
                if let Some($value) = $value {
                    match $value {
                        LengthOrPercentageOrAuto::Length(value) =>
                            Some(value.to_computed_value(&context)),
                        LengthOrPercentageOrAuto::Percentage(value) =>
                            Some(initial_viewport.$dimension.scale_by(value.0)),
                        LengthOrPercentageOrAuto::Auto => None,
                        LengthOrPercentageOrAuto::Calc(calc) => {
                            let calc = calc.to_computed_value(&context);
                            Some(initial_viewport.$dimension.scale_by(calc.percentage()) + calc.length())
                        }
                    }
                } else {
                    None
                }
            }
        }

        let min_width = to_pixel_length!(min_width, width);
        let max_width = to_pixel_length!(max_width, width);
        let min_height = to_pixel_length!(min_height, height);
        let max_height = to_pixel_length!(max_height, height);

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
                Au::from_f32_px(height.clone().unwrap().to_f32_px() * ratio)
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
            size: Size2D::typed(width.to_f32_px(), height.to_f32_px()),

            // TODO: compute a zoom factor for 'auto' as suggested by DEVICE-ADAPT § 10.
            initial_zoom: ScaleFactor::new(initial_zoom.unwrap_or(1.)),
            min_zoom: min_zoom.map(ScaleFactor::new),
            max_zoom: max_zoom.map(ScaleFactor::new),

            user_zoom: user_zoom,
            orientation: orientation
        })
    }
}
