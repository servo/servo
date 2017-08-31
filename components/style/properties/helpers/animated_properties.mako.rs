/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% from data import to_idl_name, SYSTEM_FONT_LONGHANDS %>

use app_units::Au;
use cssparser::Parser;
#[cfg(feature = "gecko")] use gecko_bindings::bindings::RawServoAnimationValueMap;
#[cfg(feature = "gecko")] use gecko_bindings::structs::RawGeckoGfxMatrix4x4;
#[cfg(feature = "gecko")] use gecko_bindings::structs::nsCSSPropertyID;
#[cfg(feature = "gecko")] use gecko_bindings::sugar::ownership::{HasFFI, HasSimpleFFI};
#[cfg(feature = "gecko")] use gecko_string_cache::Atom;
use itertools::{EitherOrBoth, Itertools};
use properties::{CSSWideKeyword, PropertyDeclaration};
use properties::longhands;
use properties::longhands::background_size::computed_value::T as BackgroundSizeList;
use properties::longhands::border_spacing::computed_value::T as BorderSpacing;
use properties::longhands::font_weight::computed_value::T as FontWeight;
use properties::longhands::font_stretch::computed_value::T as FontStretch;
use properties::longhands::line_height::computed_value::T as LineHeight;
use properties::longhands::transform::computed_value::ComputedMatrix;
use properties::longhands::transform::computed_value::ComputedOperation as TransformOperation;
use properties::longhands::transform::computed_value::T as TransformList;
use properties::longhands::vertical_align::computed_value::T as VerticalAlign;
use properties::longhands::visibility::computed_value::T as Visibility;
#[cfg(feature = "gecko")] use properties::{PropertyId, PropertyDeclarationId, LonghandId};
#[cfg(feature = "gecko")] use properties::{ShorthandId};
use selectors::parser::SelectorParseError;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cmp;
#[cfg(feature = "gecko")] use fnv::FnvHashMap;
use style_traits::ParseError;
use super::ComputedValues;
#[cfg(feature = "gecko")]
use values::Auto;
use values::{CSSFloat, CustomIdent, Either};
use values::animated::{Animate, Procedure, ToAnimatedValue, ToAnimatedZero};
use values::animated::color::{Color as AnimatedColor, RGBA as AnimatedRGBA};
use values::animated::effects::BoxShadowList as AnimatedBoxShadowList;
use values::animated::effects::Filter as AnimatedFilter;
use values::animated::effects::FilterList as AnimatedFilterList;
use values::animated::effects::TextShadowList as AnimatedTextShadowList;
use values::computed::{Angle, BorderCornerRadius, CalcLengthOrPercentage};
use values::computed::{ClipRect, Context, ComputedUrl, ComputedValueAsSpecified};
use values::computed::{LengthOrPercentage, LengthOrPercentageOrAuto};
use values::computed::{LengthOrPercentageOrNone, MaxLength, NonNegativeAu};
use values::computed::{NonNegativeNumber, Number, NumberOrPercentage, Percentage};
use values::computed::{PositiveIntegerOrAuto, ToComputedValue};
#[cfg(feature = "gecko")] use values::computed::MozLength;
use values::computed::length::{NonNegativeLengthOrAuto, NonNegativeLengthOrNormal};
use values::computed::length::NonNegativeLengthOrPercentage;
use values::computed::transform::DirectionVector;
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::NonNegative;
use values::generics::effects::Filter;
use values::generics::position as generic_position;
use values::generics::svg::{SVGLength,  SvgLengthOrPercentageOrNumber, SVGPaint};
use values::generics::svg::{SVGPaintKind, SVGStrokeDashArray, SVGOpacity};

/// https://drafts.csswg.org/css-transitions/#animtype-repeatable-list
pub trait RepeatableListAnimatable: Animate {}

/// A longhand property whose animation type is not "none".
///
/// NOTE: This includes the 'display' property since it is animatable from SMIL even though it is
/// not animatable from CSS animations or Web Animations. CSS transitions also does not allow
/// animating 'display', but for CSS transitions we have the separate TransitionProperty type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AnimatableLonghand {
    % for prop in data.longhands:
        % if prop.animatable:
            /// ${prop.name}
            ${prop.camel_case},
        % endif
    % endfor
}

impl AnimatableLonghand {
    /// Returns true if this AnimatableLonghand is one of the discretely animatable properties.
    pub fn is_discrete(&self) -> bool {
        match *self {
            % for prop in data.longhands:
                % if prop.animation_value_type == "discrete":
                    AnimatableLonghand::${prop.camel_case} => true,
                % endif
            % endfor
            _ => false
        }
    }

    /// Converts from an nsCSSPropertyID. Returns None if nsCSSPropertyID is not an animatable
    /// longhand in Servo.
    #[cfg(feature = "gecko")]
    pub fn from_nscsspropertyid(css_property: nsCSSPropertyID) -> Option<Self> {
        match css_property {
            % for prop in data.longhands:
                % if prop.animatable:
                    ${helpers.to_nscsspropertyid(prop.ident)}
                        => Some(AnimatableLonghand::${prop.camel_case}),
                % endif
            % endfor
            _ => None
        }
    }

    /// Converts from TransitionProperty. Returns None if the property is not an animatable
    /// longhand.
    pub fn from_transition_property(transition_property: &TransitionProperty) -> Option<Self> {
        match *transition_property {
            % for prop in data.longhands:
                % if prop.transitionable and prop.animatable:
                    TransitionProperty::${prop.camel_case}
                        => Some(AnimatableLonghand::${prop.camel_case}),
                % endif
            % endfor
            _ => None
        }
    }

    /// Get an animatable longhand property from a property declaration.
    pub fn from_declaration(declaration: &PropertyDeclaration) -> Option<Self> {
        use properties::LonghandId;
        match *declaration {
            % for prop in data.longhands:
                % if prop.animatable:
                    PropertyDeclaration::${prop.camel_case}(..)
                        => Some(AnimatableLonghand::${prop.camel_case}),
                % endif
            % endfor
            PropertyDeclaration::CSSWideKeyword(id, _) |
            PropertyDeclaration::WithVariables(id, _) => {
                match id {
                    % for prop in data.longhands:
                        % if prop.animatable:
                            LonghandId::${prop.camel_case} =>
                                Some(AnimatableLonghand::${prop.camel_case}),
                        % endif
                    % endfor
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

/// Convert to nsCSSPropertyID.
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl<'a> From< &'a AnimatableLonghand> for nsCSSPropertyID {
    fn from(property: &'a AnimatableLonghand) -> nsCSSPropertyID {
        match *property {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatableLonghand::${prop.camel_case}
                        => ${helpers.to_nscsspropertyid(prop.ident)},
                % endif
            % endfor
        }
    }
}

/// Convert to PropertyDeclarationId.
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl<'a> From<AnimatableLonghand> for PropertyDeclarationId<'a> {
    fn from(property: AnimatableLonghand) -> PropertyDeclarationId<'a> {
        match property {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatableLonghand::${prop.camel_case}
                        => PropertyDeclarationId::Longhand(LonghandId::${prop.camel_case}),
                % endif
            % endfor
        }
    }
}

/// Returns true if this nsCSSPropertyID is one of the animatable properties.
#[cfg(feature = "gecko")]
pub fn nscsspropertyid_is_animatable(property: nsCSSPropertyID) -> bool {
    match property {
        % for prop in data.longhands + data.shorthands_except_all():
            % if prop.animatable:
                ${helpers.to_nscsspropertyid(prop.ident)} => true,
            % endif
        % endfor
        _ => false
    }
}

/// A given transition property, that is either `All`, a transitionable longhand property,
/// a shorthand with at least one transitionable longhand component, or an unsupported property.
// NB: This needs to be here because it needs all the longhands generated
// beforehand.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, ToCss)]
pub enum TransitionProperty {
    /// All, any transitionable property changing should generate a transition.
    All,
    % for prop in data.longhands + data.shorthands_except_all():
        % if prop.transitionable:
            /// ${prop.name}
            ${prop.camel_case},
        % endif
    % endfor
    /// Unrecognized property which could be any non-transitionable, custom property, or
    /// unknown property.
    Unsupported(CustomIdent)
}


impl ComputedValueAsSpecified for TransitionProperty {}

impl TransitionProperty {
    /// Iterates over each longhand property.
    pub fn each<F: FnMut(&TransitionProperty) -> ()>(mut cb: F) {
        % for prop in data.longhands:
            % if prop.transitionable:
                cb(&TransitionProperty::${prop.camel_case});
            % endif
        % endfor
    }

    /// Iterates over every longhand property that is not TransitionProperty::All, stopping and
    /// returning true when the provided callback returns true for the first time.
    pub fn any<F: FnMut(&TransitionProperty) -> bool>(mut cb: F) -> bool {
        % for prop in data.longhands:
            % if prop.transitionable:
                if cb(&TransitionProperty::${prop.camel_case}) {
                    return true;
                }
            % endif
        % endfor
        false
    }

    /// Parse a transition-property value.
    pub fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let ident = input.expect_ident()?;
        let supported = match_ignore_ascii_case! { &ident,
            "all" => Ok(Some(TransitionProperty::All)),
            % for prop in data.longhands + data.shorthands_except_all():
                % if prop.transitionable:
                    "${prop.name}" => Ok(Some(TransitionProperty::${prop.camel_case})),
                % endif
            % endfor
            "none" => Err(()),
            _ => Ok(None),
        };

        match supported {
            Ok(Some(property)) => Ok(property),
            Ok(None) => CustomIdent::from_ident(ident, &[]).map(TransitionProperty::Unsupported),
            Err(()) => Err(SelectorParseError::UnexpectedIdent(ident.clone()).into()),
        }
    }

    /// Return transitionable longhands of this shorthand TransitionProperty, except for "all".
    pub fn longhands(&self) -> &'static [TransitionProperty] {
        % for prop in data.shorthands_except_all():
            % if prop.transitionable:
                static ${prop.ident.upper()}: &'static [TransitionProperty] = &[
                    % for sub in prop.sub_properties:
                        % if sub.transitionable:
                            TransitionProperty::${sub.camel_case},
                        % endif
                    % endfor
                ];
            % endif
        % endfor
        match *self {
            % for prop in data.shorthands_except_all():
                % if prop.transitionable:
                    TransitionProperty::${prop.camel_case} => ${prop.ident.upper()},
                % endif
            % endfor
            _ => panic!("Not allowed to call longhands() for this TransitionProperty")
        }
    }

    /// Returns true if this TransitionProperty is a shorthand.
    pub fn is_shorthand(&self) -> bool {
        match *self {
            % for prop in data.shorthands_except_all():
                % if prop.transitionable:
                    TransitionProperty::${prop.camel_case} => true,
                % endif
            % endfor
            _ => false
        }
    }
}

/// Convert to nsCSSPropertyID.
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl<'a> From< &'a TransitionProperty> for nsCSSPropertyID {
    fn from(transition_property: &'a TransitionProperty) -> nsCSSPropertyID {
        match *transition_property {
            % for prop in data.longhands + data.shorthands_except_all():
                % if prop.transitionable:
                    TransitionProperty::${prop.camel_case}
                        => ${helpers.to_nscsspropertyid(prop.ident)},
                % endif
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
            % for prop in data.longhands + data.shorthands_except_all():
                % if prop.transitionable:
                    ${helpers.to_nscsspropertyid(prop.ident)}
                        => TransitionProperty::${prop.camel_case},
                % else:
                    ${helpers.to_nscsspropertyid(prop.ident)}
                        => TransitionProperty::Unsupported(CustomIdent(Atom::from("${prop.ident}"))),
                % endif
            % endfor
            nsCSSPropertyID::eCSSPropertyExtra_all_properties => TransitionProperty::All,
            _ => panic!("Unconvertable nsCSSPropertyID: {:?}", property),
        }
    }
}

/// Returns true if this nsCSSPropertyID is one of the transitionable properties.
#[cfg(feature = "gecko")]
pub fn nscsspropertyid_is_transitionable(property: nsCSSPropertyID) -> bool {
    match property {
        % for prop in data.longhands + data.shorthands_except_all():
            % if prop.transitionable:
                ${helpers.to_nscsspropertyid(prop.ident)} => true,
            % endif
        % endfor
        _ => false
    }
}

/// An animated property interpolation between two computed values for that
/// property.
#[cfg(feature = "servo")]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AnimatedProperty {
    % for prop in data.longhands:
        % if prop.animatable:
            <%
                if prop.is_animatable_with_computed_value:
                    value_type = "longhands::{}::computed_value::T".format(prop.ident)
                else:
                    value_type = prop.animation_value_type
            %>
            /// ${prop.name}
            ${prop.camel_case}(${value_type}, ${value_type}),
        % endif
    % endfor
}

#[cfg(feature = "servo")]
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
                            let value = match from.animate(to, Procedure::Interpolate { progress }) {
                                Ok(value) => value,
                                Err(()) => return,
                            };
                        % endif
                        % if not prop.is_animatable_with_computed_value:
                            let value: longhands::${prop.ident}::computed_value::T =
                                ToAnimatedValue::from_animated_value(value);
                        % endif
                        style.mutate_${prop.style_struct.name_lower}().set_${prop.ident}(value);
                    }
                % endif
            % endfor
        }
    }

    /// Get an animatable value from a transition-property, an old style, and a
    /// new style.
    pub fn from_animatable_longhand(property: &AnimatableLonghand,
                                    old_style: &ComputedValues,
                                    new_style: &ComputedValues)
                                    -> AnimatedProperty {
        match *property {
            % for prop in data.longhands:
            % if prop.animatable:
                AnimatableLonghand::${prop.camel_case} => {
                    let old_computed = old_style.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}();
                    let new_computed = new_style.get_${prop.style_struct.ident.strip("_")}().clone_${prop.ident}();
                    AnimatedProperty::${prop.camel_case}(
                    % if prop.is_animatable_with_computed_value:
                        old_computed,
                        new_computed,
                    % else:
                        old_computed.to_animated_value(),
                        new_computed.to_animated_value(),
                    % endif
                    )
                }
            % endif
            % endfor
        }
    }
}

/// A collection of AnimationValue that were composed on an element.
/// This HashMap stores the values that are the last AnimationValue to be
/// composed for each TransitionProperty.
#[cfg(feature = "gecko")]
pub type AnimationValueMap = FnvHashMap<AnimatableLonghand, AnimationValue>;
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
                                    &ToAnimatedValue::from_animated_value(from.clone())
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

    /// Construct an AnimationValue from a property declaration.
    pub fn from_declaration(
        decl: &PropertyDeclaration,
        context: &mut Context,
        initial: &ComputedValues
    ) -> Option<Self> {
        use properties::LonghandId;

        match *decl {
            % for prop in data.longhands:
            % if prop.animatable:
            PropertyDeclaration::${prop.camel_case}(ref val) => {
            % if prop.ident in SYSTEM_FONT_LONGHANDS and product == "gecko":
                if let Some(sf) = val.get_system() {
                    longhands::system_font::resolve_system_font(sf, context);
                }
            % endif
            let computed = val.to_computed_value(context);
            Some(AnimationValue::${prop.camel_case}(
            % if prop.is_animatable_with_computed_value:
                computed
            % else:
                computed.to_animated_value()
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
                                let inherit_struct = context.builder
                                                            .get_parent_${prop.style_struct.name_lower}();
                                inherit_struct.clone_${prop.ident}()
                            },
                        };
                        % if not prop.is_animatable_with_computed_value:
                        let computed = computed.to_animated_value();
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
            PropertyDeclaration::WithVariables(id, ref unparsed) => {
                let custom_props = context.style().custom_properties();
                let substituted = unparsed.substitute_variables(id, &custom_props, context.quirks_mode);
                AnimationValue::from_declaration(&substituted, context, initial)
            },
            _ => None // non animatable properties will get included because of shorthands. ignore.
        }
    }

    /// Get an AnimationValue for an AnimatableLonghand from a given computed values.
    pub fn from_computed_values(property: &AnimatableLonghand,
                                computed_values: &ComputedValues)
                                -> Self {
        match *property {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatableLonghand::${prop.camel_case} => {
                        let computed = computed_values
                            .get_${prop.style_struct.ident.strip("_")}()
                            .clone_${prop.ident}();
                        AnimationValue::${prop.camel_case}(
                        % if prop.is_animatable_with_computed_value:
                            computed
                        % else:
                            computed.to_animated_value()
                        % endif
                        )
                    }
                % endif
            % endfor
        }
    }
}

impl Animate for AnimationValue {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            % for prop in data.longhands:
            % if prop.animatable:
            % if prop.animation_value_type != "discrete":
            (
                &AnimationValue::${prop.camel_case}(ref this),
                &AnimationValue::${prop.camel_case}(ref other),
            ) => {
                Ok(AnimationValue::${prop.camel_case}(
                    this.animate(other, procedure)?,
                ))
            },
            % else:
            (
                &AnimationValue::${prop.camel_case}(ref this),
                &AnimationValue::${prop.camel_case}(ref other),
            ) => {
                if let Procedure::Interpolate { progress } = procedure {
                    Ok(AnimationValue::${prop.camel_case}(
                        if progress < 0.5 { this.clone() } else { other.clone() },
                    ))
                } else {
                    Err(())
                }
            },
            % endif
            % endif
            % endfor
            _ => {
                panic!("Unexpected AnimationValue::animate call, got: {:?}, {:?}", self, other);
            }
        }
    }
}

impl ComputeSquaredDistance for AnimationValue {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            % for prop in data.longhands:
            % if prop.animatable:
            % if prop.animation_value_type != "discrete":
            (&AnimationValue::${prop.camel_case}(ref this), &AnimationValue::${prop.camel_case}(ref other)) => {
                this.compute_squared_distance(other)
            },
            % else:
            (&AnimationValue::${prop.camel_case}(_), &AnimationValue::${prop.camel_case}(_)) => {
                Err(())
            },
            % endif
            % endif
            % endfor
            _ => {
                panic!(
                    "computed values should be of the same property, got: {:?}, {:?}",
                    self,
                    other
                );
            },
        }
    }
}

impl ToAnimatedZero for AnimationValue {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            % for prop in data.longhands:
            % if prop.animatable and prop.animation_value_type != "discrete":
            AnimationValue::${prop.camel_case}(ref base) => {
                Ok(AnimationValue::${prop.camel_case}(base.to_animated_zero()?))
            },
            % endif
            % endfor
            _ => Err(()),
        }
    }
}

impl RepeatableListAnimatable for LengthOrPercentage {}
impl RepeatableListAnimatable for Either<f32, LengthOrPercentage> {}
impl RepeatableListAnimatable for Either<NonNegativeNumber, NonNegativeLengthOrPercentage> {}
impl RepeatableListAnimatable for SvgLengthOrPercentageOrNumber<NonNegativeLengthOrPercentage, NonNegativeNumber> {}

macro_rules! repeated_vec_impl {
    ($($ty:ty),*) => {
        $(impl<T> Animate for $ty
        where
            T: RepeatableListAnimatable,
        {
            fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
                // If the length of either list is zero, the least common multiple is undefined.
                if self.is_empty() || other.is_empty() {
                    return Err(());
                }
                use num_integer::lcm;
                let len = lcm(self.len(), other.len());
                self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(this, other)| {
                    this.animate(other, procedure)
                }).collect()
            }
        }

        impl<T> ComputeSquaredDistance for $ty
        where
            T: ComputeSquaredDistance + RepeatableListAnimatable,
        {
            #[inline]
            fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
                if self.is_empty() || other.is_empty() {
                    return Err(());
                }
                use num_integer::lcm;
                let len = lcm(self.len(), other.len());
                self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(this, other)| {
                    this.compute_squared_distance(other)
                }).sum()
            }
        })*
    };
}

repeated_vec_impl!(SmallVec<[T; 1]>, Vec<T>);

/// https://drafts.csswg.org/css-transitions/#animtype-visibility
impl Animate for Visibility {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (this_weight, other_weight) = procedure.weights();
        match (*self, *other) {
            (Visibility::visible, _) => {
                Ok(if this_weight > 0.0 { *self } else { *other })
            },
            (_, Visibility::visible) => {
                Ok(if other_weight > 0.0 { *other } else { *self })
            },
            _ => Err(()),
        }
    }
}

impl ComputeSquaredDistance for Visibility {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::Value(if *self == *other { 0. } else { 1. }))
    }
}

impl ToAnimatedZero for Visibility {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

impl ToAnimatedZero for VerticalAlign {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}

/// https://drafts.csswg.org/css-transitions/#animtype-lpcalc
impl Animate for CalcLengthOrPercentage {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let animate_percentage_half = |this: Option<Percentage>, other: Option<Percentage>| {
            if this.is_none() && other.is_none() {
                return Ok(None);
            }
            let this = this.unwrap_or_default();
            let other = other.unwrap_or_default();
            Ok(Some(this.animate(&other, procedure)?))
        };

        let length = self.unclamped_length().animate(&other.unclamped_length(), procedure)?;
        let percentage = animate_percentage_half(self.percentage, other.percentage)?;
        Ok(CalcLengthOrPercentage::with_clamping_mode(length, percentage, self.clamping_mode))
    }
}

impl ToAnimatedZero for LengthOrPercentageOrAuto {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            LengthOrPercentageOrAuto::Length(_) |
            LengthOrPercentageOrAuto::Percentage(_) |
            LengthOrPercentageOrAuto::Calc(_) => {
                Ok(LengthOrPercentageOrAuto::Length(Au(0)))
            },
            LengthOrPercentageOrAuto::Auto => Err(()),
        }
    }
}

impl ToAnimatedZero for LengthOrPercentageOrNone {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            LengthOrPercentageOrNone::Length(_) |
            LengthOrPercentageOrNone::Percentage(_) |
            LengthOrPercentageOrNone::Calc(_) => {
                Ok(LengthOrPercentageOrNone::Length(Au(0)))
            },
            LengthOrPercentageOrNone::None => Err(()),
        }
    }
}

impl ToAnimatedZero for MaxLength {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}

/// http://dev.w3.org/csswg/css-transitions/#animtype-font-weight
impl Animate for FontWeight {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let a = self.0 as f64;
        let b = other.0 as f64;
        const NORMAL: f64 = 400.;
        let (this_weight, other_weight) = procedure.weights();
        let weight = (a - NORMAL) * this_weight + (b - NORMAL) * other_weight + NORMAL;
        let weight = (weight.max(100.).min(900.) / 100.).round() * 100.;
        Ok(FontWeight(weight as u16))
    }
}

impl ToAnimatedZero for FontWeight {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(FontWeight::normal())
    }
}

/// https://drafts.csswg.org/css-fonts/#font-stretch-prop
impl Animate for FontStretch {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()>
    {
        let from = f64::from(*self);
        let to = f64::from(*other);
        let normal = f64::from(FontStretch::normal);
        let (this_weight, other_weight) = procedure.weights();
        let result = (from - normal) * this_weight + (to - normal) * other_weight + normal;
        Ok(result.into())
    }
}

impl ComputeSquaredDistance for FontStretch {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        f64::from(*self).compute_squared_distance(&(*other).into())
    }
}

impl ToAnimatedZero for FontStretch {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
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

impl<H, V> RepeatableListAnimatable for generic_position::Position<H, V>
    where H: RepeatableListAnimatable, V: RepeatableListAnimatable {}

/// https://drafts.csswg.org/css-transitions/#animtype-rect
impl Animate for ClipRect {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let animate_component = |this: &Option<Au>, other: &Option<Au>| {
            match (this.animate(other, procedure)?, procedure) {
                (None, Procedure::Interpolate { .. }) => Ok(None),
                (None, _) => Err(()),
                (result, _) => Ok(result),
            }
        };

        Ok(ClipRect {
            top:    animate_component(&self.top, &other.top)?,
            right:  animate_component(&self.right, &other.right)?,
            bottom: animate_component(&self.bottom, &other.bottom)?,
            left:   animate_component(&self.left, &other.left)?,
        })
    }
}

impl ToAnimatedZero for ClipRect {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
}

/// Build an equivalent 'identity transform function list' based
/// on an existing transform list.
/// http://dev.w3.org/csswg/css-transforms/#none-transform-animation
impl ToAnimatedZero for TransformOperation {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            TransformOperation::Matrix(..) => {
                Ok(TransformOperation::Matrix(ComputedMatrix::identity()))
            },
            TransformOperation::MatrixWithPercents(..) => {
                // FIXME(nox): Should be MatrixWithPercents value.
                Ok(TransformOperation::Matrix(ComputedMatrix::identity()))
            },
            TransformOperation::Skew(sx, sy) => {
                Ok(TransformOperation::Skew(
                    sx.to_animated_zero()?,
                    sy.to_animated_zero()?,
                ))
            },
            TransformOperation::Translate(ref tx, ref ty, ref tz) => {
                Ok(TransformOperation::Translate(
                    tx.to_animated_zero()?,
                    ty.to_animated_zero()?,
                    tz.to_animated_zero()?,
                ))
            },
            TransformOperation::Scale(..) => {
                Ok(TransformOperation::Scale(1.0, 1.0, 1.0))
            },
            TransformOperation::Rotate(x, y, z, a) => {
                let (x, y, z, _) = TransformList::get_normalized_vector_and_angle(x, y, z, a);
                Ok(TransformOperation::Rotate(x, y, z, Angle::zero()))
            },
            TransformOperation::Perspective(..) |
            TransformOperation::AccumulateMatrix { .. } |
            TransformOperation::InterpolateMatrix { .. } => {
                // Perspective: We convert a perspective function into an equivalent
                //     ComputedMatrix, and then decompose/interpolate/recompose these matrices.
                // AccumulateMatrix/InterpolateMatrix: We do interpolation on
                //     AccumulateMatrix/InterpolateMatrix by reading it as a ComputedMatrix
                //     (with layout information), and then do matrix interpolation.
                //
                // Therefore, we use an identity matrix to represent the identity transform list.
                // http://dev.w3.org/csswg/css-transforms/#identity-transform-function
                //
                // FIXME(nox): This does not actually work, given the impl of
                // Animate for TransformOperation bails out if the two given
                // values are dissimilar.
                Ok(TransformOperation::Matrix(ComputedMatrix::identity()))
            },
        }
    }
}

fn animate_multiplicative_factor(
    this: CSSFloat,
    other: CSSFloat,
    procedure: Procedure,
) -> Result<CSSFloat, ()> {
    Ok((this - 1.).animate(&(other - 1.), procedure)? + 1.)
}

/// http://dev.w3.org/csswg/css-transforms/#interpolation-of-transforms
impl Animate for TransformOperation {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (
                &TransformOperation::Matrix(ref this),
                &TransformOperation::Matrix(ref other),
            ) => {
                Ok(TransformOperation::Matrix(
                    this.animate(other, procedure)?,
                ))
            },
            (
                &TransformOperation::Skew(ref fx, ref fy),
                &TransformOperation::Skew(ref tx, ref ty),
            ) => {
                Ok(TransformOperation::Skew(
                    fx.animate(tx, procedure)?,
                    fy.animate(ty, procedure)?,
                ))
            },
            (
                &TransformOperation::Translate(ref fx, ref fy, ref fz),
                &TransformOperation::Translate(ref tx, ref ty, ref tz),
            ) => {
                Ok(TransformOperation::Translate(
                    fx.animate(tx, procedure)?,
                    fy.animate(ty, procedure)?,
                    fz.animate(tz, procedure)?,
                ))
            },
            (
                &TransformOperation::Scale(ref fx, ref fy, ref fz),
                &TransformOperation::Scale(ref tx, ref ty, ref tz),
            ) => {
                Ok(TransformOperation::Scale(
                    animate_multiplicative_factor(*fx, *tx, procedure)?,
                    animate_multiplicative_factor(*fy, *ty, procedure)?,
                    animate_multiplicative_factor(*fz, *tz, procedure)?,
                ))
            },
            (
                &TransformOperation::Rotate(fx, fy, fz, fa),
                &TransformOperation::Rotate(tx, ty, tz, ta),
            ) => {
                let (fx, fy, fz, fa) =
                    TransformList::get_normalized_vector_and_angle(fx, fy, fz, fa);
                let (tx, ty, tz, ta) =
                    TransformList::get_normalized_vector_and_angle(tx, ty, tz, ta);
                if (fx, fy, fz) == (tx, ty, tz) {
                    let ia = fa.animate(&ta, procedure)?;
                    Ok(TransformOperation::Rotate(fx, fy, fz, ia))
                } else {
                    let matrix_f = rotate_to_matrix(fx, fy, fz, fa);
                    let matrix_t = rotate_to_matrix(tx, ty, tz, ta);
                    Ok(TransformOperation::Matrix(
                        matrix_f.animate(&matrix_t, procedure)?,
                    ))
                }
            },
            (
                &TransformOperation::Perspective(ref fd),
                &TransformOperation::Perspective(ref td),
            ) => {
                let mut fd_matrix = ComputedMatrix::identity();
                let mut td_matrix = ComputedMatrix::identity();
                if fd.0 > 0 {
                    fd_matrix.m34 = -1. / fd.to_f32_px();
                }
                if td.0 > 0 {
                    td_matrix.m34 = -1. / td.to_f32_px();
                }
                Ok(TransformOperation::Matrix(
                    fd_matrix.animate(&td_matrix, procedure)?,
                ))
            },
            _ => Err(()),
        }
    }
}

/// https://www.w3.org/TR/css-transforms-1/#Rotate3dDefined
fn rotate_to_matrix(x: f32, y: f32, z: f32, a: Angle) -> ComputedMatrix {
    let half_rad = a.radians() / 2.0;
    let sc = (half_rad).sin() * (half_rad).cos();
    let sq = (half_rad).sin().powi(2);

    ComputedMatrix {
        m11: 1.0 - 2.0 * (y * y + z * z) * sq,
        m12: 2.0 * (x * y * sq + z * sc),
        m13: 2.0 * (x * z * sq - y * sc),
        m14: 0.0,

        m21: 2.0 * (x * y * sq - z * sc),
        m22: 1.0 - 2.0 * (x * x + z * z) * sq,
        m23: 2.0 * (y * z * sq + x * sc),
        m24: 0.0,

        m31: 2.0 * (x * z * sq + y * sc),
        m32: 2.0 * (y * z * sq - x * sc),
        m33: 1.0 - 2.0 * (x * x + y * y) * sq,
        m34: 0.0,

        m41: 0.0,
        m42: 0.0,
        m43: 0.0,
        m44: 1.0
    }
}

/// A 2d matrix for interpolation.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
// FIXME: We use custom derive for ComputeSquaredDistance. However, If possible, we should convert
// the InnerMatrix2D into types with physical meaning. This custom derive computes the squared
// distance from each matrix item, and this makes the result different from that in Gecko if we
// have skew factor in the ComputedMatrix.
pub struct InnerMatrix2D {
    pub m11: CSSFloat, pub m12: CSSFloat,
    pub m21: CSSFloat, pub m22: CSSFloat,
}

/// A 2d translation function.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
pub struct Translate2D(f32, f32);

/// A 2d scale function.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
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

impl Animate for InnerMatrix2D {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(InnerMatrix2D {
            m11: animate_multiplicative_factor(self.m11, other.m11, procedure)?,
            m12: self.m12.animate(&other.m12, procedure)?,
            m21: self.m21.animate(&other.m21, procedure)?,
            m22: animate_multiplicative_factor(self.m22, other.m22, procedure)?,
        })
    }
}

impl Animate for Scale2D {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Scale2D(
            animate_multiplicative_factor(self.0, other.0, procedure)?,
            animate_multiplicative_factor(self.1, other.1, procedure)?,
        ))
    }
}

impl Animate for MatrixDecomposed2D {
    /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-2d-matrix-values
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
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
        let translate = self.translate.animate(&other.translate, procedure)?;
        let scale = scale.animate(&other.scale, procedure)?;
        let angle = angle.animate(&other_angle, procedure)?;
        let matrix = self.matrix.animate(&other.matrix, procedure)?;

        Ok(MatrixDecomposed2D {
            translate: translate,
            scale: scale,
            angle: angle,
            matrix: matrix,
        })
    }
}

impl ComputeSquaredDistance for MatrixDecomposed2D {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // Use Radian to compute the distance.
        const RAD_PER_DEG: f64 = ::std::f64::consts::PI / 180.0;
        let angle1 = self.angle as f64 * RAD_PER_DEG;
        let angle2 = other.angle as f64 * RAD_PER_DEG;
        Ok(self.translate.compute_squared_distance(&other.translate)? +
           self.scale.compute_squared_distance(&other.scale)? +
           angle1.compute_squared_distance(&angle2)? +
           self.matrix.compute_squared_distance(&other.matrix)?)
    }
}

impl Animate for ComputedMatrix {
    #[cfg(feature = "servo")]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.is_3d() || other.is_3d() {
            let decomposed_from = decompose_3d_matrix(*self);
            let decomposed_to = decompose_3d_matrix(*other);
            match (decomposed_from, decomposed_to) {
                (Ok(this), Ok(other)) => {
                    Ok(ComputedMatrix::from(this.animate(&other, procedure)?))
                },
                _ => {
                    let (this_weight, other_weight) = procedure.weights();
                    let result = if this_weight > other_weight { *self } else { *other };
                    Ok(result)
                },
            }
        } else {
            let this = MatrixDecomposed2D::from(*self);
            let other = MatrixDecomposed2D::from(*other);
            Ok(ComputedMatrix::from(this.animate(&other, procedure)?))
        }
    }

    #[cfg(feature = "gecko")]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (from, to) = if self.is_3d() || other.is_3d() {
            (decompose_3d_matrix(*self), decompose_3d_matrix(*other))
        } else {
            (decompose_2d_matrix(self), decompose_2d_matrix(other))
        };
        match (from, to) {
            (Ok(from), Ok(to)) => {
                Ok(ComputedMatrix::from(from.animate(&to, procedure)?))
            },
            _ => {
                let (this_weight, other_weight) = procedure.weights();
                let result = if this_weight > other_weight { *self } else { *other };
                Ok(result)
            },
        }
    }
}

impl ComputeSquaredDistance for ComputedMatrix {
    #[inline]
    #[cfg(feature = "servo")]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        if self.is_3d() || other.is_3d() {
            let from = decompose_3d_matrix(*self)?;
            let to = decompose_3d_matrix(*other)?;
            from.compute_squared_distance(&to)
        } else {
            let from = MatrixDecomposed2D::from(*self);
            let to = MatrixDecomposed2D::from(*other);
            from.compute_squared_distance(&to)
        }
    }

    #[inline]
    #[cfg(feature = "gecko")]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let (from, to) = if self.is_3d() || other.is_3d() {
            (decompose_3d_matrix(*self)?, decompose_3d_matrix(*other)?)
        } else {
            (decompose_2d_matrix(self)?, decompose_2d_matrix(other)?)
        };
        from.compute_squared_distance(&to)
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

#[cfg(feature = "gecko")]
impl<'a> From< &'a RawGeckoGfxMatrix4x4> for ComputedMatrix {
    fn from(m: &'a RawGeckoGfxMatrix4x4) -> ComputedMatrix {
        ComputedMatrix {
            m11: m[0],  m12: m[1],  m13: m[2],  m14: m[3],
            m21: m[4],  m22: m[5],  m23: m[6],  m24: m[7],
            m31: m[8],  m32: m[9],  m33: m[10], m34: m[11],
            m41: m[12], m42: m[13], m43: m[14], m44: m[15],
        }
    }
}

#[cfg(feature = "gecko")]
impl From<ComputedMatrix> for RawGeckoGfxMatrix4x4 {
    fn from(matrix: ComputedMatrix) -> RawGeckoGfxMatrix4x4 {
        [ matrix.m11, matrix.m12, matrix.m13, matrix.m14,
          matrix.m21, matrix.m22, matrix.m23, matrix.m24,
          matrix.m31, matrix.m32, matrix.m33, matrix.m34,
          matrix.m41, matrix.m42, matrix.m43, matrix.m44 ]
    }
}

/// A 3d translation.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug)]
pub struct Translate3D(f32, f32, f32);

/// A 3d scale function.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Scale3D(f32, f32, f32);

/// A 3d skew function.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, Copy, Debug)]
pub struct Skew(f32, f32, f32);

/// A 3d perspective transformation.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Perspective(f32, f32, f32, f32);

/// A quaternion used to represent a rotation.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Quaternion(f64, f64, f64, f64);

/// A decomposed 3d matrix.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
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

impl Quaternion {
    /// Return a quaternion from a unit direction vector and angle (unit: radian).
    #[inline]
    fn from_direction_and_angle(vector: &DirectionVector, angle: f64) -> Self {
        debug_assert!((vector.length() - 1.).abs() < 0.0001,
                      "Only accept an unit direction vector to create a quaternion");
        // Reference:
        // https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
        //
        // if the direction axis is (x, y, z) = xi + yj + zk,
        // and the angle is |theta|, this formula can be done using
        // an extension of Euler's formula:
        //   q = cos(theta/2) + (xi + yj + zk)(sin(theta/2))
        //     = cos(theta/2) +
        //       x*sin(theta/2)i + y*sin(theta/2)j + z*sin(theta/2)k
        Quaternion(vector.x as f64 * (angle / 2.).sin(),
                   vector.y as f64 * (angle / 2.).sin(),
                   vector.z as f64 * (angle / 2.).sin(),
                   (angle / 2.).cos())
    }

    /// Calculate the dot product.
    #[inline]
    fn dot(&self, other: &Self) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2 + self.3 * other.3
    }
}

impl ComputeSquaredDistance for Quaternion {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // Use quaternion vectors to get the angle difference. Both q1 and q2 are unit vectors,
        // so we can get their angle difference by:
        // cos(theta/2) = (q1 dot q2) / (|q1| * |q2|) = q1 dot q2.
        let distance = self.dot(other).max(-1.0).min(1.0).acos() * 2.0;
        Ok(SquaredDistance::Value(distance * distance))
    }
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
    let row1len = (row[1][0] * row[1][0] + row[1][1] * row[1][1] + row[1][2] * row[1][2]).sqrt();
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
        0.5 * ((1.0 + row[0][0] - row[1][1] - row[2][2]).max(0.0) as f64).sqrt(),
        0.5 * ((1.0 - row[0][0] + row[1][1] - row[2][2]).max(0.0) as f64).sqrt(),
        0.5 * ((1.0 - row[0][0] - row[1][1] + row[2][2]).max(0.0) as f64).sqrt(),
        0.5 * ((1.0 + row[0][0] + row[1][1] + row[2][2]).max(0.0) as f64).sqrt()
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

/// Decompose a 2D matrix for Gecko.
// Use the algorithm from nsStyleTransformMatrix::Decompose2DMatrix() in Gecko.
#[cfg(feature = "gecko")]
fn decompose_2d_matrix(matrix: &ComputedMatrix) -> Result<MatrixDecomposed3D, ()> {
    // The index is column-major, so the equivalent transform matrix is:
    // | m11 m21  0 m41 |  =>  | m11 m21 | and translate(m41, m42)
    // | m12 m22  0 m42 |      | m12 m22 |
    // |   0   0  1   0 |
    // |   0   0  0   1 |
    let (mut m11, mut m12) = (matrix.m11, matrix.m12);
    let (mut m21, mut m22) = (matrix.m21, matrix.m22);
    // Check if this is a singular matrix.
    if m11 * m22 == m12 * m21 {
        return Err(());
    }

    let mut scale_x = (m11 * m11 + m12 * m12).sqrt();
    m11 /= scale_x;
    m12 /= scale_x;

    let mut shear_xy = m11 * m21 + m12 * m22;
    m21 -= m11 * shear_xy;
    m22 -= m12 * shear_xy;

    let scale_y = (m21 * m21 + m22 * m22).sqrt();
    m21 /= scale_y;
    m22 /= scale_y;
    shear_xy /= scale_y;

    let determinant = m11 * m22 - m12 * m21;
    // Determinant should now be 1 or -1.
    if 0.99 > determinant.abs() || determinant.abs() > 1.01 {
        return Err(());
    }

    if determinant < 0. {
        m11 = -m11;
        m12 = -m12;
        shear_xy = -shear_xy;
        scale_x = -scale_x;
    }

    Ok(MatrixDecomposed3D {
        translate: Translate3D(matrix.m41, matrix.m42, 0.),
        scale: Scale3D(scale_x, scale_y, 1.),
        skew: Skew(shear_xy, 0., 0.),
        perspective: Perspective(0., 0., 0., 1.),
        quaternion: Quaternion::from_direction_and_angle(&DirectionVector::new(0., 0., 1.),
                                                         m12.atan2(m11) as f64)
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

impl Animate for Scale3D {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Scale3D(
            animate_multiplicative_factor(self.0, other.0, procedure)?,
            animate_multiplicative_factor(self.1, other.1, procedure)?,
            animate_multiplicative_factor(self.2, other.2, procedure)?,
        ))
    }
}

impl ComputeSquaredDistance for Skew {
    // We have to use atan() to convert the skew factors into skew angles, so implement
    // ComputeSquaredDistance manually.
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(self.0.atan().compute_squared_distance(&other.0.atan())? +
           self.1.atan().compute_squared_distance(&other.1.atan())? +
           self.2.atan().compute_squared_distance(&other.2.atan())?)
    }
}

impl Animate for Perspective {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(Perspective(
            self.0.animate(&other.0, procedure)?,
            self.1.animate(&other.1, procedure)?,
            self.2.animate(&other.2, procedure)?,
            animate_multiplicative_factor(self.3, other.3, procedure)?,
        ))
    }
}

impl Animate for MatrixDecomposed3D {
    /// https://drafts.csswg.org/css-transforms/#interpolation-of-decomposed-3d-matrix-values
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use std::f64;

        let (this_weight, other_weight) = procedure.weights();

        debug_assert!((this_weight + other_weight - 1.0f64).abs() <= f64::EPSILON ||
                      other_weight == 1.0f64 || other_weight == 0.0f64,
                      "animate should only be used for interpolating or accumulating transforms");

        let mut sum = *self;

        // Add translate, scale, skew and perspective components.
        sum.translate = self.translate.animate(&other.translate, procedure)?;
        sum.scale = self.scale.animate(&other.scale, procedure)?;
        sum.skew = self.skew.animate(&other.skew, procedure)?;
        sum.perspective = self.perspective.animate(&other.perspective, procedure)?;

        // Add quaternions using spherical linear interpolation (Slerp).
        //
        // We take a specialized code path for accumulation (where other_weight is 1)
        if other_weight == 1.0 {
            if this_weight == 0.0 {
                return Ok(*other)
            }

            let clamped_w = self.quaternion.3.min(1.0).max(-1.0);

            // Determine the scale factor.
            let mut theta = clamped_w.acos();
            let mut scale = if theta == 0.0 { 0.0 } else { 1.0 / theta.sin() };
            theta *= this_weight;
            scale *= theta.sin();

            // Scale the self matrix by this_weight.
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
            let w = (other_weight * theta).sin() * 1.0 / (1.0 - product * product).sqrt();

            let mut a = *self;
            let mut b = *other;
            % for i in range(4):
                a.quaternion.${i} *= (other_weight * theta).cos() - product * w;
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
        rotation_matrix.m11 = 1.0 - 2.0 * (y * y + z * z) as f32;
        rotation_matrix.m12 = 2.0 * (x * y + z * w) as f32;
        rotation_matrix.m13 = 2.0 * (x * z - y * w) as f32;
        rotation_matrix.m21 = 2.0 * (x * y - z * w) as f32;
        rotation_matrix.m22 = 1.0 - 2.0 * (x * x + z * z) as f32;
        rotation_matrix.m23 = 2.0 * (y * z + x * w) as f32;
        rotation_matrix.m31 = 2.0 * (x * z + y * w) as f32;
        rotation_matrix.m32 = 2.0 * (y * z - x * w) as f32;
        rotation_matrix.m33 = 1.0 - 2.0 * (x * x + y * y) as f32;

        matrix = multiply(rotation_matrix, matrix);

        // Apply skew
        let mut temp = ComputedMatrix::identity();
        if decomposed.skew.2 != 0.0 {
            temp.m32 = decomposed.skew.2;
            matrix = multiply(temp, matrix);
        }

        if decomposed.skew.1 != 0.0 {
            temp.m32 = 0.0;
            temp.m31 = decomposed.skew.1;
            matrix = multiply(temp, matrix);
        }

        if decomposed.skew.0 != 0.0 {
            temp.m31 = 0.0;
            temp.m21 = decomposed.skew.0;
            matrix = multiply(temp, matrix);
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
impl Animate for TransformList {
    #[inline]
    fn animate(
        &self,
        other: &Self,
        procedure: Procedure,
    ) -> Result<Self, ()> {
        if self.0.is_none() && other.0.is_none() {
            return Ok(TransformList(None));
        }

        if procedure == Procedure::Add {
            let this = self.0.as_ref().map_or(&[][..], |l| l);
            let other = other.0.as_ref().map_or(&[][..], |l| l);
            let result = this.iter().chain(other).cloned().collect::<Vec<_>>();
            return Ok(TransformList(if result.is_empty() {
                None
            } else {
                Some(result)
            }));
        }

        let this = if self.0.is_some() {
            Cow::Borrowed(self)
        } else {
            Cow::Owned(other.to_animated_zero()?)
        };
        let other = if other.0.is_some() {
            Cow::Borrowed(other)
        } else {
            Cow::Owned(self.to_animated_zero()?)
        };

        {
            let this = (*this).0.as_ref().map_or(&[][..], |l| l);
            let other = (*other).0.as_ref().map_or(&[][..], |l| l);
            if this.len() == other.len() {
                let result = this.iter().zip(other).map(|(this, other)| {
                    this.animate(other, procedure)
                }).collect::<Result<Vec<_>, _>>();
                if let Ok(list) = result {
                    return Ok(TransformList(if list.is_empty() {
                        None
                    } else {
                        Some(list)
                    }));
                }
            }
        }

        match procedure {
            Procedure::Add => Err(()),
            Procedure::Interpolate { progress } => {
                Ok(TransformList(Some(vec![TransformOperation::InterpolateMatrix {
                    from_list: this.into_owned(),
                    to_list: other.into_owned(),
                    progress: Percentage(progress as f32),
                }])))
            },
            Procedure::Accumulate { count } => {
                Ok(TransformList(Some(vec![TransformOperation::AccumulateMatrix {
                    from_list: this.into_owned(),
                    to_list: other.into_owned(),
                    count: cmp::min(count, i32::max_value() as u64) as i32,
                }])))
            },
        }
    }
}

// This might not be the most useful definition of distance. It might be better, for example,
// to trace the distance travelled by a point as its transform is interpolated between the two
// lists. That, however, proves to be quite complicated so we take a simple approach for now.
// See https://bugzilla.mozilla.org/show_bug.cgi?id=1318591#c0.
impl ComputeSquaredDistance for TransformOperation {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (
                &TransformOperation::Matrix(ref this),
                &TransformOperation::Matrix(ref other),
            ) => {
                this.compute_squared_distance(other)
            },
            (
                &TransformOperation::Skew(ref fx, ref fy),
                &TransformOperation::Skew(ref tx, ref ty),
            ) => {
                Ok(
                    fx.compute_squared_distance(&tx)? +
                    fy.compute_squared_distance(&ty)?,
                )
            },
            (
                &TransformOperation::Translate(ref fx, ref fy, ref fz),
                &TransformOperation::Translate(ref tx, ref ty, ref tz),
            ) => {
                // We don't want to require doing layout in order to calculate the result, so
                // drop the percentage part. However, dropping percentage makes us impossible to
                // compute the distance for the percentage-percentage case, but Gecko uses the
                // same formula, so it's fine for now.
                // Note: We use pixel value to compute the distance for translate, so we have to
                // convert Au into px.
                let extract_pixel_length = |lop: &LengthOrPercentage| {
                    match *lop {
                        LengthOrPercentage::Length(au) => au.to_f64_px(),
                        LengthOrPercentage::Percentage(_) => 0.,
                        LengthOrPercentage::Calc(calc) => calc.length().to_f64_px(),
                    }
                };

                let fx = extract_pixel_length(&fx);
                let fy = extract_pixel_length(&fy);
                let tx = extract_pixel_length(&tx);
                let ty = extract_pixel_length(&ty);

                Ok(
                    fx.compute_squared_distance(&tx)? +
                    fy.compute_squared_distance(&ty)? +
                    fz.to_f64_px().compute_squared_distance(&tz.to_f64_px())?,
                )
            },
            (
                &TransformOperation::Scale(ref fx, ref fy, ref fz),
                &TransformOperation::Scale(ref tx, ref ty, ref tz),
            ) => {
                Ok(
                    fx.compute_squared_distance(&tx)? +
                    fy.compute_squared_distance(&ty)? +
                    fz.compute_squared_distance(&tz)?,
                )
            },
            (
                &TransformOperation::Rotate(fx, fy, fz, fa),
                &TransformOperation::Rotate(tx, ty, tz, ta),
            ) => {
                let (fx, fy, fz, angle1) =
                    TransformList::get_normalized_vector_and_angle(fx, fy, fz, fa);
                let (tx, ty, tz, angle2) =
                    TransformList::get_normalized_vector_and_angle(tx, ty, tz, ta);
                if (fx, fy, fz) == (tx, ty, tz) {
                    angle1.compute_squared_distance(&angle2)
                } else {
                    let v1 = DirectionVector::new(fx, fy, fz);
                    let v2 = DirectionVector::new(tx, ty, tz);
                    let q1 = Quaternion::from_direction_and_angle(&v1, angle1.radians64());
                    let q2 = Quaternion::from_direction_and_angle(&v2, angle2.radians64());
                    q1.compute_squared_distance(&q2)
                }
            }
            (
                &TransformOperation::Perspective(ref fd),
                &TransformOperation::Perspective(ref td),
            ) => {
                let mut fd_matrix = ComputedMatrix::identity();
                let mut td_matrix = ComputedMatrix::identity();
                if fd.0 > 0 {
                    fd_matrix.m34 = -1. / fd.to_f32_px();
                }

                if td.0 > 0 {
                    td_matrix.m34 = -1. / td.to_f32_px();
                }
                fd_matrix.compute_squared_distance(&td_matrix)
            }
            (
                &TransformOperation::Perspective(ref p),
                &TransformOperation::Matrix(ref m),
            ) | (
                &TransformOperation::Matrix(ref m),
                &TransformOperation::Perspective(ref p),
            ) => {
                let mut p_matrix = ComputedMatrix::identity();
                if p.0 > 0 {
                    p_matrix.m34 = -1. / p.to_f32_px();
                }
                p_matrix.compute_squared_distance(&m)
            }
            _ => Err(()),
        }
    }
}

impl ComputeSquaredDistance for TransformList {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let list1 = self.0.as_ref().map_or(&[][..], |l| l);
        let list2 = other.0.as_ref().map_or(&[][..], |l| l);

        let squared_dist: Result<SquaredDistance, _> = list1.iter().zip_longest(list2).map(|it| {
            match it {
                EitherOrBoth::Both(this, other) => {
                    this.compute_squared_distance(other)
                },
                EitherOrBoth::Left(list) | EitherOrBoth::Right(list) => {
                    list.to_animated_zero()?.compute_squared_distance(list)
                },
            }
        }).sum();

        // Roll back to matrix interpolation if there is any Err(()) in the transform lists, such
        // as mismatched transform functions.
        if let Err(_) = squared_dist {
            let matrix1: ComputedMatrix = self.to_transform_3d_matrix(None).ok_or(())?.into();
            let matrix2: ComputedMatrix = other.to_transform_3d_matrix(None).ok_or(())?.into();
            return matrix1.compute_squared_distance(&matrix2);
        }
        squared_dist
    }
}

impl ToAnimatedZero for TransformList {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match self.0 {
            None => Ok(TransformList(None)),
            Some(ref list) => {
                Ok(TransformList(Some(
                    list.iter().map(|op| op.to_animated_zero()).collect::<Result<Vec<_>, _>>()?
                )))
            },
        }
    }
}

/// Animated SVGPaint
pub type IntermediateSVGPaint = SVGPaint<AnimatedRGBA, ComputedUrl>;

/// Animated SVGPaintKind
pub type IntermediateSVGPaintKind = SVGPaintKind<AnimatedRGBA, ComputedUrl>;

impl ToAnimatedZero for IntermediateSVGPaint {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(IntermediateSVGPaint {
            kind: self.kind.to_animated_zero()?,
            fallback: self.fallback.and_then(|v| v.to_animated_zero().ok()),
        })
    }
}

impl From<NonNegativeLengthOrPercentage> for NumberOrPercentage {
    fn from(lop: NonNegativeLengthOrPercentage) -> NumberOrPercentage {
        lop.0.into()
    }
}

impl From<NonNegativeNumber> for NumberOrPercentage {
    fn from(num: NonNegativeNumber) -> NumberOrPercentage {
        num.0.into()
    }
}

impl From<LengthOrPercentage> for NumberOrPercentage {
    fn from(lop: LengthOrPercentage) -> NumberOrPercentage {
        match lop {
            LengthOrPercentage::Length(len) => NumberOrPercentage::Number(len.to_f32_px()),
            LengthOrPercentage::Percentage(p) => NumberOrPercentage::Percentage(p),
            LengthOrPercentage::Calc(_) => {
                panic!("We dont't expected calc interpolation for SvgLengthOrPercentageOrNumber");
            },
        }
    }
}

impl From<Number> for NumberOrPercentage {
    fn from(num: Number) -> NumberOrPercentage {
        NumberOrPercentage::Number(num)
    }
}

fn convert_to_number_or_percentage<LengthOrPercentageType, NumberType>(
    from: SvgLengthOrPercentageOrNumber<LengthOrPercentageType, NumberType>)
    -> NumberOrPercentage
    where LengthOrPercentageType: Into<NumberOrPercentage>,
          NumberType: Into<NumberOrPercentage>
{
    match from {
        SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) => {
            lop.into()
        }
        SvgLengthOrPercentageOrNumber::Number(num) => {
            num.into()
        }
    }
}

fn convert_from_number_or_percentage<LengthOrPercentageType, NumberType>(
    from: NumberOrPercentage)
    -> SvgLengthOrPercentageOrNumber<LengthOrPercentageType, NumberType>
    where LengthOrPercentageType: From<LengthOrPercentage>,
          NumberType: From<Number>
{
    match from {
        NumberOrPercentage::Number(num) =>
            SvgLengthOrPercentageOrNumber::Number(num.into()),
        NumberOrPercentage::Percentage(p) =>
            SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                (LengthOrPercentage::Percentage(p)).into())
    }
}

impl <L, N> Animate for SvgLengthOrPercentageOrNumber<L, N>
where
    L: Animate + From<LengthOrPercentage> + Into<NumberOrPercentage> + Copy,
    N: Animate + From<Number> + Into<NumberOrPercentage>,
    LengthOrPercentage: From<L>,
    Self: Copy,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.has_calc() || other.has_calc() {
            // TODO: We need to treat calc value.
            // https://bugzilla.mozilla.org/show_bug.cgi?id=1386967
            return Err(());
        }

        let this = convert_to_number_or_percentage(*self);
        let other = convert_to_number_or_percentage(*other);

        match (this, other) {
            (
                NumberOrPercentage::Number(ref this),
                NumberOrPercentage::Number(ref other),
            ) => {
                Ok(convert_from_number_or_percentage(
                    NumberOrPercentage::Number(this.animate(other, procedure)?)
                ))
            },
            (
                NumberOrPercentage::Percentage(ref this),
                NumberOrPercentage::Percentage(ref other),
            ) => {
                Ok(convert_from_number_or_percentage(
                    NumberOrPercentage::Percentage(this.animate(other, procedure)?)
                ))
            },
            _ => Err(()),
        }
    }
}

impl<L> Animate for SVGLength<L>
where
    L: Animate + Clone,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&SVGLength::Length(ref this), &SVGLength::Length(ref other)) => {
                Ok(SVGLength::Length(this.animate(other, procedure)?))
            },
            _ => Err(()),
        }
    }
}

/// https://www.w3.org/TR/SVG11/painting.html#StrokeDasharrayProperty
impl<L> Animate for SVGStrokeDashArray<L>
where
    L: Clone + RepeatableListAnimatable,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if matches!(procedure, Procedure::Add | Procedure::Accumulate { .. }) {
            // Non-additive.
            return Err(());
        }
        match (self, other) {
            (&SVGStrokeDashArray::Values(ref this), &SVGStrokeDashArray::Values(ref other)) => {
                Ok(SVGStrokeDashArray::Values(this.animate(other, procedure)?))
            },
            _ => Err(()),
        }
    }
}

impl<L> ToAnimatedZero for SVGStrokeDashArray<L>
where
    L: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            SVGStrokeDashArray::Values(ref values) => {
                Ok(SVGStrokeDashArray::Values(
                    values.iter().map(ToAnimatedZero::to_animated_zero).collect::<Result<Vec<_>, _>>()?,
                ))
            }
            SVGStrokeDashArray::ContextValue => Ok(SVGStrokeDashArray::ContextValue),
        }
    }
}

impl<O> Animate for SVGOpacity<O>
where
    O: Animate + Clone,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&SVGOpacity::Opacity(ref this), &SVGOpacity::Opacity(ref other)) => {
                Ok(SVGOpacity::Opacity(this.animate(other, procedure)?))
            },
            _ => Err(()),
        }
    }
}

<%
    FILTER_FUNCTIONS = [ 'Blur', 'Brightness', 'Contrast', 'Grayscale',
                         'HueRotate', 'Invert', 'Opacity', 'Saturate',
                         'Sepia' ]
%>

/// https://drafts.fxtf.org/filters/#animation-of-filters
impl Animate for AnimatedFilter {
    fn animate(
        &self,
        other: &Self,
        procedure: Procedure,
    ) -> Result<Self, ()> {
        match (self, other) {
            % for func in ['Blur', 'Grayscale', 'HueRotate', 'Invert', 'Sepia']:
            (&Filter::${func}(ref this), &Filter::${func}(ref other)) => {
                Ok(Filter::${func}(this.animate(other, procedure)?))
            },
            % endfor
            % for func in ['Brightness', 'Contrast', 'Opacity', 'Saturate']:
            (&Filter::${func}(ref this), &Filter::${func}(ref other)) => {
                Ok(Filter::${func}(NonNegative(animate_multiplicative_factor(
                    this.0,
                    other.0,
                    procedure,
                )?)))
            },
            % endfor
            % if product == "gecko":
            (&Filter::DropShadow(ref this), &Filter::DropShadow(ref other)) => {
                Ok(Filter::DropShadow(this.animate(other, procedure)?))
            },
            % endif
            _ => Err(()),
        }
    }
}

/// http://dev.w3.org/csswg/css-transforms/#none-transform-animation
impl ToAnimatedZero for AnimatedFilter {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            % for func in ['Blur', 'Grayscale', 'HueRotate', 'Invert', 'Sepia']:
            Filter::${func}(ref this) => Ok(Filter::${func}(this.to_animated_zero()?)),
            % endfor
            % for func in ['Brightness', 'Contrast', 'Opacity', 'Saturate']:
            Filter::${func}(_) => Ok(Filter::${func}(NonNegative(1.))),
            % endfor
            % if product == "gecko":
            Filter::DropShadow(ref this) => Ok(Filter::DropShadow(this.to_animated_zero()?)),
            % endif
            _ => Err(()),
        }
    }
}

// FIXME(nox): This should be derived.
impl ComputeSquaredDistance for AnimatedFilter {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            % for func in FILTER_FUNCTIONS:
            (&Filter::${func}(ref this), &Filter::${func}(ref other)) => {
                this.compute_squared_distance(other)
            },
            % endfor
            % if product == "gecko":
            (&Filter::DropShadow(ref this), &Filter::DropShadow(ref other)) => {
                this.compute_squared_distance(other)
            },
            % endif
            _ => Err(()),
        }
    }
}

impl Animate for AnimatedFilterList {
    #[inline]
    fn animate(
        &self,
        other: &Self,
        procedure: Procedure,
    ) -> Result<Self, ()> {
        if procedure == Procedure::Add {
            return Ok(AnimatedFilterList(
                self.0.iter().chain(other.0.iter()).cloned().collect(),
            ));
        }
        Ok(AnimatedFilterList(self.0.iter().zip_longest(other.0.iter()).map(|it| {
            match it {
                EitherOrBoth::Both(this, other) => {
                    this.animate(other, procedure)
                },
                EitherOrBoth::Left(this) => {
                    this.animate(&this.to_animated_zero()?, procedure)
                },
                EitherOrBoth::Right(other) => {
                    other.to_animated_zero()?.animate(other, procedure)
                },
            }
        }).collect::<Result<Vec<_>, _>>()?))
    }
}

impl ComputeSquaredDistance for AnimatedFilterList {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        self.0.iter().zip_longest(other.0.iter()).map(|it| {
            match it {
                EitherOrBoth::Both(this, other) => {
                    this.compute_squared_distance(other)
                },
                EitherOrBoth::Left(list) | EitherOrBoth::Right(list) => {
                    list.to_animated_zero()?.compute_squared_distance(list)
                },
            }
        }).sum()
    }
}

/// A comparator to sort PropertyIds such that longhands are sorted before shorthands,
/// shorthands with fewer components are sorted before shorthands with more components,
/// and otherwise shorthands are sorted by IDL name as defined by [Web Animations][property-order].
///
/// Using this allows us to prioritize values specified by longhands (or smaller
/// shorthand subsets) when longhands and shorthands are both specified on the one keyframe.
///
/// Example orderings that result from this:
///
///   margin-left, margin
///
/// and:
///
///   border-top-color, border-color, border-top, border
///
/// [property-order] https://w3c.github.io/web-animations/#calculating-computed-keyframes
#[cfg(feature = "gecko")]
pub fn compare_property_priority(a: &PropertyId, b: &PropertyId) -> cmp::Ordering {
    match (a.as_shorthand(), b.as_shorthand()) {
        // Within shorthands, sort by the number of subproperties, then by IDL name.
        (Ok(a), Ok(b)) => {
            let subprop_count_a = a.longhands().len();
            let subprop_count_b = b.longhands().len();
            subprop_count_a.cmp(&subprop_count_b).then_with(
                || get_idl_name_sort_order(&a).cmp(&get_idl_name_sort_order(&b)))
        },

        // Longhands go before shorthands.
        (Ok(_), Err(_)) => cmp::Ordering::Greater,
        (Err(_), Ok(_)) => cmp::Ordering::Less,

        // Both are longhands or custom properties in which case they don't overlap and should
        // sort equally.
        _ => cmp::Ordering::Equal,
    }
}

#[cfg(feature = "gecko")]
fn get_idl_name_sort_order(shorthand: &ShorthandId) -> u32 {
<%
# Sort by IDL name.
sorted_shorthands = sorted(data.shorthands, key=lambda p: to_idl_name(p.ident))

# Annotate with sorted position
sorted_shorthands = [(p, position) for position, p in enumerate(sorted_shorthands)]
%>
    match *shorthand {
        % for property, position in sorted_shorthands:
            ShorthandId::${property.camel_case} => ${position},
        % endfor
    }
}
