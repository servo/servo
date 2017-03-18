/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

use app_units::Au;
use cssparser::{Color as CSSParserColor, Parser, RGBA};
use euclid::{Point2D, Size2D};
#[cfg(feature = "gecko")] use gecko_bindings::structs::nsCSSPropertyID;
use properties::{CSSWideKeyword, PropertyDeclaration};
use properties::longhands;
use properties::longhands::background_size::computed_value::T as BackgroundSizeList;
use properties::longhands::font_weight::computed_value::T as FontWeight;
use properties::longhands::line_height::computed_value::T as LineHeight;
use properties::longhands::text_shadow::computed_value::T as TextShadowList;
use properties::longhands::text_shadow::computed_value::TextShadow;
use properties::longhands::box_shadow::computed_value::T as BoxShadowList;
use properties::longhands::box_shadow::single_value::computed_value::T as BoxShadow;
use properties::longhands::transform::computed_value::ComputedMatrix;
use properties::longhands::transform::computed_value::ComputedOperation as TransformOperation;
use properties::longhands::transform::computed_value::T as TransformList;
use properties::longhands::vertical_align::computed_value::T as VerticalAlign;
use properties::longhands::visibility::computed_value::T as Visibility;
#[cfg(feature = "gecko")] use properties::{PropertyDeclarationId, LonghandId};
use std::cmp;
#[cfg(feature = "gecko")] use std::collections::HashMap;
use std::fmt;
use style_traits::ToCss;
use super::ComputedValues;
use values::CSSFloat;
use values::{Auto, Either, Normal};
use values::computed::{Angle, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};
use values::computed::{BorderRadiusSize, ClipRect, LengthOrNone};
use values::computed::{CalcLengthOrPercentage, Context, LengthOrPercentage};
use values::computed::{MaxLength, MinLength};
use values::computed::position::{HorizontalPosition, Position, VerticalPosition};
use values::computed::ToComputedValue;



/// A given transition property, that is either `All`, or an animatable
/// property.
// NB: This needs to be here because it needs all the longhands generated
// beforehand.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum TransitionProperty {
    /// All, any animatable property changing should generate a transition.
    All,
    % for prop in data.longhands:
        % if prop.animatable:
            /// ${prop.name}
            ${prop.camel_case},
        % endif
    % endfor
    // Shorthand properties may or may not contain any animatable property. Either should still be
    // parsed properly.
    % for prop in data.shorthands_except_all():
        /// ${prop.name}
        ${prop.camel_case},
    % endfor
}

impl TransitionProperty {
    /// Iterates over each longhand property.
    pub fn each<F: FnMut(TransitionProperty) -> ()>(mut cb: F) {
        % for prop in data.longhands:
            % if prop.animatable:
                cb(TransitionProperty::${prop.camel_case});
            % endif
        % endfor
    }

    /// Iterates over every property that is not TransitionProperty::All, stopping and returning
    /// true when the provided callback returns true for the first time.
    pub fn any<F: FnMut(TransitionProperty) -> bool>(mut cb: F) -> bool {
        % for prop in data.longhands:
            % if prop.animatable:
                if cb(TransitionProperty::${prop.camel_case}) {
                    return true;
                }
            % endif
        % endfor
        false
    }

    /// Parse a transition-property value.
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { &try!(input.expect_ident()),
            "all" => Ok(TransitionProperty::All),
            % for prop in data.longhands:
                % if prop.animatable:
                    "${prop.name}" => Ok(TransitionProperty::${prop.camel_case}),
                % endif
            % endfor
            % for prop in data.shorthands_except_all():
                "${prop.name}" => Ok(TransitionProperty::${prop.camel_case}),
            % endfor
            _ => Err(())
        }
    }

    /// Get a transition property from a property declaration.
    pub fn from_declaration(declaration: &PropertyDeclaration) -> Option<Self> {
        use properties::LonghandId;
        match *declaration {
            % for prop in data.longhands:
                % if prop.animatable:
                    PropertyDeclaration::${prop.camel_case}(..)
                        => Some(TransitionProperty::${prop.camel_case}),
                % endif
            % endfor
            PropertyDeclaration::CSSWideKeyword(id, _) |
            PropertyDeclaration::WithVariables(id, _) => {
                match id {
                    % for prop in data.longhands:
                        % if prop.animatable:
                            LonghandId::${prop.camel_case} =>
                                Some(TransitionProperty::${prop.camel_case}),
                        % endif
                    % endfor
                    _ => None,
                }
            },
            _ => None,
        }
    }

    /// Returns true if this TransitionProperty is one of the discrete animatable properties and
    /// this TransitionProperty should be a longhand property.
    pub fn is_discrete(&self) -> bool {
        match *self {
            % for prop in data.longhands:
                % if prop.animation_type == "discrete":
                    TransitionProperty::${prop.camel_case} => true,
                % endif
            % endfor
            _ => false
        }
    }

    /// Return animatable longhands of this shorthand TransitionProperty, except for "all".
    pub fn longhands(&self) -> &'static [TransitionProperty] {
        % for prop in data.shorthands_except_all():
            static ${prop.ident.upper()}: &'static [TransitionProperty] = &[
                % for sub in prop.sub_properties:
                    % if sub.animatable:
                        TransitionProperty::${sub.camel_case},
                    % endif
                % endfor
            ];
        % endfor
        match *self {
            % for prop in data.shorthands_except_all():
                TransitionProperty::${prop.camel_case} => ${prop.ident.upper()},
            % endfor
            _ => panic!("Not allowed to call longhands() for this TransitionProperty")
        }
    }

    /// Returns true if this TransitionProperty is a shorthand.
    pub fn is_shorthand(&self) -> bool {
        match *self {
            % for prop in data.shorthands_except_all():
                TransitionProperty::${prop.camel_case} => true,
            % endfor
            _ => false
        }
    }
}

/// Returns true if this nsCSSPropertyID is one of the animatable properties.
#[cfg(feature = "gecko")]
pub fn nscsspropertyid_is_animatable(property: nsCSSPropertyID) -> bool {
    match property {
        % for prop in data.longhands:
            % if prop.animatable:
                ${helpers.to_nscsspropertyid(prop.ident)} => true,
            % endif
        % endfor
        _ => false
    }
}

impl ToCss for TransitionProperty {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            TransitionProperty::All => dest.write_str("all"),
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => dest.write_str("${prop.name}"),
                % endif
            % endfor
            % for prop in data.shorthands_except_all():
                TransitionProperty::${prop.camel_case} => dest.write_str("${prop.name}"),
            % endfor
        }
    }
}

/// Convert to nsCSSPropertyID.
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl From<TransitionProperty> for nsCSSPropertyID {
    fn from(transition_property: TransitionProperty) -> nsCSSPropertyID {
        match transition_property {
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case}
                        => ${helpers.to_nscsspropertyid(prop.ident)},
                % endif
            % endfor
            % for prop in data.shorthands_except_all():
                TransitionProperty::${prop.camel_case}
                    => ${helpers.to_nscsspropertyid(prop.ident)},
            % endfor
            TransitionProperty::All => nsCSSPropertyID::eCSSPropertyExtra_all_properties,
        }
    }
}

/// Convert nsCSSPropertyID to TransitionProperty
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl From<nsCSSPropertyID> for TransitionProperty {
    fn from(property: nsCSSPropertyID) -> TransitionProperty {
        match property {
            % for prop in data.longhands:
                % if prop.animatable:
                    ${helpers.to_nscsspropertyid(prop.ident)}
                        => TransitionProperty::${prop.camel_case},
                % endif
            % endfor
            % for prop in data.shorthands_except_all():
                ${helpers.to_nscsspropertyid(prop.ident)}
                    => TransitionProperty::${prop.camel_case},
            % endfor
            nsCSSPropertyID::eCSSPropertyExtra_all_properties => TransitionProperty::All,
            _ => panic!("Unsupported Servo transition property: {:?}", property),
        }
    }
}

/// Convert to PropertyDeclarationId.
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl<'a> From<TransitionProperty> for PropertyDeclarationId<'a> {
    fn from(transition_property: TransitionProperty) -> PropertyDeclarationId<'a> {
        match transition_property {
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case}
                        => PropertyDeclarationId::Longhand(LonghandId::${prop.camel_case}),
                % endif
            % endfor
            _ => panic!(),
        }
    }
}

/// An animated property interpolation between two computed values for that
/// property.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AnimatedProperty {
    % for prop in data.longhands:
        % if prop.animatable:
            /// ${prop.name}
            ${prop.camel_case}(longhands::${prop.ident}::computed_value::T,
                               longhands::${prop.ident}::computed_value::T),
        % endif
    % endfor
}

impl AnimatedProperty {
    /// Get the name of this property.
    pub fn name(&self) -> &'static str {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatedProperty::${prop.camel_case}(..) => "${prop.name}",
                % endif
            % endfor
        }
    }

    /// Whether this interpolation does animate, that is, whether the start and
    /// end values are different.
    pub fn does_animate(&self) -> bool {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => from != to,
                % endif
            % endfor
        }
    }

    /// Whether an animated property has the same end value as another.
    pub fn has_the_same_end_value_as(&self, other: &Self) -> bool {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable:
                    (&AnimatedProperty::${prop.camel_case}(_, ref this_end_value),
                     &AnimatedProperty::${prop.camel_case}(_, ref other_end_value)) => {
                        this_end_value == other_end_value
                    }
                % endif
            % endfor
            _ => false,
        }
    }

    /// Update `style` with the proper computed style corresponding to this
    /// animation at `progress`.
    pub fn update(&self, style: &mut ComputedValues, progress: f64) {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => {
                        // https://w3c.github.io/web-animations/#discrete-animation-type
                        % if prop.animation_type == "discrete":
                            let value = if progress < 0.5 { *from } else { *to };
                        % else:
                            let value = match from.interpolate(to, progress) {
                                Ok(value) => value,
                                Err(()) => return,
                            };
                        % endif
                        style.mutate_${prop.style_struct.ident.strip("_")}().set_${prop.ident}(value);
                    }
                % endif
            % endfor
        }
    }

    /// Get an animatable value from a transition-property, an old style, and a
    /// new style.
    pub fn from_transition_property(transition_property: &TransitionProperty,
                                    old_style: &ComputedValues,
                                    new_style: &ComputedValues)
                                    -> AnimatedProperty {
        match *transition_property {
            TransitionProperty::All => panic!("Can't use TransitionProperty::All here."),
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => {
                        AnimatedProperty::${prop.camel_case}(
                            old_style.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}(),
                            new_style.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                    }
                % endif
            % endfor
            other => panic!("Can't use TransitionProperty::{:?} here", other),
        }
    }
}

/// A collection of AnimationValue that were composed on an element.
/// This HashMap stores the values that are the last AnimationValue to be
/// composed for each TransitionProperty.
#[cfg(feature = "gecko")]
pub type AnimationValueMap = HashMap<TransitionProperty, AnimationValue>;

/// An enum to represent a single computed value belonging to an animated
/// property in order to be interpolated with another one. When interpolating,
/// both values need to belong to the same property.
///
/// This is different to AnimatedProperty in the sense that AnimatedProperty
/// also knows the final value to be used during the animation.
///
/// This is to be used in Gecko integration code.
///
/// FIXME: We need to add a path for custom properties, but that's trivial after
/// this (is a similar path to that of PropertyDeclaration).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AnimationValue {
    % for prop in data.longhands:
        % if prop.animatable:
            /// ${prop.name}
            ${prop.camel_case}(longhands::${prop.ident}::computed_value::T),
        % endif
    % endfor
}

impl AnimationValue {
    /// "Uncompute" this animation value in order to be used inside the CSS
    /// cascade.
    pub fn uncompute(&self) -> PropertyDeclaration {
        use properties::longhands;
        match *self {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimationValue::${prop.camel_case}(ref from) => {
                        PropertyDeclaration::${prop.camel_case}(
                            % if prop.boxed:
                                Box::new(longhands::${prop.ident}::SpecifiedValue::from_computed_value(from)))
                            % else:
                                longhands::${prop.ident}::SpecifiedValue::from_computed_value(from))
                            % endif
                    }
                % endif
            % endfor
        }
    }

    /// Construct an AnimationValue from a property declaration
    pub fn from_declaration(decl: &PropertyDeclaration, context: &Context, initial: &ComputedValues) -> Option<Self> {
        use error_reporting::StdoutErrorReporter;
        use properties::LonghandId;
        use properties::DeclaredValue;

        match *decl {
            % for prop in data.longhands:
            % if prop.animatable:
            PropertyDeclaration::${prop.camel_case}(ref val) => {
                Some(AnimationValue::${prop.camel_case}(val.to_computed_value(context)))
            },
            % endif
            % endfor
            PropertyDeclaration::CSSWideKeyword(id, keyword) => {
                match id {
                    // We put all the animatable properties first in the hopes
                    // that it might increase match locality.
                    % for prop in data.longhands:
                    % if prop.animatable:
                    LonghandId::${prop.camel_case} => {
                        let computed = match keyword {
                            % if not prop.style_struct.inherited:
                                CSSWideKeyword::Unset |
                            % endif
                            CSSWideKeyword::Initial => {
                                let initial_struct = initial.get_${prop.style_struct.name_lower}();
                                initial_struct.clone_${prop.ident}()
                            },
                            % if prop.style_struct.inherited:
                                CSSWideKeyword::Unset |
                            % endif
                            CSSWideKeyword::Inherit => {
                                let inherit_struct = context.inherited_style
                                                            .get_${prop.style_struct.name_lower}();
                                inherit_struct.clone_${prop.ident}()
                            },
                        };
                        Some(AnimationValue::${prop.camel_case}(computed))
                    },
                    % endif
                    % endfor
                    % for prop in data.longhands:
                    % if not prop.animatable:
                    LonghandId::${prop.camel_case} => None,
                    % endif
                    % endfor
                }
            },
            PropertyDeclaration::WithVariables(id, ref variables) => {
                let custom_props = context.style().custom_properties();
                let reporter = StdoutErrorReporter;
                match id {
                    % for prop in data.longhands:
                    % if prop.animatable:
                    LonghandId::${prop.camel_case} => {
                        let mut result = None;
                        ::properties::substitute_variables_${prop.ident}_slow(
                            &variables.css,
                            variables.first_token_type,
                            &variables.url_data,
                            variables.from_shorthand,
                            &custom_props,
                            |v| {
                                let declaration = match *v {
                                    DeclaredValue::Value(value) => {
                                        PropertyDeclaration::${prop.camel_case}(value.clone())
                                    },
                                    DeclaredValue::CSSWideKeyword(keyword) => {
                                        PropertyDeclaration::CSSWideKeyword(id, keyword)
                                    },
                                    DeclaredValue::WithVariables(_) => unreachable!(),
                                };
                                result = AnimationValue::from_declaration(&declaration, context, initial);
                            },
                            &reporter);
                        result
                    },
                    % else:
                    LonghandId::${prop.camel_case} => None,
                    % endif
                    % endfor
                }
            },
            _ => None // non animatable properties will get included because of shorthands. ignore.
        }
    }

    /// Get an AnimationValue for a TransitionProperty from a given computed values.
    pub fn from_computed_values(transition_property: &TransitionProperty,
                                computed_values: &ComputedValues)
                                -> Self {
        match *transition_property {
            TransitionProperty::All => panic!("Can't use TransitionProperty::All here."),
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => {
                        AnimationValue::${prop.camel_case}(
                            computed_values.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                    }
                % endif
            % endfor
            other => panic!("Can't use TransitionProperty::{:?} here.", other),
        }
    }
}

impl Interpolate for AnimationValue {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable:
                    (&AnimationValue::${prop.camel_case}(ref from),
                     &AnimationValue::${prop.camel_case}(ref to)) => {
                        // https://w3c.github.io/web-animations/#discrete-animation-type
                        % if prop.animation_type == "discrete":
                            if progress < 0.5 {
                                Ok(AnimationValue::${prop.camel_case}(*from))
                            } else {
                                Ok(AnimationValue::${prop.camel_case}(*to))
                            }
                        % else:
                            from.interpolate(to, progress).map(AnimationValue::${prop.camel_case})
                        % endif
                    }
                % endif
            % endfor
            _ => {
                panic!("Expected interpolation of computed values of the same \
                        property, got: {:?}, {:?}", self, other);
            }
        }
    }
}


/// A trait used to implement [interpolation][interpolated-types].
///
/// [interpolated-types]: https://drafts.csswg.org/css-transitions/#interpolated-types
pub trait Interpolate: Sized {
    /// Interpolate a value with another for a given property.
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()>;
}

/// https://drafts.csswg.org/css-transitions/#animtype-repeatable-list
pub trait RepeatableListInterpolate: Interpolate {}

impl<T: RepeatableListInterpolate> Interpolate for Vec<T> {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        use num_integer::lcm;
        let len = lcm(self.len(), other.len());
        self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(me, you)| {
            me.interpolate(you, progress)
        }).collect()
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for Au {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Au((self.0 as f64 + (other.0 as f64 - self.0 as f64) * progress).round() as i32))
    }
}

impl Interpolate for Auto {
    #[inline]
    fn interpolate(&self, _other: &Self, _progress: f64) -> Result<Self, ()> {
        Ok(Auto)
    }
}

impl Interpolate for Normal {
    #[inline]
    fn interpolate(&self, _other: &Self, _progress: f64) -> Result<Self, ()> {
        Ok(Normal)
    }
}

impl <T> Interpolate for Option<T>
    where T: Interpolate,
{
    #[inline]
    fn interpolate(&self, other: &Option<T>, progress: f64) -> Result<Option<T>, ()> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                Ok(this.interpolate(other, progress).ok())
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for f32 {
    #[inline]
    fn interpolate(&self, other: &f32, progress: f64) -> Result<Self, ()> {
        Ok(((*self as f64) + ((*other as f64) - (*self as f64)) * progress) as f32)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for f64 {
    #[inline]
    fn interpolate(&self, other: &f64, progress: f64) -> Result<Self, ()> {
        Ok(*self + (*other - *self) * progress)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-integer
impl Interpolate for i32 {
    #[inline]
    fn interpolate(&self, other: &i32, progress: f64) -> Result<Self, ()> {
        let a = *self as f64;
        let b = *other as f64;
        Ok((a + (b - a) * progress).round() as i32)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Interpolate for Angle {
    #[inline]
    fn interpolate(&self, other: &Angle, progress: f64) -> Result<Self, ()> {
        self.radians().interpolate(&other.radians(), progress).map(Angle::from_radians)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-visibility
impl Interpolate for Visibility {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (Visibility::visible, _) | (_, Visibility::visible) => {
                Ok(if progress >= 0.0 && progress <= 1.0 {
                    Visibility::visible
                } else if progress < 0.0 {
                    *self
                } else {
                    *other
                })
            }
            _ => Err(()),
        }
    }
}

impl<T: Interpolate + Copy> Interpolate for Size2D<T> {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        let width = try!(self.width.interpolate(&other.width, progress));
        let height = try!(self.height.interpolate(&other.height, progress));

        Ok(Size2D::new(width, height))
    }
}

impl<T: Interpolate + Copy> Interpolate for Point2D<T> {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        let x = try!(self.x.interpolate(&other.x, progress));
        let y = try!(self.y.interpolate(&other.y, progress));

        Ok(Point2D::new(x, y))
    }
}

impl Interpolate for BorderRadiusSize {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        self.0.interpolate(&other.0, progress).map(BorderRadiusSize)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-length
impl Interpolate for VerticalAlign {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref this)),
             VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref other))) => {
                this.interpolate(other, progress).map(|value| {
                    VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(value))
                })
            }
            _ => Err(()),
        }
    }
}

impl Interpolate for BackgroundSizeList {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        self.0.interpolate(&other.0, progress).map(BackgroundSizeList)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-color
impl Interpolate for RGBA {
    #[inline]
    fn interpolate(&self, other: &RGBA, progress: f64) -> Result<Self, ()> {
        fn clamp(val: f32) -> f32 {
            val.max(0.).min(1.)
        }

        let alpha = clamp(try!(self.alpha_f32().interpolate(&other.alpha_f32(), progress)));
        if alpha == 0. {
            Ok(RGBA::transparent())
        } else {
            // NB: We rely on RGBA::from_floats clamping already.
            let red = try!((self.red_f32() * self.alpha_f32())
                            .interpolate(&(other.red_f32() * other.alpha_f32()), progress))
                            * 1. / alpha;
            let green = try!((self.green_f32() * self.alpha_f32())
                             .interpolate(&(other.green_f32() * other.alpha_f32()), progress))
                             * 1. / alpha;
            let blue = try!((self.blue_f32() * self.alpha_f32())
                             .interpolate(&(other.blue_f32() * other.alpha_f32()), progress))
                             * 1. / alpha;
            Ok(RGBA::from_floats(red, green, blue, alpha))
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-color
impl Interpolate for CSSParserColor {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (CSSParserColor::RGBA(ref this), CSSParserColor::RGBA(ref other)) => {
                this.interpolate(other, progress).map(CSSParserColor::RGBA)
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for CalcLengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        fn interpolate_half<T>(this: Option<T>,
                               other: Option<T>,
                               progress: f64)
                               -> Result<Option<T>, ()>
            where T: Default + Interpolate,
        {
            match (this, other) {
                (None, None) => Ok(None),
                (this, other) => {
                    let this = this.unwrap_or(T::default());
                    let other = other.unwrap_or(T::default());
                    this.interpolate(&other, progress).map(Some)
                }
            }
        }

        Ok(CalcLengthOrPercentage {
            length: try!(self.length.interpolate(&other.length, progress)),
            percentage: try!(interpolate_half(self.percentage, other.percentage, progress)),
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for LengthOrPercentage {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                this.interpolate(other, progress).map(LengthOrPercentage::Length)
            }
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                this.interpolate(other, progress).map(LengthOrPercentage::Percentage)
            }
            (this, other) => {
                let this: CalcLengthOrPercentage = From::from(this);
                let other: CalcLengthOrPercentage = From::from(other);
                this.interpolate(&other, progress)
                    .map(LengthOrPercentage::Calc)
            }
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for LengthOrPercentageOrAuto {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.interpolate(other, progress).map(LengthOrPercentageOrAuto::Length)
            }
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.interpolate(other, progress).map(LengthOrPercentageOrAuto::Percentage)
            }
            (LengthOrPercentageOrAuto::Auto, LengthOrPercentageOrAuto::Auto) => {
                Ok(LengthOrPercentageOrAuto::Auto)
            }
            (this, other) => {
                let this: Option<CalcLengthOrPercentage> = From::from(this);
                let other: Option<CalcLengthOrPercentage> = From::from(other);
                match this.interpolate(&other, progress) {
                    Ok(Some(result)) => Ok(LengthOrPercentageOrAuto::Calc(result)),
                    _ => Err(()),
                }
            }
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for LengthOrPercentageOrNone {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrNone::Length(ref this),
             LengthOrPercentageOrNone::Length(ref other)) => {
                this.interpolate(other, progress).map(LengthOrPercentageOrNone::Length)
            }
            (LengthOrPercentageOrNone::Percentage(ref this),
             LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.interpolate(other, progress).map(LengthOrPercentageOrNone::Percentage)
            }
            (LengthOrPercentageOrNone::None, LengthOrPercentageOrNone::None) => {
                Ok(LengthOrPercentageOrNone::None)
            }
            _ => Err(())
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for MinLength {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (MinLength::LengthOrPercentage(ref this),
             MinLength::LengthOrPercentage(ref other)) => {
                this.interpolate(other, progress).map(MinLength::LengthOrPercentage)
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Interpolate for MaxLength {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (MaxLength::LengthOrPercentage(ref this),
             MaxLength::LengthOrPercentage(ref other)) => {
                this.interpolate(other, progress).map(MaxLength::LengthOrPercentage)
            }
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
/// https://drafts.csswg.org/css-transitions/#animtype-length
impl Interpolate for LineHeight {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LineHeight::Length(ref this),
             LineHeight::Length(ref other)) => {
                this.interpolate(other, progress).map(LineHeight::Length)
            }
            (LineHeight::Number(ref this),
             LineHeight::Number(ref other)) => {
                this.interpolate(other, progress).map(LineHeight::Number)
            }
            (LineHeight::Normal, LineHeight::Normal) => {
                Ok(LineHeight::Normal)
            }
            _ => Err(()),
        }
    }
}

/// http://dev.w3.org/csswg/css-transitions/#animtype-font-weight
impl Interpolate for FontWeight {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        let a = (*self as u32) as f64;
        let b = (*other as u32) as f64;
        let weight = a + (b - a) * progress;
        Ok(if weight < 150. {
            FontWeight::Weight100
        } else if weight < 250. {
            FontWeight::Weight200
        } else if weight < 350. {
            FontWeight::Weight300
        } else if weight < 450. {
            FontWeight::Weight400
        } else if weight < 550. {
            FontWeight::Weight500
        } else if weight < 650. {
            FontWeight::Weight600
        } else if weight < 750. {
            FontWeight::Weight700
        } else if weight < 850. {
            FontWeight::Weight800
        } else {
            FontWeight::Weight900
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-simple-list
impl Interpolate for Position {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Position {
            horizontal: try!(self.horizontal.interpolate(&other.horizontal, progress)),
            vertical: try!(self.vertical.interpolate(&other.vertical, progress)),
        })
    }
}

impl RepeatableListInterpolate for Position {}

/// https://drafts.csswg.org/css-transitions/#animtype-simple-list
impl Interpolate for HorizontalPosition {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(HorizontalPosition(try!(self.0.interpolate(&other.0, progress))))
    }
}

impl RepeatableListInterpolate for HorizontalPosition {}

/// https://drafts.csswg.org/css-transitions/#animtype-simple-list
impl Interpolate for VerticalPosition {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(VerticalPosition(try!(self.0.interpolate(&other.0, progress))))
    }
}

impl RepeatableListInterpolate for VerticalPosition {}

/// https://drafts.csswg.org/css-transitions/#animtype-rect
impl Interpolate for ClipRect {
    #[inline]
    fn interpolate(&self, other: &Self, time: f64) -> Result<Self, ()> {
        Ok(ClipRect {
            top: try!(self.top.interpolate(&other.top, time)),
            right: try!(self.right.interpolate(&other.right, time)),
            bottom: try!(self.bottom.interpolate(&other.bottom, time)),
            left: try!(self.left.interpolate(&other.left, time)),
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Interpolate for TextShadow {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(TextShadow {
            offset_x: try!(self.offset_x.interpolate(&other.offset_x, progress)),
            offset_y: try!(self.offset_y.interpolate(&other.offset_y, progress)),
            blur_radius: try!(self.blur_radius.interpolate(&other.blur_radius, progress)),
            color: try!(self.color.interpolate(&other.color, progress)),
        })
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Interpolate for TextShadowList {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        let zero = TextShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA::transparent()),
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let shadow = match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other))
                    => try!(shadow.interpolate(other, progress)),
                (Some(shadow), None) => {
                    shadow.interpolate(&zero, progress).unwrap()
                }
                (None, Some(shadow)) => {
                    zero.interpolate(&shadow, progress).unwrap()
                }
                (None, None) => unreachable!(),
            };
            result.push(shadow);
        }

        Ok(TextShadowList(result))
    }
}


impl Interpolate for BoxShadowList {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        // The inset value must change
        let mut zero = BoxShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            spread_radius: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA::transparent()),
            inset: false,
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut result = Vec::with_capacity(max_len);

        for i in 0..max_len {
            let shadow = match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other))
                    => try!(shadow.interpolate(other, progress)),
                (Some(shadow), None) => {
                    zero.inset = shadow.inset;
                    shadow.interpolate(&zero, progress).unwrap()
                }
                (None, Some(shadow)) => {
                    zero.inset = shadow.inset;
                    zero.interpolate(&shadow, progress).unwrap()
                }
                (None, None) => unreachable!(),
            };
            result.push(shadow);
        }

        Ok(BoxShadowList(result))
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
impl Interpolate for BoxShadow {
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        if self.inset != other.inset {
            return Err(());
        }

        let x = try!(self.offset_x.interpolate(&other.offset_x, progress));
        let y = try!(self.offset_y.interpolate(&other.offset_y, progress));
        let color = try!(self.color.interpolate(&other.color, progress));
        let spread = try!(self.spread_radius.interpolate(&other.spread_radius, progress));
        let blur = try!(self.blur_radius.interpolate(&other.blur_radius, progress));

        Ok(BoxShadow {
            offset_x: x,
            offset_y: y,
            blur_radius: blur,
            spread_radius: spread,
            color: color,
            inset: self.inset,
        })
    }
}

impl Interpolate for LengthOrNone {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (Either::First(ref length), Either::First(ref other)) =>
                length.interpolate(&other, progress).map(Either::First),
            _ => Err(()),
        }
    }
}

/// Check if it's possible to do a direct numerical interpolation
/// between these two transform lists.
/// http://dev.w3.org/csswg/css-transforms/#transform-transform-animation
fn can_interpolate_list(from_list: &[TransformOperation],
                        to_list: &[TransformOperation]) -> bool {
    // Lists must be equal length
    if from_list.len() != to_list.len() {
        return false;
    }

    // Each transform operation must match primitive type in other list
    for (from, to) in from_list.iter().zip(to_list) {
        match (from, to) {
            (&TransformOperation::Matrix(..), &TransformOperation::Matrix(..)) |
            (&TransformOperation::Skew(..), &TransformOperation::Skew(..)) |
            (&TransformOperation::Translate(..), &TransformOperation::Translate(..)) |
            (&TransformOperation::Scale(..), &TransformOperation::Scale(..)) |
            (&TransformOperation::Rotate(..), &TransformOperation::Rotate(..)) |
            (&TransformOperation::Perspective(..), &TransformOperation::Perspective(..)) => {}
            _ => {
                return false;
            }
        }
    }

    true
}

/// Build an equivalent 'identity transform function list' based
/// on an existing transform list.
/// http://dev.w3.org/csswg/css-transforms/#none-transform-animation
fn build_identity_transform_list(list: &[TransformOperation]) -> Vec<TransformOperation> {
    let mut result = vec!();

    for operation in list {
        match *operation {
            TransformOperation::Matrix(..) => {
                let identity = ComputedMatrix::identity();
                result.push(TransformOperation::Matrix(identity));
            }
            TransformOperation::MatrixWithPercents(..) => {}
            TransformOperation::Skew(..) => {
                result.push(TransformOperation::Skew(Angle::zero(), Angle::zero()))
            }
            TransformOperation::Translate(..) => {
                result.push(TransformOperation::Translate(LengthOrPercentage::zero(),
                                                          LengthOrPercentage::zero(),
                                                          Au(0)));
            }
            TransformOperation::Scale(..) => {
                result.push(TransformOperation::Scale(1.0, 1.0, 1.0));
            }
            TransformOperation::Rotate(..) => {
                result.push(TransformOperation::Rotate(0.0, 0.0, 1.0, Angle::zero()));
            }
            TransformOperation::Perspective(..) => {
                // http://dev.w3.org/csswg/css-transforms/#identity-transform-function
                let identity = ComputedMatrix::identity();
                result.push(TransformOperation::Matrix(identity));
            }
        }
    }

    result
}

/// Interpolate two transform lists.
/// http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
fn interpolate_transform_list(from_list: &[TransformOperation],
                              to_list: &[TransformOperation],
                              progress: f64) -> TransformList {
    let mut result = vec![];

    if can_interpolate_list(from_list, to_list) {
        for (from, to) in from_list.iter().zip(to_list) {
            match (from, to) {
                (&TransformOperation::Matrix(from),
                 &TransformOperation::Matrix(_to)) => {
                    let interpolated = from.interpolate(&_to, progress).unwrap();
                    result.push(TransformOperation::Matrix(interpolated));
                }
                (&TransformOperation::MatrixWithPercents(_),
                 &TransformOperation::MatrixWithPercents(_)) => {
                    // We don't interpolate `-moz-transform` matrices yet.
                    // They contain percentage values.
                    {}
                }
                (&TransformOperation::Skew(fx, fy),
                 &TransformOperation::Skew(tx, ty)) => {
                    let ix = fx.interpolate(&tx, progress).unwrap();
                    let iy = fy.interpolate(&ty, progress).unwrap();
                    result.push(TransformOperation::Skew(ix, iy));
                }
                (&TransformOperation::Translate(fx, fy, fz),
                 &TransformOperation::Translate(tx, ty, tz)) => {
                    let ix = fx.interpolate(&tx, progress).unwrap();
                    let iy = fy.interpolate(&ty, progress).unwrap();
                    let iz = fz.interpolate(&tz, progress).unwrap();
                    result.push(TransformOperation::Translate(ix, iy, iz));
                }
                (&TransformOperation::Scale(fx, fy, fz),
                 &TransformOperation::Scale(tx, ty, tz)) => {
                    let ix = fx.interpolate(&tx, progress).unwrap();
                    let iy = fy.interpolate(&ty, progress).unwrap();
                    let iz = fz.interpolate(&tz, progress).unwrap();
                    result.push(TransformOperation::Scale(ix, iy, iz));
                }
                (&TransformOperation::Rotate(fx, fy, fz, fa),
                 &TransformOperation::Rotate(tx, ty, tz, ta)) => {
                    let norm_f = ((fx * fx) + (fy * fy) + (fz * fz)).sqrt();
                    let norm_t = ((tx * tx) + (ty * ty) + (tz * tz)).sqrt();
                    let (fx, fy, fz) = (fx / norm_f, fy / norm_f, fz / norm_f);
                    let (tx, ty, tz) = (tx / norm_t, ty / norm_t, tz / norm_t);
                    if fx == tx && fy == ty && fz == tz {
                        let ia = fa.interpolate(&ta, progress).unwrap();
                        result.push(TransformOperation::Rotate(fx, fy, fz, ia));
                    } else {
                        let matrix_f = rotate_to_matrix(fx, fy, fz, fa);
                        let matrix_t = rotate_to_matrix(tx, ty, tz, ta);
                        let interpolated = matrix_f.interpolate(&matrix_t, progress).unwrap();

                        result.push(TransformOperation::Matrix(interpolated));
                    }
                }
                (&TransformOperation::Perspective(fd),
                 &TransformOperation::Perspective(_td)) => {
                    let mut fd_matrix = ComputedMatrix::identity();
                    let mut td_matrix = ComputedMatrix::identity();
                    fd_matrix.m43 = -1. / fd.to_f32_px();
                    td_matrix.m43 = -1. / _td.to_f32_px();
                    let interpolated = fd_matrix.interpolate(&td_matrix, progress).unwrap();
                    result.push(TransformOperation::Matrix(interpolated));
                }
                _ => {
                    // This should be unreachable due to the can_interpolate_list() call.
                    unreachable!();
                }
            }
        }
    } else {
        // TODO(gw): Implement matrix decomposition and interpolation
        result.extend_from_slice(from_list);
    }

    TransformList(Some(result))
}

/// https://drafts.csswg.org/css-transforms/#Rotate3dDefined
fn rotate_to_matrix(x: f32, y: f32, z: f32, a: Angle) -> ComputedMatrix {
    let half_rad = a.radians() / 2.0;
    let sc = (half_rad).sin() * (half_rad).cos();
    let sq = (half_rad).sin().powi(2);

    ComputedMatrix {
        m11: 1.0 - 2.0 * (y * y + z * z) * sq,
        m12: 2.0 * (x * y * sq - z * sc),
        m13: 2.0 * (x * z * sq + y * sc),
        m14: 0.0,

        m21: 2.0 * (x * y * sq + z * sc),
        m22: 1.0 - 2.0 * (x * x + z * z) * sq,
        m23: 2.0 * (y * z * sq - x * sc),
        m24: 0.0,

        m31: 2.0 * (x * z * sq - y * sc),
        m32: 2.0 * (y * z * sq + x * sc),
        m33: 1.0 - 2.0 * (x * x + y * y) * sq,
        m34: 0.0,

        m41: 0.0,
        m42: 0.0,
        m43: 0.0,
        m44: 1.0
    }
}

/// A 2d matrix for interpolation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct InnerMatrix2D {
    pub m11: CSSFloat, pub m12: CSSFloat,
    pub m21: CSSFloat, pub m22: CSSFloat,
}

/// A 2d translation function.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Translate2D(f32, f32);

/// A 2d scale function.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Scale2D(f32, f32);

/// A decomposed 2d matrix.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MatrixDecomposed2D {
    /// The translation function.
    pub translate: Translate2D,
    /// The scale function.
    pub scale: Scale2D,
    /// The rotation angle.
    pub angle: f32,
    /// The inner matrix.
    pub matrix: InnerMatrix2D,
}

impl Interpolate for InnerMatrix2D {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(InnerMatrix2D {
            m11: try!(self.m11.interpolate(&other.m11, progress)),
            m12: try!(self.m12.interpolate(&other.m12, progress)),
            m21: try!(self.m21.interpolate(&other.m21, progress)),
            m22: try!(self.m22.interpolate(&other.m22, progress)),
        })
    }
}

impl Interpolate for Translate2D {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Translate2D(
            try!(self.0.interpolate(&other.0, progress)),
            try!(self.1.interpolate(&other.1, progress))
        ))
    }
}

impl Interpolate for Scale2D {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Scale2D(
            try!(self.0.interpolate(&other.0, progress)),
            try!(self.1.interpolate(&other.1, progress))
        ))
    }
}

impl Interpolate for MatrixDecomposed2D {
    /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-2d-matrix-values
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        // If x-axis of one is flipped, and y-axis of the other,
        // convert to an unflipped rotation.
        let mut scale = self.scale;
        let mut angle = self.angle;
        let mut other_angle = other.angle;
        if (scale.0 < 0.0 && other.scale.1 < 0.0) || (scale.1 < 0.0 && other.scale.0 < 0.0) {
            scale.0 = -scale.0;
            scale.1 = -scale.1;
            angle += if angle < 0.0 {180.} else {-180.};
        }

        // Don't rotate the long way around.
        if angle == 0.0 {
            angle = 360.
        }
        if other_angle == 0.0 {
            other_angle = 360.
        }

        if (angle - other_angle).abs() > 180. {
            if angle > other_angle {
                angle -= 360.
            }
            else{
                other_angle -= 360.
            }
        }

        // Interpolate all values.
        let translate = try!(self.translate.interpolate(&other.translate, progress));
        let scale = try!(scale.interpolate(&other.scale, progress));
        let angle = try!(angle.interpolate(&other_angle, progress));
        let matrix = try!(self.matrix.interpolate(&other.matrix, progress));

        Ok(MatrixDecomposed2D {
            translate: translate,
            scale: scale,
            angle: angle,
            matrix: matrix,
        })
    }
}

impl Interpolate for ComputedMatrix {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        if self.is_3d() || other.is_3d() {
            let decomposed_from = decompose_3d_matrix(*self);
            let decomposed_to = decompose_3d_matrix(*other);
            match (decomposed_from, decomposed_to) {
                (Ok(from), Ok(to)) => {
                    let interpolated = try!(from.interpolate(&to, progress));
                    Ok(ComputedMatrix::from(interpolated))
                },
                _ => {
                    let interpolated = if progress < 0.5 {*self} else {*other};
                    Ok(interpolated)
                }
            }
        } else {
            let decomposed_from = MatrixDecomposed2D::from(*self);
            let decomposed_to = MatrixDecomposed2D::from(*other);
            let interpolated = try!(decomposed_from.interpolate(&decomposed_to, progress));
            Ok(ComputedMatrix::from(interpolated))
        }
    }
}

impl From<ComputedMatrix> for MatrixDecomposed2D {
    /// Decompose a 2D matrix.
    /// https://drafts.csswg.org/css-transforms/#decomposing-a-2d-matrix
    fn from(matrix: ComputedMatrix) -> MatrixDecomposed2D {
        let mut row0x = matrix.m11;
        let mut row0y = matrix.m12;
        let mut row1x = matrix.m21;
        let mut row1y = matrix.m22;

        let translate = Translate2D(matrix.m41, matrix.m42);
        let mut scale = Scale2D((row0x * row0x + row0y * row0y).sqrt(),
                                (row1x * row1x + row1y * row1y).sqrt());

        // If determinant is negative, one axis was flipped.
        let determinant = row0x * row1y - row0y * row1x;
        if determinant < 0. {
            if row0x < row1y {
                scale.0 = -scale.0;
            } else {
                scale.1 = -scale.1;
            }
        }

        // Renormalize matrix to remove scale.
        if scale.0 != 0.0 {
            row0x *= 1. / scale.0;
            row0y *= 1. / scale.0;
        }
        if scale.1 != 0.0 {
            row1x *= 1. / scale.1;
            row1y *= 1. / scale.1;
        }

        // Compute rotation and renormalize matrix.
        let mut angle = row0y.atan2(row0x);
        if angle != 0.0 {
            let sn = -row0y;
            let cs = row0x;
            let m11 = row0x;
            let m12 = row0y;
            let m21 = row1x;
            let m22 = row1y;
            row0x = cs * m11 + sn * m21;
            row0y = cs * m12 + sn * m22;
            row1x = -sn * m11 + cs * m21;
            row1y = -sn * m12 + cs * m22;
        }

        let m = InnerMatrix2D {
            m11: row0x, m12: row0y,
            m21: row1x, m22: row1y,
        };

        // Convert into degrees because our rotation functions expect it.
        angle = angle.to_degrees();
        MatrixDecomposed2D {
            translate: translate,
            scale: scale,
            angle: angle,
            matrix: m,
        }
    }
}

impl From<MatrixDecomposed2D> for ComputedMatrix {
    /// Recompose a 2D matrix.
    /// https://drafts.csswg.org/css-transforms/#recomposing-to-a-2d-matrix
    fn from(decomposed: MatrixDecomposed2D) -> ComputedMatrix {
        let mut computed_matrix = ComputedMatrix::identity();
        computed_matrix.m11 = decomposed.matrix.m11;
        computed_matrix.m12 = decomposed.matrix.m12;
        computed_matrix.m21 = decomposed.matrix.m21;
        computed_matrix.m22 = decomposed.matrix.m22;

        // Translate matrix.
        computed_matrix.m41 = decomposed.translate.0;
        computed_matrix.m42 = decomposed.translate.1;

        // Rotate matrix.
        let angle = decomposed.angle.to_radians();
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        let mut rotate_matrix = ComputedMatrix::identity();
        rotate_matrix.m11 = cos_angle;
        rotate_matrix.m12 = sin_angle;
        rotate_matrix.m21 = -sin_angle;
        rotate_matrix.m22 = cos_angle;

        // Multiplication of computed_matrix and rotate_matrix
        computed_matrix = multiply(rotate_matrix, computed_matrix);

        // Scale matrix.
        computed_matrix.m11 *= decomposed.scale.0;
        computed_matrix.m12 *= decomposed.scale.0;
        computed_matrix.m21 *= decomposed.scale.1;
        computed_matrix.m22 *= decomposed.scale.1;
        computed_matrix
    }
}

/// A 3d translation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Translate3D(f32, f32, f32);

/// A 3d scale function.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Scale3D(f32, f32, f32);

/// A 3d skew function.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Skew(f32, f32, f32);

/// A 3d perspective transformation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Perspective(f32, f32, f32, f32);

/// A quaternion used to represent a rotation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Quaternion(f32, f32, f32, f32);

/// A decomposed 3d matrix.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct MatrixDecomposed3D {
    /// A translation function.
    pub translate: Translate3D,
    /// A scale function.
    pub scale: Scale3D,
    /// The skew component of the transformation.
    pub skew: Skew,
    /// The perspective component of the transformation.
    pub perspective: Perspective,
    /// The quaternion used to represent the rotation.
    pub quaternion: Quaternion,
}

/// Decompose a 3D matrix.
/// https://drafts.csswg.org/css-transforms/#decomposing-a-3d-matrix
fn decompose_3d_matrix(mut matrix: ComputedMatrix) -> Result<MatrixDecomposed3D, ()> {
    // Normalize the matrix.
    if matrix.m44 == 0.0 {
        return Err(());
    }

    let scaling_factor = matrix.m44;
    % for i in range(1, 5):
        % for j in range(1, 5):
            matrix.m${i}${j} /= scaling_factor;
        % endfor
    % endfor

    // perspective_matrix is used to solve for perspective, but it also provides
    // an easy way to test for singularity of the upper 3x3 component.
    let mut perspective_matrix = matrix;

    % for i in range(1, 4):
        perspective_matrix.m${i}4 = 0.0;
    % endfor
    perspective_matrix.m44 = 1.0;

    if perspective_matrix.determinant() == 0.0 {
        return Err(());
    }

    // First, isolate perspective.
    let perspective = if matrix.m14 != 0.0 || matrix.m24 != 0.0 || matrix.m34 != 0.0 {
        let right_hand_side: [f32; 4] = [
            matrix.m14,
            matrix.m24,
            matrix.m34,
            matrix.m44
        ];

        perspective_matrix = perspective_matrix.inverse().unwrap();

        // Transpose perspective_matrix
        perspective_matrix = ComputedMatrix {
            % for i in range(1, 5):
                % for j in range(1, 5):
                    m${i}${j}: perspective_matrix.m${j}${i},
                % endfor
            % endfor
        };

        // Multiply right_hand_side with perspective_matrix
        let mut tmp: [f32; 4] = [0.0; 4];
        % for i in range(1, 5):
            tmp[${i - 1}] = (right_hand_side[0] * perspective_matrix.m1${i}) +
                            (right_hand_side[1] * perspective_matrix.m2${i}) +
                            (right_hand_side[2] * perspective_matrix.m3${i}) +
                            (right_hand_side[3] * perspective_matrix.m4${i});
        % endfor

        Perspective(tmp[0], tmp[1], tmp[2], tmp[3])
    } else {
        Perspective(0.0, 0.0, 0.0, 1.0)
    };

    // Next take care of translation
    let translate = Translate3D (
        matrix.m41,
        matrix.m42,
        matrix.m43
    );

    // Now get scale and shear. 'row' is a 3 element array of 3 component vectors
    let mut row: [[f32; 3]; 3] = [[0.0; 3]; 3];
    % for i in range(1, 4):
        row[${i - 1}][0] = matrix.m${i}1;
        row[${i - 1}][1] = matrix.m${i}2;
        row[${i - 1}][2] = matrix.m${i}3;
    % endfor

    // Compute X scale factor and normalize first row.
    let row0len = (row[0][0] * row[0][0] + row[0][1] * row[0][1] + row[0][2] * row[0][2]).sqrt();
    let mut scale = Scale3D(row0len, 0.0, 0.0);
    row[0] = [row[0][0] / row0len, row[0][1] / row0len, row[0][2] / row0len];

    // Compute XY shear factor and make 2nd row orthogonal to 1st.
    let mut skew = Skew(dot(row[0], row[1]), 0.0, 0.0);
    row[1] = combine(row[1], row[0], 1.0, -skew.0);

    // Now, compute Y scale and normalize 2nd row.
    let row1len = (row[0][0] * row[0][0] + row[0][1] * row[0][1] + row[0][2] * row[0][2]).sqrt();
    scale.1 = row1len;
    row[1] = [row[1][0] / row1len, row[1][1] / row1len, row[1][2] / row1len];
    skew.0 /= scale.1;

    // Compute XZ and YZ shears, orthogonalize 3rd row
    skew.1 = dot(row[0], row[2]);
    row[2] = combine(row[2], row[0], 1.0, -skew.1);
    skew.2 = dot(row[1], row[2]);
    row[2] = combine(row[2], row[1], 1.0, -skew.2);

    // Next, get Z scale and normalize 3rd row.
    let row2len = (row[2][0] * row[2][0] + row[2][1] * row[2][1] + row[2][2] * row[2][2]).sqrt();
    scale.2 = row2len;
    row[2] = [row[2][0] / row2len, row[2][1] / row2len, row[2][2] / row2len];
    skew.1 /= scale.2;
    skew.2 /= scale.2;

    // At this point, the matrix (in rows) is orthonormal.
    // Check for a coordinate system flip.  If the determinant
    // is -1, then negate the matrix and the scaling factors.
    let pdum3 = cross(row[1], row[2]);
    if dot(row[0], pdum3) < 0.0 {
        % for i in range(3):
            scale.${i} *= -1.0;
            row[${i}][0] *= -1.0;
            row[${i}][1] *= -1.0;
            row[${i}][2] *= -1.0;
        % endfor
    }

    // Now, get the rotations out
    let mut quaternion = Quaternion (
        0.5 * ((1.0 + row[0][0] - row[1][1] - row[2][2]).max(0.0)).sqrt(),
        0.5 * ((1.0 - row[0][0] + row[1][1] - row[2][2]).max(0.0)).sqrt(),
        0.5 * ((1.0 - row[0][0] - row[1][1] + row[2][2]).max(0.0)).sqrt(),
        0.5 * ((1.0 + row[0][0] + row[1][1] + row[2][2]).max(0.0)).sqrt()
    );

    if row[2][1] > row[1][2] {
        quaternion.0 = -quaternion.0
    }
    if row[0][2] > row[2][0] {
        quaternion.1 = -quaternion.1
    }
    if row[1][0] > row[0][1] {
        quaternion.2 = -quaternion.2
    }

    Ok(MatrixDecomposed3D {
        translate: translate,
        scale: scale,
        skew: skew,
        perspective: perspective,
        quaternion: quaternion
    })
}

// Combine 2 point.
fn combine(a: [f32; 3], b: [f32; 3], ascl: f32, bscl: f32) -> [f32; 3] {
    [
        (ascl * a[0]) + (bscl * b[0]),
        (ascl * a[1]) + (bscl * b[1]),
        (ascl * a[2]) + (bscl * b[2])
    ]
}

// Dot product.
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// Cross product.
fn cross(row1: [f32; 3], row2: [f32; 3]) -> [f32; 3] {
    [
        row1[1] * row2[2] - row1[2] * row2[1],
        row1[2] * row2[0] - row1[0] * row2[2],
        row1[0] * row2[1] - row1[1] * row2[0]
    ]
}

impl Interpolate for Translate3D {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Translate3D(
            try!(self.0.interpolate(&other.0, progress)),
            try!(self.1.interpolate(&other.1, progress)),
            try!(self.2.interpolate(&other.2, progress))
        ))
    }
}

impl Interpolate for Scale3D {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Scale3D(
            try!(self.0.interpolate(&other.0, progress)),
            try!(self.1.interpolate(&other.1, progress)),
            try!(self.2.interpolate(&other.2, progress))
        ))
    }
}

impl Interpolate for Skew {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Skew(
            try!(self.0.interpolate(&other.0, progress)),
            try!(self.1.interpolate(&other.1, progress)),
            try!(self.2.interpolate(&other.2, progress))
        ))
    }
}

impl Interpolate for Perspective {
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        Ok(Perspective(
            try!(self.0.interpolate(&other.0, progress)),
            try!(self.1.interpolate(&other.1, progress)),
            try!(self.2.interpolate(&other.2, progress)),
            try!(self.3.interpolate(&other.3, progress))
        ))
    }
}

impl Interpolate for MatrixDecomposed3D {
    /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-3d-matrix-values
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        let mut interpolated = *self;

        // Interpolate translate, scale, skew and perspective components.
        interpolated.translate = try!(self.translate.interpolate(&other.translate, progress));
        interpolated.scale = try!(self.scale.interpolate(&other.scale, progress));
        interpolated.skew = try!(self.skew.interpolate(&other.skew, progress));
        interpolated.perspective = try!(self.perspective.interpolate(&other.perspective, progress));

        // Interpolate quaternions using spherical linear interpolation (Slerp).
        let mut product = self.quaternion.0 * other.quaternion.0 +
                          self.quaternion.1 * other.quaternion.1 +
                          self.quaternion.2 * other.quaternion.2 +
                          self.quaternion.3 * other.quaternion.3;

        // Clamp product to -1.0 <= product <= 1.0
        product = product.min(1.0);
        product = product.max(-1.0);

        if product == 1.0 {
            return Ok(interpolated);
        }

        let theta = product.acos();
        let w = (progress as f32 * theta).sin() * 1.0 / (1.0 - product * product).sqrt();

        let mut a = *self;
        let mut b = *other;
        % for i in range(4):
            a.quaternion.${i} *= (progress as f32 * theta).cos() - product * w;
            b.quaternion.${i} *= w;
            interpolated.quaternion.${i} = a.quaternion.${i} + b.quaternion.${i};
        % endfor

        Ok(interpolated)
    }
}

impl From<MatrixDecomposed3D> for ComputedMatrix {
    /// Recompose a 3D matrix.
    /// https://drafts.csswg.org/css-transforms/#recomposing-to-a-3d-matrix
    fn from(decomposed: MatrixDecomposed3D) -> ComputedMatrix {
        let mut matrix = ComputedMatrix::identity();

        // Apply perspective
        % for i in range(1, 5):
            matrix.m${i}4 = decomposed.perspective.${i - 1};
        % endfor

        // Apply translation
        % for i in range(1, 4):
            % for j in range(1, 4):
                matrix.m4${i} += decomposed.translate.${j - 1} * matrix.m${j}${i};
            % endfor
        % endfor

        // Apply rotation
        let x = decomposed.quaternion.0;
        let y = decomposed.quaternion.1;
        let z = decomposed.quaternion.2;
        let w = decomposed.quaternion.3;

        // Construct a composite rotation matrix from the quaternion values
        // rotationMatrix is a identity 4x4 matrix initially
        let mut rotation_matrix = ComputedMatrix::identity();
        rotation_matrix.m11 = 1.0 - 2.0 * (y * y + z * z);
        rotation_matrix.m12 = 2.0 * (x * y + z * w);
        rotation_matrix.m13 = 2.0 * (x * z - y * w);
        rotation_matrix.m21 = 2.0 * (x * y - z * w);
        rotation_matrix.m22 = 1.0 - 2.0 * (x * x + z * z);
        rotation_matrix.m23 = 2.0 * (y * z + x * w);
        rotation_matrix.m31 = 2.0 * (x * z + y * w);
        rotation_matrix.m32 = 2.0 * (y * z - x * w);
        rotation_matrix.m33 = 1.0 - 2.0 * (x * x + y * y);

        matrix = multiply(rotation_matrix, matrix);

        // Apply skew
        let mut temp = ComputedMatrix::identity();
        if decomposed.skew.2 != 0.0 {
            temp.m32 = decomposed.skew.2;
            matrix = multiply(matrix, temp);
        }

        if decomposed.skew.1 != 0.0 {
            temp.m32 = 0.0;
            temp.m31 = decomposed.skew.1;
            matrix = multiply(matrix, temp);
        }

        if decomposed.skew.0 != 0.0 {
            temp.m31 = 0.0;
            temp.m21 = decomposed.skew.0;
            matrix = multiply(matrix, temp);
        }

        // Apply scale
        % for i in range(1, 4):
            % for j in range(1, 4):
                matrix.m${i}${j} *= decomposed.scale.${i - 1};
            % endfor
        % endfor

        matrix
    }
}

// Multiplication of two 4x4 matrices.
fn multiply(a: ComputedMatrix, b: ComputedMatrix) -> ComputedMatrix {
    let mut a_clone = a;
    % for i in range(1, 5):
        % for j in range(1, 5):
            a_clone.m${i}${j} = (a.m${i}1 * b.m1${j}) +
                               (a.m${i}2 * b.m2${j}) +
                               (a.m${i}3 * b.m3${j}) +
                               (a.m${i}4 * b.m4${j});
        % endfor
    % endfor
    a_clone
}

impl ComputedMatrix {
    fn is_3d(&self) -> bool {
        self.m13 != 0.0 || self.m14 != 0.0 ||
        self.m23 != 0.0 || self.m24 != 0.0 ||
        self.m31 != 0.0 || self.m32 != 0.0 || self.m33 != 1.0 || self.m34 != 0.0 ||
        self.m43 != 0.0 || self.m44 != 1.0
    }

    fn determinant(&self) -> CSSFloat {
        self.m14 * self.m23 * self.m32 * self.m41 -
        self.m13 * self.m24 * self.m32 * self.m41 -
        self.m14 * self.m22 * self.m33 * self.m41 +
        self.m12 * self.m24 * self.m33 * self.m41 +
        self.m13 * self.m22 * self.m34 * self.m41 -
        self.m12 * self.m23 * self.m34 * self.m41 -
        self.m14 * self.m23 * self.m31 * self.m42 +
        self.m13 * self.m24 * self.m31 * self.m42 +
        self.m14 * self.m21 * self.m33 * self.m42 -
        self.m11 * self.m24 * self.m33 * self.m42 -
        self.m13 * self.m21 * self.m34 * self.m42 +
        self.m11 * self.m23 * self.m34 * self.m42 +
        self.m14 * self.m22 * self.m31 * self.m43 -
        self.m12 * self.m24 * self.m31 * self.m43 -
        self.m14 * self.m21 * self.m32 * self.m43 +
        self.m11 * self.m24 * self.m32 * self.m43 +
        self.m12 * self.m21 * self.m34 * self.m43 -
        self.m11 * self.m22 * self.m34 * self.m43 -
        self.m13 * self.m22 * self.m31 * self.m44 +
        self.m12 * self.m23 * self.m31 * self.m44 +
        self.m13 * self.m21 * self.m32 * self.m44 -
        self.m11 * self.m23 * self.m32 * self.m44 -
        self.m12 * self.m21 * self.m33 * self.m44 +
        self.m11 * self.m22 * self.m33 * self.m44
    }

    fn inverse(&self) -> Option<ComputedMatrix> {
        let mut det = self.determinant();

        if det == 0.0 {
            return None;
        }

        det = 1.0 / det;
        let x = ComputedMatrix {
            m11: det *
            (self.m23*self.m34*self.m42 - self.m24*self.m33*self.m42 +
             self.m24*self.m32*self.m43 - self.m22*self.m34*self.m43 -
             self.m23*self.m32*self.m44 + self.m22*self.m33*self.m44),
            m12: det *
            (self.m14*self.m33*self.m42 - self.m13*self.m34*self.m42 -
             self.m14*self.m32*self.m43 + self.m12*self.m34*self.m43 +
             self.m13*self.m32*self.m44 - self.m12*self.m33*self.m44),
            m13: det *
            (self.m13*self.m24*self.m42 - self.m14*self.m23*self.m42 +
             self.m14*self.m22*self.m43 - self.m12*self.m24*self.m43 -
             self.m13*self.m22*self.m44 + self.m12*self.m23*self.m44),
            m14: det *
            (self.m14*self.m23*self.m32 - self.m13*self.m24*self.m32 -
             self.m14*self.m22*self.m33 + self.m12*self.m24*self.m33 +
             self.m13*self.m22*self.m34 - self.m12*self.m23*self.m34),
            m21: det *
            (self.m24*self.m33*self.m41 - self.m23*self.m34*self.m41 -
             self.m24*self.m31*self.m43 + self.m21*self.m34*self.m43 +
             self.m23*self.m31*self.m44 - self.m21*self.m33*self.m44),
            m22: det *
            (self.m13*self.m34*self.m41 - self.m14*self.m33*self.m41 +
             self.m14*self.m31*self.m43 - self.m11*self.m34*self.m43 -
             self.m13*self.m31*self.m44 + self.m11*self.m33*self.m44),
            m23: det *
            (self.m14*self.m23*self.m41 - self.m13*self.m24*self.m41 -
             self.m14*self.m21*self.m43 + self.m11*self.m24*self.m43 +
             self.m13*self.m21*self.m44 - self.m11*self.m23*self.m44),
            m24: det *
            (self.m13*self.m24*self.m31 - self.m14*self.m23*self.m31 +
             self.m14*self.m21*self.m33 - self.m11*self.m24*self.m33 -
             self.m13*self.m21*self.m34 + self.m11*self.m23*self.m34),
            m31: det *
            (self.m22*self.m34*self.m41 - self.m24*self.m32*self.m41 +
             self.m24*self.m31*self.m42 - self.m21*self.m34*self.m42 -
             self.m22*self.m31*self.m44 + self.m21*self.m32*self.m44),
            m32: det *
            (self.m14*self.m32*self.m41 - self.m12*self.m34*self.m41 -
             self.m14*self.m31*self.m42 + self.m11*self.m34*self.m42 +
             self.m12*self.m31*self.m44 - self.m11*self.m32*self.m44),
            m33: det *
            (self.m12*self.m24*self.m41 - self.m14*self.m22*self.m41 +
             self.m14*self.m21*self.m42 - self.m11*self.m24*self.m42 -
             self.m12*self.m21*self.m44 + self.m11*self.m22*self.m44),
            m34: det *
            (self.m14*self.m22*self.m31 - self.m12*self.m24*self.m31 -
             self.m14*self.m21*self.m32 + self.m11*self.m24*self.m32 +
             self.m12*self.m21*self.m34 - self.m11*self.m22*self.m34),
            m41: det *
            (self.m23*self.m32*self.m41 - self.m22*self.m33*self.m41 -
             self.m23*self.m31*self.m42 + self.m21*self.m33*self.m42 +
             self.m22*self.m31*self.m43 - self.m21*self.m32*self.m43),
            m42: det *
            (self.m12*self.m33*self.m41 - self.m13*self.m32*self.m41 +
             self.m13*self.m31*self.m42 - self.m11*self.m33*self.m42 -
             self.m12*self.m31*self.m43 + self.m11*self.m32*self.m43),
            m43: det *
            (self.m13*self.m22*self.m41 - self.m12*self.m23*self.m41 -
             self.m13*self.m21*self.m42 + self.m11*self.m23*self.m42 +
             self.m12*self.m21*self.m43 - self.m11*self.m22*self.m43),
            m44: det *
            (self.m12*self.m23*self.m31 - self.m13*self.m22*self.m31 +
             self.m13*self.m21*self.m32 - self.m11*self.m23*self.m32 -
             self.m12*self.m21*self.m33 + self.m11*self.m22*self.m33),
        };

        Some(x)
    }
}

/// https://drafts.csswg.org/css-transforms/#interpolation-of-transforms
impl Interpolate for TransformList {
    #[inline]
    fn interpolate(&self, other: &TransformList, progress: f64) -> Result<Self, ()> {
        // http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
        let result = match (&self.0, &other.0) {
            (&Some(ref from_list), &Some(ref to_list)) => {
                // Two lists of transforms
                interpolate_transform_list(from_list, &to_list, progress)
            }
            (&Some(ref from_list), &None) => {
                // http://dev.w3.org/csswg/css-transforms/#none-transform-animation
                let to_list = build_identity_transform_list(from_list);
                interpolate_transform_list(from_list, &to_list, progress)
            }
            (&None, &Some(ref to_list)) => {
                // http://dev.w3.org/csswg/css-transforms/#none-transform-animation
                let from_list = build_identity_transform_list(to_list);
                interpolate_transform_list(&from_list, to_list, progress)
            }
            _ => {
                // http://dev.w3.org/csswg/css-transforms/#none-none-animation
                TransformList(None)
            }
        };

        Ok(result)
    }
}

impl<T, U> Interpolate for Either<T, U>
        where T: Interpolate + Copy, U: Interpolate + Copy,
{
    #[inline]
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (Either::First(ref this), Either::First(ref other)) => {
                this.interpolate(&other, progress).map(Either::First)
            },
            (Either::Second(ref this), Either::Second(ref other)) => {
                this.interpolate(&other, progress).map(Either::Second)
            },
            _ => {
                let interpolated = if progress < 0.5 { *self } else { *other };
                Ok(interpolated)
            }
        }
    }
}


/// We support ComputeDistance for an API in gecko to test the transition per property.
impl ComputeDistance for AnimationValue {
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable:
                    % if prop.animation_type == "normal":
                        (&AnimationValue::${prop.camel_case}(ref from),
                         &AnimationValue::${prop.camel_case}(ref to)) => {
                            from.compute_distance(to)
                        },
                    % else:
                        (&AnimationValue::${prop.camel_case}(ref _from),
                         &AnimationValue::${prop.camel_case}(ref _to)) => {
                            Err(())
                        },
                    % endif
                % endif
            % endfor
            _ => {
                panic!("Expected compute_distance of computed values of the same \
                        property, got: {:?}, {:?}", self, other);
            }
        }
    }
}

/// A trait used to implement [compute_distance].
/// In order to compute the Euclidean distance of a list, we need to compute squared distance
/// for each element, so the vector can sum it and then get its squared root as the distance.
pub trait ComputeDistance: Sized {
    /// Compute distance between a value and another for a given property.
    fn compute_distance(&self, other: &Self) -> Result<f64, ()>;

    /// Compute squared distance between a value and another for a given property.
    /// This is used for list or if there are many components in a property value.
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_distance(other).map(|d| d * d)
    }
}

impl<T: ComputeDistance> ComputeDistance for Vec<T> {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        if self.len() != other.len() {
            return Err(());
        }

        let mut squared_dist = 0.0f64;
        for (this, other) in self.iter().zip(other) {
            let diff = try!(this.compute_squared_distance(other));
            squared_dist += diff;
        }
        Ok(squared_dist)
    }
}

impl ComputeDistance for Au {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }
}

impl ComputeDistance for Auto {
    #[inline]
    fn compute_distance(&self, _other: &Self) -> Result<f64, ()> {
        Err(())
    }
}

impl ComputeDistance for Normal {
    #[inline]
    fn compute_distance(&self, _other: &Self) -> Result<f64, ()> {
        Err(())
    }
}

impl <T> ComputeDistance for Option<T>
    where T: ComputeDistance,
{
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                this.compute_squared_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl ComputeDistance for f32 {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok((*self - *other).abs() as f64)
    }
}

impl ComputeDistance for f64 {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok((*self - *other).abs())
    }
}

impl ComputeDistance for i32 {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok((*self - *other).abs() as f64)
    }
}

impl ComputeDistance for Visibility {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        if *self == *other {
            Ok(0.0)
        } else {
            Ok(1.0)
        }
    }
}

/// https://www.w3.org/TR/smil-animation/#animateColorElement says we should use Euclidean RGB-cube distance.
impl ComputeDistance for RGBA {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        fn clamp(val: f32) -> f32 {
            val.max(0.).min(1.)
        }

        let start_a = clamp(self.alpha_f32());
        let end_a = clamp(other.alpha_f32());
        let start = [ start_a,
                      self.red_f32() * start_a,
                      self.green_f32() * start_a,
                      self.blue_f32() * start_a ];
        let end = [ end_a,
                    other.red_f32() * end_a,
                    other.green_f32() * end_a,
                    other.blue_f32() * end_a ];
        let diff = start.iter().zip(&end)
                               .fold(0.0f64, |n, (&a, &b)| {
                                   let diff = (a - b) as f64;
                                   n + diff * diff
                               });
        Ok(diff)
    }
}

impl ComputeDistance for CSSParserColor {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sq| sq.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (CSSParserColor::RGBA(ref this), CSSParserColor::RGBA(ref other)) => {
                this.compute_squared_distance(other)
            },
            _ => Ok(0.0),
        }
    }
}

impl ComputeDistance for CalcLengthOrPercentage {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sq| sq.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        let length_diff = (self.length().0 - other.length().0) as f64;
        let percentage_diff = (self.percentage() - other.percentage()) as f64;
        Ok(length_diff * length_diff + percentage_diff * percentage_diff)
    }
}

impl ComputeDistance for LengthOrPercentage {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                this.compute_distance(other)
            },
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                this.compute_distance(other)
            },
            (this, other) => {
                let this: CalcLengthOrPercentage = From::from(this);
                let other: CalcLengthOrPercentage = From::from(other);
                this.compute_distance(&other)
            }
        }
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                let diff = (this.0 - other.0) as f64;
                Ok(diff * diff)
            },
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                let diff = (this - other) as f64;
                Ok(diff * diff)
            },
            (this, other) => {
                let this: CalcLengthOrPercentage = From::from(this);
                let other: CalcLengthOrPercentage = From::from(other);
                let length_diff = (this.length().0 - other.length().0) as f64;
                let percentage_diff = (this.percentage() - other.percentage()) as f64;
                Ok(length_diff * length_diff + percentage_diff * percentage_diff)
            }
        }
    }
}

impl ComputeDistance for LengthOrPercentageOrAuto {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.compute_distance(other)
            },
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.compute_distance(other)
            },
            (this, other) => {
                // If one of the element is Auto, Option<> will be None, and the returned distance is Err(())
                let this: Option<CalcLengthOrPercentage> = From::from(this);
                let other: Option<CalcLengthOrPercentage> = From::from(other);
                this.compute_distance(&other)
            }
        }
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                let diff = (this.0 - other.0) as f64;
                Ok(diff * diff)
            },
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                let diff = (this - other) as f64;
                Ok(diff * diff)
            },
            (this, other) => {
                let this: Option<CalcLengthOrPercentage> = From::from(this);
                let other: Option<CalcLengthOrPercentage> = From::from(other);
                if this.is_none() || other.is_none() {
                    Err(())
                } else {
                    let length_diff = (this.unwrap().length().0 - other.unwrap().length().0) as f64;
                    let percentage_diff = (this.unwrap().percentage() - other.unwrap().percentage()) as f64;
                    Ok(length_diff * length_diff + percentage_diff * percentage_diff)
                }
            }
        }
    }
}

impl ComputeDistance for LengthOrPercentageOrNone {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrNone::Length(ref this),
             LengthOrPercentageOrNone::Length(ref other)) => {
                this.compute_distance(other)
            },
            (LengthOrPercentageOrNone::Percentage(ref this),
             LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(())
        }
    }
}

impl ComputeDistance for LengthOrNone {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (Either::First(ref length), Either::First(ref other)) => {
                length.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl ComputeDistance for MinLength {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (MinLength::LengthOrPercentage(ref this),
             MinLength::LengthOrPercentage(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl ComputeDistance for MaxLength {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (MaxLength::LengthOrPercentage(ref this),
             MaxLength::LengthOrPercentage(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl ComputeDistance for VerticalAlign {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (VerticalAlign::LengthOrPercentage(ref this),
             VerticalAlign::LengthOrPercentage(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl ComputeDistance for BorderRadiusSize {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok(try!(self.0.width.compute_squared_distance(&other.0.width)) +
           try!(self.0.height.compute_squared_distance(&other.0.height)))
    }
}

impl ComputeDistance for BackgroundSizeList {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_squared_distance(&other.0)
    }
}

impl ComputeDistance for LineHeight {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (LineHeight::Length(ref this),
             LineHeight::Length(ref other)) => {
                this.compute_distance(other)
            },
            (LineHeight::Number(ref this),
             LineHeight::Number(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

impl ComputeDistance for FontWeight {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        let a = (*self as u32) as f64;
        let b = (*other as u32) as f64;
        a.compute_distance(&b)
    }
}

impl ComputeDistance for Position {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok(try!(self.horizontal.compute_squared_distance(&other.horizontal)) +
           try!(self.vertical.compute_squared_distance(&other.vertical)))
    }
}

impl ComputeDistance for HorizontalPosition {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_squared_distance(&other.0)
    }
}

impl ComputeDistance for VerticalPosition {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_squared_distance(&other.0)
    }
}

impl ComputeDistance for ClipRect {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        let list = [ try!(self.top.compute_distance(&other.top)),
                     try!(self.right.compute_distance(&other.right)),
                     try!(self.bottom.compute_distance(&other.bottom)),
                     try!(self.left.compute_distance(&other.left)) ];
        Ok(list.iter().fold(0.0f64, |sum, diff| sum + diff * diff))
    }
}

impl ComputeDistance for TextShadow {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        let list = [ try!(self.offset_x.compute_distance(&other.offset_x)),
                     try!(self.offset_y.compute_distance(&other.offset_y)),
                     try!(self.blur_radius.compute_distance(&other.blur_radius)),
                     try!(self.color.compute_distance(&other.color)) ];
        Ok(list.iter().fold(0.0f64, |sum, diff| sum + diff * diff))
    }
}

impl ComputeDistance for TextShadowList {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        let zero = TextShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA::transparent()),
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut diff_squared = 0.0f64;
        for i in 0..max_len {
            diff_squared += match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other)) => {
                    try!(shadow.compute_squared_distance(other))
                },
                (Some(shadow), None) |
                (None, Some(shadow)) => {
                    try!(shadow.compute_squared_distance(&zero))
                },
                (None, None) => unreachable!(),
            };
        }
        Ok(diff_squared)
    }
}

impl ComputeDistance for BoxShadow {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        if self.inset != other.inset {
            return Err(());
        }
        let list = [ try!(self.offset_x.compute_distance(&other.offset_x)),
                     try!(self.offset_y.compute_distance(&other.offset_y)),
                     try!(self.color.compute_distance(&other.color)),
                     try!(self.spread_radius.compute_distance(&other.spread_radius)),
                     try!(self.blur_radius.compute_distance(&other.blur_radius)) ];
        Ok(list.iter().fold(0.0f64, |sum, diff| sum + diff * diff))
    }
}

impl ComputeDistance for BoxShadowList {
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        // The inset value must change
        let mut zero = BoxShadow {
            offset_x: Au(0),
            offset_y: Au(0),
            spread_radius: Au(0),
            blur_radius: Au(0),
            color: CSSParserColor::RGBA(RGBA::transparent()),
            inset: false,
        };

        let max_len = cmp::max(self.0.len(), other.0.len());
        let mut diff_squared = 0.0f64;
        for i in 0..max_len {
            diff_squared += match (self.0.get(i), other.0.get(i)) {
                (Some(shadow), Some(other)) => {
                    try!(shadow.compute_squared_distance(other))
                },
                (Some(shadow), None) |
                (None, Some(shadow)) => {
                    zero.inset = shadow.inset;
                    try!(shadow.compute_squared_distance(&zero))
                }
                (None, None) => unreachable!(),
            };
        }
        Ok(diff_squared)
    }
}

impl ComputeDistance for TransformList {
    #[inline]
    fn compute_distance(&self, _other: &Self) -> Result<f64, ()> {
        Err(())
    }
}

impl<T, U> ComputeDistance for Either<T, U>
    where T: ComputeDistance, U: ComputeDistance
{
    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            (&Either::First(ref this), &Either::First(ref other)) => {
                this.compute_distance(other)
            },
            (&Either::Second(ref this), &Either::Second(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(())
        }
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            (&Either::First(ref this), &Either::First(ref other)) => {
                this.compute_squared_distance(other)
            },
            (&Either::Second(ref this), &Either::Second(ref other)) => {
                this.compute_squared_distance(other)
            },
            _ => Err(())
        }
    }
}
