/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% from data import SYSTEM_FONT_LONGHANDS %>

use app_units::Au;
use cssparser::{Color as CSSParserColor, Parser, RGBA, serialize_identifier};
use euclid::{Point2D, Size2D};
#[cfg(feature = "gecko")] use gecko_bindings::bindings::RawServoAnimationValueMap;
#[cfg(feature = "gecko")] use gecko_bindings::structs::nsCSSPropertyID;
#[cfg(feature = "gecko")] use gecko_bindings::sugar::ownership::{HasFFI, HasSimpleFFI};
#[cfg(feature = "gecko")] use gecko_string_cache::Atom;
use properties::{CSSWideKeyword, PropertyDeclaration};
use properties::longhands;
use properties::longhands::background_size::computed_value::T as BackgroundSizeList;
use properties::longhands::font_weight::computed_value::T as FontWeight;
use properties::longhands::font_stretch::computed_value::T as FontStretch;
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
#[cfg(feature = "servo")] use servo_atoms::Atom;
use smallvec::SmallVec;
use std::cmp;
#[cfg(feature = "gecko")] use std::collections::HashMap;
use std::fmt;
use style_traits::ToCss;
use super::ComputedValues;
use values::CSSFloat;
use values::{Auto, Either};
use values::computed::{Angle, LengthOrPercentageOrAuto, LengthOrPercentageOrNone};
use values::computed::{BorderCornerRadius, ClipRect};
use values::computed::{CalcLengthOrPercentage, Context, LengthOrPercentage};
use values::computed::{MaxLength, MozLength};
use values::computed::ToComputedValue;
use values::generics::border::BorderCornerRadius as GenericBorderCornerRadius;
use values::generics::position as generic_position;


/// A given transition property, that is either `All`, or an animatable
/// property.
// NB: This needs to be here because it needs all the longhands generated
// beforehand.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    /// Unrecognized property which could be any non-animatable, custom property, or
    /// unknown property.
    Unsupported(Atom)
}

impl TransitionProperty {
    /// Iterates over each longhand property.
    pub fn each<F: FnMut(&TransitionProperty) -> ()>(mut cb: F) {
        % for prop in data.longhands:
            % if prop.animatable:
                cb(&TransitionProperty::${prop.camel_case});
            % endif
        % endfor
    }

    /// Iterates over every property that is not TransitionProperty::All, stopping and returning
    /// true when the provided callback returns true for the first time.
    pub fn any<F: FnMut(&TransitionProperty) -> bool>(mut cb: F) -> bool {
        % for prop in data.longhands:
            % if prop.animatable:
                if cb(&TransitionProperty::${prop.camel_case}) {
                    return true;
                }
            % endif
        % endfor
        false
    }

    /// Parse a transition-property value.
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        let ident = try!(input.expect_ident());
        match_ignore_ascii_case! { &ident,
            "all" => Ok(TransitionProperty::All),
            % for prop in data.longhands:
                % if prop.animatable:
                    "${prop.name}" => Ok(TransitionProperty::${prop.camel_case}),
                % endif
            % endfor
            % for prop in data.shorthands_except_all():
                "${prop.name}" => Ok(TransitionProperty::${prop.camel_case}),
            % endfor
            "none" => Err(()),
            _ => {
                match CSSWideKeyword::from_ident(&ident) {
                    Some(_) => Err(()),
                    None => Ok(TransitionProperty::Unsupported((&*ident).into()))
                }
            }
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
                % if prop.animation_value_type == "discrete":
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
            #[cfg(feature = "gecko")]
            TransitionProperty::Unsupported(ref atom) => serialize_identifier(&atom.to_string(),
                                                                              dest),
            #[cfg(feature = "servo")]
            TransitionProperty::Unsupported(ref atom) => serialize_identifier(atom, dest),
        }
    }
}

/// Convert to nsCSSPropertyID.
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl<'a> From< &'a TransitionProperty> for nsCSSPropertyID {
    fn from(transition_property: &'a TransitionProperty) -> nsCSSPropertyID {
        match *transition_property {
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
            _ => panic!("Unconvertable Servo transition property: {:?}", transition_property),
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
                % else:
                    ${helpers.to_nscsspropertyid(prop.ident)}
                        => TransitionProperty::Unsupported(Atom::from("${prop.ident}")),
                % endif
            % endfor
            % for prop in data.shorthands_except_all():
                ${helpers.to_nscsspropertyid(prop.ident)}
                    => TransitionProperty::${prop.camel_case},
            % endfor
            nsCSSPropertyID::eCSSPropertyExtra_all_properties => TransitionProperty::All,
            _ => panic!("Unconvertable nsCSSPropertyID: {:?}", property),
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
                        % if prop.animation_value_type == "discrete":
                            let value = if progress < 0.5 { from.clone() } else { to.clone() };
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
            ref other => panic!("Can't use TransitionProperty::{:?} here", other),
        }
    }
}

/// A collection of AnimationValue that were composed on an element.
/// This HashMap stores the values that are the last AnimationValue to be
/// composed for each TransitionProperty.
#[cfg(feature = "gecko")]
pub type AnimationValueMap = HashMap<TransitionProperty, AnimationValue>;
#[cfg(feature = "gecko")]
unsafe impl HasFFI for AnimationValueMap {
    type FFIType = RawServoAnimationValueMap;
}
#[cfg(feature = "gecko")]
unsafe impl HasSimpleFFI for AnimationValueMap {}

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
            % if prop.is_animatable_with_computed_value:
                ${prop.camel_case}(longhands::${prop.ident}::computed_value::T),
            % else:
                ${prop.camel_case}(${prop.animation_value_type}),
            % endif
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
                            Box::new(
                            % endif
                                longhands::${prop.ident}::SpecifiedValue::from_computed_value(
                                % if prop.is_animatable_with_computed_value:
                                    from
                                % else:
                                    &from.into()
                                % endif
                                ))
                            % if prop.boxed:
                            )
                            % endif
                    }
                % endif
            % endfor
        }
    }

    /// Construct an AnimationValue from a property declaration
    pub fn from_declaration(decl: &PropertyDeclaration, context: &mut Context,
                            initial: &ComputedValues) -> Option<Self> {
        use error_reporting::RustLogReporter;
        use properties::LonghandId;
        use properties::DeclaredValue;

        match *decl {
            % for prop in data.longhands:
            % if prop.animatable:
            PropertyDeclaration::${prop.camel_case}(ref val) => {
            % if prop.ident in SYSTEM_FONT_LONGHANDS and product == "gecko":
                if let Some(sf) = val.get_system() {
                    longhands::system_font::resolve_system_font(sf, context);
                }
            % endif
                Some(AnimationValue::${prop.camel_case}(
                % if prop.is_animatable_with_computed_value:
                    val.to_computed_value(context)
                % else:
                    From::from(&val.to_computed_value(context))
                % endif
                ))
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
                        % if not prop.is_animatable_with_computed_value:
                            let computed = From::from(&computed);
                        % endif
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
                let reporter = RustLogReporter;
                match id {
                    % for prop in data.longhands:
                    % if prop.animatable:
                    LonghandId::${prop.camel_case} => {
                        let mut result = None;
                        let quirks_mode = context.quirks_mode;
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
                            &reporter,
                            quirks_mode);
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
                        % if prop.is_animatable_with_computed_value:
                            computed_values.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}())
                        % else:
                            From::from(&computed_values.get_${prop.style_struct.ident.strip("_")}()
                                                                  .clone_${prop.ident}()))
                        % endif
                    }
                % endif
            % endfor
            ref other => panic!("Can't use TransitionProperty::{:?} here.", other),
        }
    }
}

impl Animatable for AnimationValue {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
        -> Result<Self, ()> {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable:
                    (&AnimationValue::${prop.camel_case}(ref from),
                     &AnimationValue::${prop.camel_case}(ref to)) => {
                        % if prop.animation_value_type == "discrete":
                            if self_portion > other_portion {
                                Ok(AnimationValue::${prop.camel_case}(from.clone()))
                            } else {
                                Ok(AnimationValue::${prop.camel_case}(to.clone()))
                            }
                        % else:
                            from.add_weighted(to, self_portion, other_portion)
                                .map(AnimationValue::${prop.camel_case})
                        % endif
                    }
                % endif
            % endfor
            _ => {
                panic!("Expected weighted addition of computed values of the same \
                        property, got: {:?}, {:?}", self, other);
            }
        }
    }

    fn add(&self, other: &Self) -> Result<Self, ()> {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable:
                    % if prop.animation_value_type == "discrete":
                        (&AnimationValue::${prop.camel_case}(_),
                         &AnimationValue::${prop.camel_case}(_)) => {
                            Err(())
                        }
                    % else:
                        (&AnimationValue::${prop.camel_case}(ref from),
                         &AnimationValue::${prop.camel_case}(ref to)) => {
                            from.add(to).map(AnimationValue::${prop.camel_case})
                        }
                    % endif
                % endif
            % endfor
            _ => {
                panic!("Expected weighted addition of computed values of the same \
                        property, got: {:?}, {:?}", self, other);
            }
        }
    }

    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable:
                    % if prop.animation_value_type != "discrete":
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


/// A trait used to implement various procedures used during animation.
pub trait Animatable: Sized {
    /// Performs a weighted sum of this value and |other|. This is used for
    /// interpolation and addition of animation values.
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
        -> Result<Self, ()>;

    /// [Interpolates][interpolation] a value with another for a given property.
    ///
    /// [interpolation]: https://w3c.github.io/web-animations/#animation-interpolation
    fn interpolate(&self, other: &Self, progress: f64) -> Result<Self, ()> {
        self.add_weighted(other, 1.0 - progress, progress)
    }

    /// Returns the [sum][animation-addition] of this value and |other|.
    ///
    /// [animation-addition]: https://w3c.github.io/web-animations/#animation-addition
    fn add(&self, other: &Self) -> Result<Self, ()> {
        self.add_weighted(other, 1.0, 1.0)
    }

    /// [Accumulates][animation-accumulation] this value onto itself (|count| - 1) times then
    /// accumulates |other| onto the result.
    /// If |count| is zero, the result will be |other|.
    ///
    /// [animation-accumulation]: https://w3c.github.io/web-animations/#animation-accumulation
    fn accumulate(&self, other: &Self, count: u64) -> Result<Self, ()> {
        self.add_weighted(other, count as f64, 1.0)
    }

    /// Compute distance between a value and another for a given property.
    fn compute_distance(&self, _other: &Self) -> Result<f64, ()>  { Err(()) }

    /// In order to compute the Euclidean distance of a list or property value with multiple
    /// components, we need to compute squared distance for each element, so the vector can sum it
    /// and then get its squared root as the distance.
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_distance(other).map(|d| d * d)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-repeatable-list
pub trait RepeatableListAnimatable: Animatable {}

impl RepeatableListAnimatable for LengthOrPercentage {}
impl RepeatableListAnimatable for Either<f32, LengthOrPercentage> {}

impl<T: RepeatableListAnimatable> Animatable for SmallVec<[T; 1]> {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
        -> Result<Self, ()> {
        use num_integer::lcm;
        let len = lcm(self.len(), other.len());
        self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(me, you)| {
            me.add_weighted(you, self_portion, other_portion)
        }).collect()
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sd| sd.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        use num_integer::lcm;
        let len = lcm(self.len(), other.len());
        self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(me, you)| {
            me.compute_squared_distance(you)
        }).collect::<Result<Vec<_>, _>>().map(|d| d.iter().sum())
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Animatable for Au {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Au((self.0 as f64 * self_portion + other.0 as f64 * other_portion).round() as i32))
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }
}

impl <T> Animatable for Option<T>
    where T: Animatable,
{
    #[inline]
    fn add_weighted(&self, other: &Option<T>, self_portion: f64, other_portion: f64) -> Result<Option<T>, ()> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                Ok(this.add_weighted(other, self_portion, other_portion).ok())
            }
            (&None, &None) => Ok(None),
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (self, other) {
            (&Some(ref this), &Some(ref other)) => {
                this.compute_distance(other)
            },
            (&None, &None) => Ok(0.0),
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

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Animatable for f32 {
    #[inline]
    fn add_weighted(&self, other: &f32, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok((*self as f64 * self_portion + *other as f64 * other_portion) as f32)
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok((*self - *other).abs() as f64)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Animatable for f64 {
    #[inline]
    fn add_weighted(&self, other: &f64, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(*self * self_portion + *other * other_portion)
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok((*self - *other).abs())
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-integer
impl Animatable for i32 {
    #[inline]
    fn add_weighted(&self, other: &i32, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok((*self as f64 * self_portion + *other as f64 * other_portion).round() as i32)
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        Ok((*self - *other).abs() as f64)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
impl Animatable for Angle {
    #[inline]
    fn add_weighted(&self, other: &Angle, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        self.radians()
            .add_weighted(&other.radians(), self_portion, other_portion)
            .map(Angle::from_radians)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-visibility
impl Animatable for Visibility {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (Visibility::visible, _) => {
                Ok(if self_portion > 0.0 { *self } else { *other })
            },
            (_, Visibility::visible) => {
                Ok(if other_portion > 0.0 { *other } else { *self })
            },
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        if *self == *other {
            Ok(0.0)
        } else {
            Ok(1.0)
        }
    }
}

impl<T: Animatable + Copy> Animatable for Size2D<T> {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        let width = try!(self.width.add_weighted(&other.width, self_portion, other_portion));
        let height = try!(self.height.add_weighted(&other.height, self_portion, other_portion));

        Ok(Size2D::new(width, height))
    }
}

impl<T: Animatable + Copy> Animatable for Point2D<T> {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        let x = try!(self.x.add_weighted(&other.x, self_portion, other_portion));
        let y = try!(self.y.add_weighted(&other.y, self_portion, other_portion));

        Ok(Point2D::new(x, y))
    }
}

impl Animatable for BorderCornerRadius {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        self.0.add_weighted(&other.0, self_portion, other_portion).map(GenericBorderCornerRadius)
    }

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

/// https://drafts.csswg.org/css-transitions/#animtype-length
impl Animatable for VerticalAlign {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref this)),
             VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(ref other))) => {
                this.add_weighted(other, self_portion, other_portion).map(|value| {
                    VerticalAlign::LengthOrPercentage(LengthOrPercentage::Length(value))
                })
            }
            _ => Err(()),
        }
    }

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

impl Animatable for BackgroundSizeList {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        self.0.add_weighted(&other.0, self_portion, other_portion).map(BackgroundSizeList)
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_distance(&other.0)
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        self.0.compute_squared_distance(&other.0)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-color
impl Animatable for RGBA {
    #[inline]
    fn add_weighted(&self, other: &RGBA, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        fn clamp(val: f32) -> f32 {
            val.max(0.).min(1.)
        }

        let alpha = clamp(try!(self.alpha_f32().add_weighted(&other.alpha_f32(),
                                                             self_portion, other_portion)));
        if alpha == 0. {
            Ok(RGBA::transparent())
        } else {
            // NB: We rely on RGBA::from_floats clamping already.
            let red = try!((self.red_f32() * self.alpha_f32())
                            .add_weighted(&(other.red_f32() * other.alpha_f32()),
                                          self_portion, other_portion))
                            * 1. / alpha;
            let green = try!((self.green_f32() * self.alpha_f32())
                             .add_weighted(&(other.green_f32() * other.alpha_f32()),
                                           self_portion, other_portion))
                             * 1. / alpha;
            let blue = try!((self.blue_f32() * self.alpha_f32())
                             .add_weighted(&(other.blue_f32() * other.alpha_f32()),
                                           self_portion, other_portion))
                             * 1. / alpha;
            Ok(RGBA::from_floats(red, green, blue, alpha))
        }
    }

    /// https://www.w3.org/TR/smil-animation/#animateColorElement says we should use Euclidean
    /// RGB-cube distance.
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

/// https://drafts.csswg.org/css-transitions/#animtype-color
impl Animatable for CSSParserColor {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (CSSParserColor::RGBA(ref this), CSSParserColor::RGBA(ref other)) => {
                this.add_weighted(other, self_portion, other_portion).map(CSSParserColor::RGBA)
            }
            _ => Err(()),
        }
    }

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

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animatable for CalcLengthOrPercentage {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        fn add_weighted_half<T>(this: Option<T>,
                                other: Option<T>,
                                self_portion: f64,
                                other_portion: f64)
                                -> Result<Option<T>, ()>
            where T: Default + Animatable,
        {
            match (this, other) {
                (None, None) => Ok(None),
                (this, other) => {
                    let this = this.unwrap_or(T::default());
                    let other = other.unwrap_or(T::default());
                    this.add_weighted(&other, self_portion, other_portion).map(Some)
                }
            }
        }

        let length = self.unclamped_length().add_weighted(&other.unclamped_length(), self_portion, other_portion)?;
        let percentage = add_weighted_half(self.percentage, other.percentage, self_portion, other_portion)?;
        Ok(CalcLengthOrPercentage::with_clamping_mode(length, percentage, self.clamping_mode))
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sq| sq.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        let length_diff = (self.unclamped_length().0 - other.unclamped_length().0) as f64;
        let percentage_diff = (self.percentage() - other.percentage()) as f64;
        Ok(length_diff * length_diff + percentage_diff * percentage_diff)
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animatable for LengthOrPercentage {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentage::Length(ref this),
             LengthOrPercentage::Length(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(LengthOrPercentage::Length)
            }
            (LengthOrPercentage::Percentage(ref this),
             LengthOrPercentage::Percentage(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(LengthOrPercentage::Percentage)
            }
            (this, other) => {
                let this: CalcLengthOrPercentage = From::from(this);
                let other: CalcLengthOrPercentage = From::from(other);
                this.add_weighted(&other, self_portion, other_portion)
                    .map(LengthOrPercentage::Calc)
            }
        }
    }

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
                let length_diff = (this.unclamped_length().0 - other.unclamped_length().0) as f64;
                let percentage_diff = (this.percentage() - other.percentage()) as f64;
                Ok(length_diff * length_diff + percentage_diff * percentage_diff)
            }
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animatable for LengthOrPercentageOrAuto {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrAuto::Length(ref this),
             LengthOrPercentageOrAuto::Length(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(LengthOrPercentageOrAuto::Length)
            }
            (LengthOrPercentageOrAuto::Percentage(ref this),
             LengthOrPercentageOrAuto::Percentage(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(LengthOrPercentageOrAuto::Percentage)
            }
            (LengthOrPercentageOrAuto::Auto, LengthOrPercentageOrAuto::Auto) => {
                Ok(LengthOrPercentageOrAuto::Auto)
            }
            (this, other) => {
                let this: Option<CalcLengthOrPercentage> = From::from(this);
                let other: Option<CalcLengthOrPercentage> = From::from(other);
                match this.add_weighted(&other, self_portion, other_portion) {
                    Ok(Some(result)) => Ok(LengthOrPercentageOrAuto::Calc(result)),
                    _ => Err(()),
                }
            }
        }
    }

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
                if let (Some(this), Some(other)) = (this, other) {
                    let length_diff = (this.unclamped_length().0 - other.unclamped_length().0) as f64;
                    let percentage_diff = (this.percentage() - other.percentage()) as f64;
                    Ok(length_diff * length_diff + percentage_diff * percentage_diff)
                } else {
                    Err(())
                }
            }
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animatable for LengthOrPercentageOrNone {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LengthOrPercentageOrNone::Length(ref this),
             LengthOrPercentageOrNone::Length(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(LengthOrPercentageOrNone::Length)
            }
            (LengthOrPercentageOrNone::Percentage(ref this),
             LengthOrPercentageOrNone::Percentage(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(LengthOrPercentageOrNone::Percentage)
            }
            (LengthOrPercentageOrNone::None, LengthOrPercentageOrNone::None) => {
                Ok(LengthOrPercentageOrNone::None)
            }
            _ => Err(())
        }
    }

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

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animatable for MozLength {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (MozLength::LengthOrPercentageOrAuto(ref this),
             MozLength::LengthOrPercentageOrAuto(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(MozLength::LengthOrPercentageOrAuto)
            }
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (MozLength::LengthOrPercentageOrAuto(ref this),
             MozLength::LengthOrPercentageOrAuto(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animatable for MaxLength {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (MaxLength::LengthOrPercentageOrNone(ref this),
             MaxLength::LengthOrPercentageOrNone(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(MaxLength::LengthOrPercentageOrNone)
            }
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (MaxLength::LengthOrPercentageOrNone(ref this),
             MaxLength::LengthOrPercentageOrNone(ref other)) => {
                this.compute_distance(other)
            },
            _ => Err(()),
        }
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-number
/// https://drafts.csswg.org/css-transitions/#animtype-length
impl Animatable for LineHeight {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (LineHeight::Length(ref this),
             LineHeight::Length(ref other)) => {
                this.add_weighted(other, self_portion, other_portion).map(LineHeight::Length)
            }
            (LineHeight::Number(ref this),
             LineHeight::Number(ref other)) => {
                this.add_weighted(other, self_portion, other_portion).map(LineHeight::Number)
            }
            (LineHeight::Normal, LineHeight::Normal) => {
                Ok(LineHeight::Normal)
            }
            _ => Err(()),
        }
    }

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

/// http://dev.w3.org/csswg/css-transitions/#animtype-font-weight
impl Animatable for FontWeight {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        let a = (*self as u32) as f64;
        let b = (*other as u32) as f64;
        let weight = a * self_portion + b * other_portion;
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

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        let a = (*self as u32) as f64;
        let b = (*other as u32) as f64;
        a.compute_distance(&b)
    }
}

/// https://drafts.csswg.org/css-fonts/#font-stretch-prop
impl Animatable for FontStretch {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
        -> Result<Self, ()>
    {
        let from = f64::from(*self);
        let to = f64::from(*other);
        // FIXME: When `const fn` is available in release rust, make |normal|, below, const.
        let normal = f64::from(FontStretch::normal);
        let result = (from - normal) * self_portion + (to - normal) * other_portion + normal;

        Ok(result.into())
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        let from = f64::from(*self);
        let to   = f64::from(*other);
        from.compute_distance(&to)
    }
}

/// We should treat font stretch as real number in order to interpolate this property.
/// https://drafts.csswg.org/css-fonts-3/#font-stretch-animation
impl From<FontStretch> for f64 {
    fn from(stretch: FontStretch) -> f64 {
        use self::FontStretch::*;
        match stretch {
            ultra_condensed => 1.0,
            extra_condensed => 2.0,
            condensed       => 3.0,
            semi_condensed  => 4.0,
            normal          => 5.0,
            semi_expanded   => 6.0,
            expanded        => 7.0,
            extra_expanded  => 8.0,
            ultra_expanded  => 9.0,
        }
    }
}

impl Into<FontStretch> for f64 {
    fn into(self) -> FontStretch {
        use properties::longhands::font_stretch::computed_value::T::*;
        let index = (self + 0.5).floor().min(9.0).max(1.0);
        static FONT_STRETCH_ENUM_MAP: [FontStretch; 9] =
            [ ultra_condensed, extra_condensed, condensed, semi_condensed, normal,
              semi_expanded, expanded, extra_expanded, ultra_expanded ];
        FONT_STRETCH_ENUM_MAP[(index - 1.0) as usize]
    }
}

/// https://drafts.csswg.org/css-transitions/#animtype-simple-list
impl<H: Animatable, V: Animatable> Animatable for generic_position::Position<H, V> {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(generic_position::Position {
            horizontal: try!(self.horizontal.add_weighted(&other.horizontal,
                                                          self_portion, other_portion)),
            vertical: try!(self.vertical.add_weighted(&other.vertical,
                                                      self_portion, other_portion)),
        })
    }

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

impl<H, V> RepeatableListAnimatable for generic_position::Position<H, V>
    where H: RepeatableListAnimatable, V: RepeatableListAnimatable {}

/// https://drafts.csswg.org/css-transitions/#animtype-rect
impl Animatable for ClipRect {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
        -> Result<Self, ()> {
        Ok(ClipRect {
            top: try!(self.top.add_weighted(&other.top, self_portion, other_portion)),
            right: try!(self.right.add_weighted(&other.right, self_portion, other_portion)),
            bottom: try!(self.bottom.add_weighted(&other.bottom, self_portion, other_portion)),
            left: try!(self.left.add_weighted(&other.left, self_portion, other_portion)),
        })
    }

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

<%def name="impl_animatable_for_shadow(item, transparent_color)">
    impl Animatable for ${item} {
        #[inline]
        fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
            % if "Box" in item:
            // It can't be interpolated if inset does not match.
            if self.inset != other.inset {
                return Err(());
            }
            % endif

            let x = try!(self.offset_x.add_weighted(&other.offset_x, self_portion, other_portion));
            let y = try!(self.offset_y.add_weighted(&other.offset_y, self_portion, other_portion));
            let color = try!(self.color.add_weighted(&other.color, self_portion, other_portion));
            let blur = try!(self.blur_radius.add_weighted(&other.blur_radius,
                                                          self_portion, other_portion));
            % if "Box" in item:
            let spread = try!(self.spread_radius.add_weighted(&other.spread_radius,
                                                              self_portion, other_portion));
            % endif

            Ok(${item} {
                offset_x: x,
                offset_y: y,
                blur_radius: blur,
                color: color,
                % if "Box" in item:
                spread_radius: spread,
                inset: self.inset,
                % endif
            })
        }

        #[inline]
        fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
            self.compute_squared_distance(other).map(|sd| sd.sqrt())
        }

        #[inline]
        fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
            % if "Box" in item:
            if self.inset != other.inset {
                return Err(());
            }
            % endif
            let list = [ try!(self.offset_x.compute_distance(&other.offset_x)),
                         try!(self.offset_y.compute_distance(&other.offset_y)),
                         try!(self.blur_radius.compute_distance(&other.blur_radius)),
                         try!(self.color.compute_distance(&other.color)),
                         % if "Box" in item:
                         try!(self.spread_radius.compute_distance(&other.spread_radius)),
                         % endif
                       ];
            Ok(list.iter().fold(0.0f64, |sum, diff| sum + diff * diff))
        }
    }

    /// https://drafts.csswg.org/css-transitions/#animtype-shadow-list
    impl Animatable for ${item}List {
        #[inline]
        fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
            // The inset value must change
            % if "Box" in item:
            let mut zero = ${item} {
            % else:
            let zero = ${item} {
            % endif
                offset_x: Au(0),
                offset_y: Au(0),
                blur_radius: Au(0),
                color: ${transparent_color},
                % if "Box" in item:
                spread_radius: Au(0),
                inset: false,
                % endif
            };

            let max_len = cmp::max(self.0.len(), other.0.len());

            let mut result = if max_len > 1 {
                SmallVec::from_vec(Vec::with_capacity(max_len))
            } else {
                SmallVec::new()
            };

            for i in 0..max_len {
                let shadow = match (self.0.get(i), other.0.get(i)) {
                    (Some(shadow), Some(other))
                        => try!(shadow.add_weighted(other, self_portion, other_portion)),
                    (Some(shadow), None) => {
                        % if "Box" in item:
                        zero.inset = shadow.inset;
                        % endif
                        shadow.add_weighted(&zero, self_portion, other_portion).unwrap()
                    }
                    (None, Some(shadow)) => {
                        % if "Box" in item:
                        zero.inset = shadow.inset;
                        % endif
                        zero.add_weighted(&shadow, self_portion, other_portion).unwrap()
                    }
                    (None, None) => unreachable!(),
                };
                result.push(shadow);
            }

            Ok(${item}List(result))
        }

        fn add(&self, other: &Self) -> Result<Self, ()> {
            let len = self.0.len() + other.0.len();

            let mut result = if len > 1 {
                SmallVec::from_vec(Vec::with_capacity(len))
            } else {
                SmallVec::new()
            };

            result.extend(self.0.iter().cloned());
            result.extend(other.0.iter().cloned());

            Ok(${item}List(result))
        }
    }
</%def>

${impl_animatable_for_shadow('BoxShadow', 'CSSParserColor::RGBA(RGBA::transparent())',)}
${impl_animatable_for_shadow('TextShadow', 'CSSParserColor::RGBA(RGBA::transparent())',)}

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

/// A wrapper for calling add_weighted that interpolates the distance of the two values from
/// an initial_value and uses that to produce an interpolated value.
/// This is used for values such as 'scale' where the initial value is 1 and where if we interpolate
/// the absolute values, we will produce odd results for accumulation.
fn add_weighted_with_initial_val<T: Animatable>(a: &T,
                                                b: &T,
                                                a_portion: f64,
                                                b_portion: f64,
                                                initial_val: &T) -> Result<T, ()> {
    let a = try!(a.add_weighted(&initial_val, 1.0, -1.0));
    let b = try!(b.add_weighted(&initial_val, 1.0, -1.0));
    let result = try!(a.add_weighted(&b, a_portion, b_portion));
    result.add_weighted(&initial_val, 1.0, 1.0)
}

/// Add two transform lists.
/// http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
fn add_weighted_transform_lists(from_list: &[TransformOperation],
                                to_list: &[TransformOperation],
                                self_portion: f64,
                                other_portion: f64) -> TransformList {
    let mut result = vec![];

    if can_interpolate_list(from_list, to_list) {
        for (from, to) in from_list.iter().zip(to_list) {
            match (from, to) {
                (&TransformOperation::Matrix(from),
                 &TransformOperation::Matrix(_to)) => {
                    let sum = from.add_weighted(&_to, self_portion, other_portion).unwrap();
                    result.push(TransformOperation::Matrix(sum));
                }
                (&TransformOperation::MatrixWithPercents(_),
                 &TransformOperation::MatrixWithPercents(_)) => {
                    // We don't add_weighted `-moz-transform` matrices yet.
                    // They contain percentage values.
                    {}
                }
                (&TransformOperation::Skew(fx, fy),
                 &TransformOperation::Skew(tx, ty)) => {
                    let ix = fx.add_weighted(&tx, self_portion, other_portion).unwrap();
                    let iy = fy.add_weighted(&ty, self_portion, other_portion).unwrap();
                    result.push(TransformOperation::Skew(ix, iy));
                }
                (&TransformOperation::Translate(fx, fy, fz),
                 &TransformOperation::Translate(tx, ty, tz)) => {
                    let ix = fx.add_weighted(&tx, self_portion, other_portion).unwrap();
                    let iy = fy.add_weighted(&ty, self_portion, other_portion).unwrap();
                    let iz = fz.add_weighted(&tz, self_portion, other_portion).unwrap();
                    result.push(TransformOperation::Translate(ix, iy, iz));
                }
                (&TransformOperation::Scale(fx, fy, fz),
                 &TransformOperation::Scale(tx, ty, tz)) => {
                    let ix = add_weighted_with_initial_val(&fx, &tx, self_portion,
                                                           other_portion, &1.0).unwrap();
                    let iy = add_weighted_with_initial_val(&fy, &ty, self_portion,
                                                           other_portion, &1.0).unwrap();
                    let iz = add_weighted_with_initial_val(&fz, &tz, self_portion,
                                                           other_portion, &1.0).unwrap();
                    result.push(TransformOperation::Scale(ix, iy, iz));
                }
                (&TransformOperation::Rotate(fx, fy, fz, fa),
                 &TransformOperation::Rotate(tx, ty, tz, ta)) => {
                    let norm_f = ((fx * fx) + (fy * fy) + (fz * fz)).sqrt();
                    let norm_t = ((tx * tx) + (ty * ty) + (tz * tz)).sqrt();
                    let (fx, fy, fz) = (fx / norm_f, fy / norm_f, fz / norm_f);
                    let (tx, ty, tz) = (tx / norm_t, ty / norm_t, tz / norm_t);
                    if fx == tx && fy == ty && fz == tz {
                        let ia = fa.add_weighted(&ta, self_portion, other_portion).unwrap();
                        result.push(TransformOperation::Rotate(fx, fy, fz, ia));
                    } else {
                        let matrix_f = rotate_to_matrix(fx, fy, fz, fa);
                        let matrix_t = rotate_to_matrix(tx, ty, tz, ta);
                        let sum = matrix_f.add_weighted(&matrix_t, self_portion, other_portion)
                                          .unwrap();

                        result.push(TransformOperation::Matrix(sum));
                    }
                }
                (&TransformOperation::Perspective(fd),
                 &TransformOperation::Perspective(_td)) => {
                    let mut fd_matrix = ComputedMatrix::identity();
                    let mut td_matrix = ComputedMatrix::identity();
                    fd_matrix.m43 = -1. / fd.to_f32_px();
                    td_matrix.m43 = -1. / _td.to_f32_px();
                    let sum = fd_matrix.add_weighted(&td_matrix, self_portion, other_portion)
                                       .unwrap();
                    result.push(TransformOperation::Matrix(sum));
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

impl Animatable for InnerMatrix2D {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(InnerMatrix2D {
            m11: try!(add_weighted_with_initial_val(&self.m11, &other.m11,
                                                    self_portion, other_portion, &1.0)),
            m12: try!(self.m12.add_weighted(&other.m12, self_portion, other_portion)),
            m21: try!(self.m21.add_weighted(&other.m21, self_portion, other_portion)),
            m22: try!(add_weighted_with_initial_val(&self.m22, &other.m22,
                                                    self_portion, other_portion, &1.0)),
        })
    }
}

impl Animatable for Translate2D {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Translate2D(
            try!(self.0.add_weighted(&other.0, self_portion, other_portion)),
            try!(self.1.add_weighted(&other.1, self_portion, other_portion))
        ))
    }
}

impl Animatable for Scale2D {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Scale2D(
            try!(add_weighted_with_initial_val(&self.0, &other.0, self_portion, other_portion, &1.0)),
            try!(add_weighted_with_initial_val(&self.1, &other.1, self_portion, other_portion, &1.0))
        ))
    }
}

impl Animatable for MatrixDecomposed2D {
    /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-2d-matrix-values
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
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
        let translate = try!(self.translate.add_weighted(&other.translate,
                                                         self_portion, other_portion));
        let scale = try!(scale.add_weighted(&other.scale, self_portion, other_portion));
        let angle = try!(angle.add_weighted(&other_angle, self_portion, other_portion));
        let matrix = try!(self.matrix.add_weighted(&other.matrix, self_portion, other_portion));

        Ok(MatrixDecomposed2D {
            translate: translate,
            scale: scale,
            angle: angle,
            matrix: matrix,
        })
    }
}

impl Animatable for ComputedMatrix {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        if self.is_3d() || other.is_3d() {
            let decomposed_from = decompose_3d_matrix(*self);
            let decomposed_to = decompose_3d_matrix(*other);
            match (decomposed_from, decomposed_to) {
                (Ok(from), Ok(to)) => {
                    let sum = try!(from.add_weighted(&to, self_portion, other_portion));
                    Ok(ComputedMatrix::from(sum))
                },
                _ => {
                    let result = if self_portion > other_portion {*self} else {*other};
                    Ok(result)
                }
            }
        } else {
            let decomposed_from = MatrixDecomposed2D::from(*self);
            let decomposed_to = MatrixDecomposed2D::from(*other);
            let sum = try!(decomposed_from.add_weighted(&decomposed_to,
                                                        self_portion, other_portion));
            Ok(ComputedMatrix::from(sum))
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

impl Animatable for Translate3D {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Translate3D(
            try!(self.0.add_weighted(&other.0, self_portion, other_portion)),
            try!(self.1.add_weighted(&other.1, self_portion, other_portion)),
            try!(self.2.add_weighted(&other.2, self_portion, other_portion))
        ))
    }
}

impl Animatable for Scale3D {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Scale3D(
            try!(add_weighted_with_initial_val(&self.0, &other.0, self_portion, other_portion, &1.0)),
            try!(add_weighted_with_initial_val(&self.1, &other.1, self_portion, other_portion, &1.0)),
            try!(add_weighted_with_initial_val(&self.2, &other.2, self_portion, other_portion, &1.0))
        ))
    }
}

impl Animatable for Skew {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Skew(
            try!(self.0.add_weighted(&other.0, self_portion, other_portion)),
            try!(self.1.add_weighted(&other.1, self_portion, other_portion)),
            try!(self.2.add_weighted(&other.2, self_portion, other_portion))
        ))
    }
}

impl Animatable for Perspective {
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        Ok(Perspective(
            try!(self.0.add_weighted(&other.0, self_portion, other_portion)),
            try!(self.1.add_weighted(&other.1, self_portion, other_portion)),
            try!(self.2.add_weighted(&other.2, self_portion, other_portion)),
            try!(add_weighted_with_initial_val(&self.3, &other.3, self_portion, other_portion, &1.0))
        ))
    }
}

impl Animatable for MatrixDecomposed3D {
    /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-3d-matrix-values
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64)
        -> Result<Self, ()> {
        assert!(self_portion + other_portion == 1.0f64 ||
                other_portion == 1.0f64,
                "add_weighted should only be used for interpolating or accumulating transforms");

        let mut sum = *self;

        // Add translate, scale, skew and perspective components.
        sum.translate = try!(self.translate.add_weighted(&other.translate,
                                                         self_portion, other_portion));
        sum.scale = try!(self.scale.add_weighted(&other.scale, self_portion, other_portion));
        sum.skew = try!(self.skew.add_weighted(&other.skew, self_portion, other_portion));
        sum.perspective = try!(self.perspective.add_weighted(&other.perspective,
                                                             self_portion, other_portion));

        // Add quaternions using spherical linear interpolation (Slerp).
        //
        // We take a specialized code path for accumulation (where other_portion is 1)
        if other_portion == 1.0 {
            if self_portion == 0.0 {
                return Ok(*other)
            }

            let clamped_w = self.quaternion.3.min(1.0).max(-1.0);

            // Determine the scale factor.
            let mut theta = clamped_w.acos();
            let mut scale = if theta == 0.0 { 0.0 } else { 1.0 / theta.sin() };
            theta *= self_portion as f32;
            scale *= theta.sin();

            // Scale the self matrix by self_portion.
            let mut scaled_self = *self;
            % for i in range(3):
                scaled_self.quaternion.${i} *= scale;
            % endfor
            scaled_self.quaternion.3 = theta.cos();

            // Multiply scaled-self by other.
            let a = &scaled_self.quaternion;
            let b = &other.quaternion;
            sum.quaternion = Quaternion(
                a.3 * b.0 + a.0 * b.3 + a.1 * b.2 - a.2 * b.1,
                a.3 * b.1 - a.0 * b.2 + a.1 * b.3 + a.2 * b.0,
                a.3 * b.2 + a.0 * b.1 - a.1 * b.0 + a.2 * b.3,
                a.3 * b.3 - a.0 * b.0 - a.1 * b.1 - a.2 * b.2,
            );
        } else {
            let mut product = self.quaternion.0 * other.quaternion.0 +
                              self.quaternion.1 * other.quaternion.1 +
                              self.quaternion.2 * other.quaternion.2 +
                              self.quaternion.3 * other.quaternion.3;

            // Clamp product to -1.0 <= product <= 1.0
            product = product.min(1.0);
            product = product.max(-1.0);

            if product == 1.0 {
                return Ok(sum);
            }

            let theta = product.acos();
            let w = (other_portion as f32 * theta).sin() * 1.0 / (1.0 - product * product).sqrt();

            let mut a = *self;
            let mut b = *other;
            % for i in range(4):
                a.quaternion.${i} *= (other_portion as f32 * theta).cos() - product * w;
                b.quaternion.${i} *= w;
                sum.quaternion.${i} = a.quaternion.${i} + b.quaternion.${i};
            % endfor
        }

        Ok(sum)
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
impl Animatable for TransformList {
    #[inline]
    fn add_weighted(&self, other: &TransformList, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        // http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
        let result = match (&self.0, &other.0) {
            (&Some(ref from_list), &Some(ref to_list)) => {
                // Two lists of transforms
                add_weighted_transform_lists(from_list, &to_list, self_portion, other_portion)
            }
            (&Some(ref from_list), &None) => {
                // http://dev.w3.org/csswg/css-transforms/#none-transform-animation
                let to_list = build_identity_transform_list(from_list);
                add_weighted_transform_lists(from_list, &to_list, self_portion, other_portion)
            }
            (&None, &Some(ref to_list)) => {
                // http://dev.w3.org/csswg/css-transforms/#none-transform-animation
                let from_list = build_identity_transform_list(to_list);
                add_weighted_transform_lists(&from_list, to_list, self_portion, other_portion)
            }
            _ => {
                // http://dev.w3.org/csswg/css-transforms/#none-none-animation
                TransformList(None)
            }
        };

        Ok(result)
    }

    fn add(&self, other: &Self) -> Result<Self, ()> {
        match (&self.0, &other.0) {
            (&Some(ref from_list), &Some(ref to_list)) => {
                Ok(TransformList(Some([&from_list[..], &to_list[..]].concat())))
            }
            (&Some(_), &None) => {
                Ok(self.clone())
            }
            (&None, &Some(_)) => {
                Ok(other.clone())
            }
            _ => {
                Ok(TransformList(None))
            }
        }
    }
}

impl<T, U> Animatable for Either<T, U>
        where T: Animatable + Copy, U: Animatable + Copy,
{
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (Either::First(ref this), Either::First(ref other)) => {
                this.add_weighted(&other, self_portion, other_portion).map(Either::First)
            },
            (Either::Second(ref this), Either::Second(ref other)) => {
                this.add_weighted(&other, self_portion, other_portion).map(Either::Second)
            },
            _ => {
                let result = if self_portion > other_portion {*self} else {*other};
                Ok(result)
            }
        }
    }

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

impl <'a> From<<&'a IntermediateRGBA> for RGBA {
    fn from(extended_rgba: &IntermediateRGBA) -> RGBA {
        // RGBA::from_floats clamps each component values.
        RGBA::from_floats(extended_rgba.red,
                          extended_rgba.green,
                          extended_rgba.blue,
                          extended_rgba.alpha)
    }
}

impl <'a> From<<&'a RGBA> for IntermediateRGBA {
    fn from(rgba: &RGBA) -> IntermediateRGBA {
        IntermediateRGBA::new(rgba.red_f32(),
                              rgba.green_f32(),
                              rgba.blue_f32(),
                              rgba.alpha_f32())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// Unlike RGBA, each component value may exceed the range [0.0, 1.0].
pub struct IntermediateRGBA {
    /// The red component.
    pub red: f32,
    /// The green component.
    pub green: f32,
    /// The blue component.
    pub blue: f32,
    /// The alpha component.
    pub alpha: f32,
}

impl IntermediateRGBA {
    /// Returns a transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self::new(0., 0., 0., 0.)
    }

    /// Returns a new color.
    #[inline]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        IntermediateRGBA { red: red, green: green, blue: blue, alpha: alpha }
    }
}

/// Unlike Animatable for RGBA we don't clamp any component values.
impl Animatable for IntermediateRGBA {
    #[inline]
    fn add_weighted(&self, other: &IntermediateRGBA, self_portion: f64, other_portion: f64)
        -> Result<Self, ()> {
        let mut alpha = try!(self.alpha.add_weighted(&other.alpha, self_portion, other_portion));
        if alpha <= 0. {
            // Ideally we should return color value that only alpha component is
            // 0, but this is what current gecko does.
            Ok(IntermediateRGBA::transparent())
        } else {
            alpha = alpha.min(1.);
            let red = try!((self.red * self.alpha)
                            .add_weighted(&(other.red * other.alpha),
                                          self_portion, other_portion))
                            * 1. / alpha;
            let green = try!((self.green * self.alpha)
                             .add_weighted(&(other.green * other.alpha),
                                           self_portion, other_portion))
                             * 1. / alpha;
            let blue = try!((self.blue * self.alpha)
                             .add_weighted(&(other.blue * other.alpha),
                                           self_portion, other_portion))
                             * 1. / alpha;
            Ok(IntermediateRGBA::new(red, green, blue, alpha))
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sq| sq.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        let start = [ self.alpha,
                      self.red * self.alpha,
                      self.green * self.alpha,
                      self.blue * self.alpha ];
        let end = [ other.alpha,
                    other.red * other.alpha,
                    other.green * other.alpha,
                    other.blue * other.alpha ];
        let diff = start.iter().zip(&end)
                               .fold(0.0f64, |n, (&a, &b)| {
                                   let diff = (a - b) as f64;
                                   n + diff * diff
                               });
        Ok(diff)
    }
}

impl<'a> From<<&'a Either<CSSParserColor, Auto>> for Either<IntermediateColor, Auto> {
    fn from(from: &Either<CSSParserColor, Auto>) -> Either<IntermediateColor, Auto> {
        match *from {
            Either::First(ref from) =>
                match *from {
                    CSSParserColor::RGBA(ref color) =>
                        Either::First(IntermediateColor::IntermediateRGBA(
                            IntermediateRGBA::new(color.red_f32(),
                                                  color.green_f32(),
                                                  color.blue_f32(),
                                                  color.alpha_f32()))),
                    CSSParserColor::CurrentColor =>
                        Either::First(IntermediateColor::CurrentColor),
                },
            Either::Second(Auto) => Either::Second(Auto),
        }
    }
}

impl<'a> From<<&'a Either<IntermediateColor, Auto>> for Either<CSSParserColor, Auto> {
    fn from(from: &Either<IntermediateColor, Auto>) -> Either<CSSParserColor, Auto> {
        match *from {
            Either::First(ref from) =>
                match *from {
                    IntermediateColor::IntermediateRGBA(ref color) =>
                        Either::First(CSSParserColor::RGBA(RGBA::from_floats(color.red,
                                                                             color.green,
                                                                             color.blue,
                                                                             color.alpha))),
                    IntermediateColor::CurrentColor =>
                        Either::First(CSSParserColor::CurrentColor),
                },
            Either::Second(Auto) => Either::Second(Auto),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum IntermediateColor {
    CurrentColor,
    IntermediateRGBA(IntermediateRGBA),
}

impl Animatable for IntermediateColor {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        match (*self, *other) {
            (IntermediateColor::IntermediateRGBA(ref this),
             IntermediateColor::IntermediateRGBA(ref other)) => {
                this.add_weighted(other, self_portion, other_portion)
                    .map(IntermediateColor::IntermediateRGBA)
            }
            // FIXME: Bug 1345709: Implement currentColor animations.
            _ => Err(()),
        }
    }

    #[inline]
    fn compute_distance(&self, other: &Self) -> Result<f64, ()> {
        self.compute_squared_distance(other).map(|sq| sq.sqrt())
    }

    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<f64, ()> {
        match (*self, *other) {
            (IntermediateColor::IntermediateRGBA(ref this), IntermediateColor::IntermediateRGBA(ref other)) => {
                this.compute_squared_distance(other)
            },
            _ => Ok(0.0),
        }
    }
}

impl <'a> From<<&'a CSSParserColor> for IntermediateColor {
    fn from(color: &CSSParserColor) -> IntermediateColor {
        match *color {
            CSSParserColor::RGBA(ref color) =>
                IntermediateColor::IntermediateRGBA(IntermediateRGBA::new(color.red_f32(),
                                                                          color.green_f32(),
                                                                          color.blue_f32(),
                                                                          color.alpha_f32())),
            CSSParserColor::CurrentColor => IntermediateColor::CurrentColor,
        }
    }
}

impl <'a> From<<&'a IntermediateColor> for CSSParserColor {
    fn from(color: &IntermediateColor) -> CSSParserColor {
        match *color {
            IntermediateColor::IntermediateRGBA(ref color) =>
                CSSParserColor::RGBA(RGBA::from_floats(color.red,
                                                       color.green,
                                                       color.blue,
                                                       color.alpha)),
            IntermediateColor::CurrentColor => CSSParserColor::CurrentColor,
        }
    }
}

<%def name="impl_intermediate_type_for_shadow(type)">
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[allow(missing_docs)]
    /// Intermediate type for box-shadow and text-shadow.
    /// The difference between normal shadow type is that this type uses
    /// IntermediateColor instead of ParserColor.
    pub struct Intermediate${type}Shadow {
        pub offset_x: Au,
        pub offset_y: Au,
        pub blur_radius: Au,
        pub color: IntermediateColor,
        % if type == "Box":
        pub spread_radius: Au,
        pub inset: bool,
        % endif
    }

    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[allow(missing_docs)]
    /// Intermediate type for box-shadow list and text-shadow list.
    pub struct Intermediate${type}ShadowList(pub SmallVec<[Intermediate${type}Shadow; 1]>);

    impl <'a> From<<&'a Intermediate${type}ShadowList> for ${type}ShadowList {
        fn from(shadow_list: &Intermediate${type}ShadowList) -> ${type}ShadowList {
            ${type}ShadowList(shadow_list.0.iter().map(|s| s.into()).collect())
        }
    }

    impl <'a> From<<&'a ${type}ShadowList> for Intermediate${type}ShadowList {
        fn from(shadow_list: &${type}ShadowList) -> Intermediate${type}ShadowList {
            Intermediate${type}ShadowList(shadow_list.0.iter().map(|s| s.into()).collect())
        }
    }

    impl <'a> From<<&'a Intermediate${type}Shadow> for ${type}Shadow {
        fn from(shadow: &Intermediate${type}Shadow) -> ${type}Shadow {
            ${type}Shadow {
                offset_x: shadow.offset_x,
                offset_y: shadow.offset_y,
                blur_radius: shadow.blur_radius,
                color: (&shadow.color).into(),
                % if type == "Box":
                spread_radius: shadow.spread_radius,
                inset: shadow.inset,
                % endif
            }
        }
    }

    impl <'a> From<<&'a ${type}Shadow> for Intermediate${type}Shadow {
        fn from(shadow: &${type}Shadow) -> Intermediate${type}Shadow {
            Intermediate${type}Shadow {
                offset_x: shadow.offset_x,
                offset_y: shadow.offset_y,
                blur_radius: shadow.blur_radius,
                color: (&shadow.color).into(),
                % if type == "Box":
                spread_radius: shadow.spread_radius,
                inset: shadow.inset,
                % endif
            }
        }
    }
    ${impl_animatable_for_shadow('Intermediate%sShadow' % type,
                                 'IntermediateColor::IntermediateRGBA(IntermediateRGBA::transparent())')}
</%def>

${impl_intermediate_type_for_shadow('Box')}
${impl_intermediate_type_for_shadow('Text')}
