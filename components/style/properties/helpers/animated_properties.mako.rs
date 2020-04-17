/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<%
    from data import to_idl_name, SYSTEM_FONT_LONGHANDS, to_camel_case
    from itertools import groupby
%>

#[cfg(feature = "gecko")] use crate::gecko_bindings::structs::nsCSSPropertyID;
use itertools::{EitherOrBoth, Itertools};
use crate::properties::{CSSWideKeyword, PropertyDeclaration, NonCustomPropertyIterator};
use crate::properties::longhands;
use crate::properties::longhands::visibility::computed_value::T as Visibility;
use crate::properties::LonghandId;
use servo_arc::Arc;
use smallvec::SmallVec;
use std::ptr;
use std::mem;
use crate::hash::FxHashMap;
use super::ComputedValues;
use crate::values::animated::{Animate, Procedure, ToAnimatedValue, ToAnimatedZero};
use crate::values::animated::effects::AnimatedFilter;
#[cfg(feature = "gecko")] use crate::values::computed::TransitionProperty;
use crate::values::computed::{ClipRect, Context};
use crate::values::computed::ToComputedValue;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::effects::Filter;
use void::{self, Void};

/// Convert nsCSSPropertyID to TransitionProperty
#[cfg(feature = "gecko")]
#[allow(non_upper_case_globals)]
impl From<nsCSSPropertyID> for TransitionProperty {
    fn from(property: nsCSSPropertyID) -> TransitionProperty {
        use properties::ShorthandId;
        match property {
            % for prop in data.longhands:
            ${prop.nscsspropertyid()} => {
                TransitionProperty::Longhand(LonghandId::${prop.camel_case})
            }
            % endfor
            % for prop in data.shorthands_except_all():
            ${prop.nscsspropertyid()} => {
                TransitionProperty::Shorthand(ShorthandId::${prop.camel_case})
            }
            % endfor
            nsCSSPropertyID::eCSSPropertyExtra_all_properties => {
                TransitionProperty::Shorthand(ShorthandId::All)
            }
            _ => {
                panic!("non-convertible nsCSSPropertyID")
            }
        }
    }
}

/// An animated property interpolation between two computed values for that
/// property.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub enum AnimatedProperty {
    % for prop in data.longhands:
        % if prop.animatable and not prop.logical:
            <%
                value_type = "longhands::{}::computed_value::T".format(prop.ident)
                if not prop.is_animatable_with_computed_value:
                    value_type = "<{} as ToAnimatedValue>::AnimatedValue".format(value_type)
            %>
            /// ${prop.name}
            ${prop.camel_case}(${value_type}, ${value_type}),
        % endif
    % endfor
}

impl AnimatedProperty {
    /// Get the id of the property we're animating.
    pub fn id(&self) -> LonghandId {
        match *self {
            % for prop in data.longhands:
            % if prop.animatable and not prop.logical:
            AnimatedProperty::${prop.camel_case}(..) => LonghandId::${prop.camel_case},
            % endif
            % endfor
        }
    }

    /// Get the name of this property.
    pub fn name(&self) -> &'static str {
        self.id().name()
    }

    /// Whether this interpolation does animate, that is, whether the start and
    /// end values are different.
    pub fn does_animate(&self) -> bool {
        match *self {
            % for prop in data.longhands:
                % if prop.animatable and not prop.logical:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => from != to,
                % endif
            % endfor
        }
    }

    /// Whether an animated property has the same end value as another.
    pub fn has_the_same_end_value_as(&self, other: &Self) -> bool {
        match (self, other) {
            % for prop in data.longhands:
                % if prop.animatable and not prop.logical:
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
    #[cfg_attr(feature = "gecko", allow(unused))]
    pub fn update(&self, style: &mut ComputedValues, progress: f64) {
        #[cfg(feature = "servo")]
        {
            match *self {
                % for prop in data.longhands:
                % if prop.animatable and not prop.logical:
                    AnimatedProperty::${prop.camel_case}(ref from, ref to) => {
                        // https://drafts.csswg.org/web-animations/#discrete-animation-type
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
    }

    /// Get an animatable value from a transition-property, an old style, and a
    /// new style.
    pub fn from_longhand(
        property: LonghandId,
        old_style: &ComputedValues,
        new_style: &ComputedValues,
    ) -> Option<AnimatedProperty> {
        // FIXME(emilio): Handle the case where old_style and new_style's
        // writing mode differ.
        let property = property.to_physical(new_style.writing_mode);
        Some(match property {
            % for prop in data.longhands:
            % if prop.animatable and not prop.logical:
                LonghandId::${prop.camel_case} => {
                    let old_computed = old_style.clone_${prop.ident}();
                    let new_computed = new_style.clone_${prop.ident}();
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
            _ => return None,
        })
    }
}

/// A collection of AnimationValue that were composed on an element.
/// This HashMap stores the values that are the last AnimationValue to be
/// composed for each TransitionProperty.
pub type AnimationValueMap = FxHashMap<LonghandId, AnimationValue>;

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
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Debug)]
#[repr(u16)]
pub enum AnimationValue {
    % for prop in data.longhands:
    /// `${prop.name}`
    % if prop.animatable and not prop.logical:
    ${prop.camel_case}(${prop.animated_type()}),
    % else:
    ${prop.camel_case}(Void),
    % endif
    % endfor
}

<%
    animated = []
    unanimated = []
    animated_with_logical = []
    for prop in data.longhands:
        if prop.animatable:
            animated_with_logical.append(prop)
        if prop.animatable and not prop.logical:
            animated.append(prop)
        else:
            unanimated.append(prop)
%>

#[repr(C)]
struct AnimationValueVariantRepr<T> {
    tag: u16,
    value: T
}

impl Clone for AnimationValue {
    #[inline]
    fn clone(&self) -> Self {
        use self::AnimationValue::*;

        <%
            [copy, others] = [list(g) for _, g in groupby(animated, key=lambda x: not x.specified_is_copy())]
        %>

        let self_tag = unsafe { *(self as *const _ as *const u16) };
        if self_tag <= LonghandId::${copy[-1].camel_case} as u16 {
            #[derive(Clone, Copy)]
            #[repr(u16)]
            enum CopyVariants {
                % for prop in copy:
                _${prop.camel_case}(${prop.animated_type()}),
                % endfor
            }

            unsafe {
                let mut out = mem::MaybeUninit::uninit();
                ptr::write(
                    out.as_mut_ptr() as *mut CopyVariants,
                    *(self as *const _ as *const CopyVariants),
                );
                return out.assume_init();
            }
        }

        match *self {
            % for ty, props in groupby(others, key=lambda x: x.animated_type()):
            <% props = list(props) %>
            ${" |\n".join("{}(ref value)".format(prop.camel_case) for prop in props)} => {
                % if len(props) == 1:
                ${props[0].camel_case}(value.clone())
                % else:
                unsafe {
                    let mut out = mem::MaybeUninit::uninit();
                    ptr::write(
                        out.as_mut_ptr() as *mut AnimationValueVariantRepr<${ty}>,
                        AnimationValueVariantRepr {
                            tag: *(self as *const _ as *const u16),
                            value: value.clone(),
                        },
                    );
                    out.assume_init()
                }
                % endif
            }
            % endfor
            _ => unsafe { debug_unreachable!() }
        }
    }
}

impl PartialEq for AnimationValue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        use self::AnimationValue::*;

        unsafe {
            let this_tag = *(self as *const _ as *const u16);
            let other_tag = *(other as *const _ as *const u16);
            if this_tag != other_tag {
                return false;
            }

            match *self {
                % for ty, props in groupby(animated, key=lambda x: x.animated_type()):
                ${" |\n".join("{}(ref this)".format(prop.camel_case) for prop in props)} => {
                    let other_repr =
                        &*(other as *const _ as *const AnimationValueVariantRepr<${ty}>);
                    *this == other_repr.value
                }
                % endfor
                ${" |\n".join("{}(void)".format(prop.camel_case) for prop in unanimated)} => {
                    void::unreachable(void)
                }
            }
        }
    }
}

impl AnimationValue {
    /// Returns the longhand id this animated value corresponds to.
    #[inline]
    pub fn id(&self) -> LonghandId {
        let id = unsafe { *(self as *const _ as *const LonghandId) };
        debug_assert_eq!(id, match *self {
            % for prop in data.longhands:
            % if prop.animatable and not prop.logical:
            AnimationValue::${prop.camel_case}(..) => LonghandId::${prop.camel_case},
            % else:
            AnimationValue::${prop.camel_case}(void) => void::unreachable(void),
            % endif
            % endfor
        });
        id
    }

    /// "Uncompute" this animation value in order to be used inside the CSS
    /// cascade.
    pub fn uncompute(&self) -> PropertyDeclaration {
        use crate::properties::longhands;
        use self::AnimationValue::*;

        use super::PropertyDeclarationVariantRepr;

        match *self {
            <% keyfunc = lambda x: (x.base_type(), x.specified_type(), x.boxed, x.is_animatable_with_computed_value) %>
            % for (ty, specified, boxed, computed), props in groupby(animated, key=keyfunc):
            <% props = list(props) %>
            ${" |\n".join("{}(ref value)".format(prop.camel_case) for prop in props)} => {
                % if not computed:
                let ref value = ToAnimatedValue::from_animated_value(value.clone());
                % endif
                let value = ${ty}::from_computed_value(&value);
                % if boxed:
                let value = Box::new(value);
                % endif
                % if len(props) == 1:
                PropertyDeclaration::${props[0].camel_case}(value)
                % else:
                unsafe {
                    let mut out = mem::MaybeUninit::uninit();
                    ptr::write(
                        out.as_mut_ptr() as *mut PropertyDeclarationVariantRepr<${specified}>,
                        PropertyDeclarationVariantRepr {
                            tag: *(self as *const _ as *const u16),
                            value,
                        },
                    );
                    out.assume_init()
                }
                % endif
            }
            % endfor
            ${" |\n".join("{}(void)".format(prop.camel_case) for prop in unanimated)} => {
                void::unreachable(void)
            }
        }
    }

    /// Construct an AnimationValue from a property declaration.
    pub fn from_declaration(
        decl: &PropertyDeclaration,
        context: &mut Context,
        extra_custom_properties: Option<<&Arc<crate::custom_properties::CustomPropertiesMap>>,
        initial: &ComputedValues
    ) -> Option<Self> {
        use super::PropertyDeclarationVariantRepr;

        <%
            keyfunc = lambda x: (
                x.specified_type(),
                x.animated_type(),
                x.boxed,
                not x.is_animatable_with_computed_value,
                x.style_struct.inherited,
                x.ident in SYSTEM_FONT_LONGHANDS and engine == "gecko",
            )
        %>

        let animatable = match *decl {
            % for (specified_ty, ty, boxed, to_animated, inherit, system), props in groupby(animated_with_logical, key=keyfunc):
            ${" |\n".join("PropertyDeclaration::{}(ref value)".format(prop.camel_case) for prop in props)} => {
                let decl_repr = unsafe {
                    &*(decl as *const _ as *const PropertyDeclarationVariantRepr<${specified_ty}>)
                };
                let longhand_id = unsafe {
                    *(&decl_repr.tag as *const u16 as *const LonghandId)
                };
                % if inherit:
                context.for_non_inherited_property = None;
                % else:
                context.for_non_inherited_property = Some(longhand_id);
                % endif
                % if system:
                if let Some(sf) = value.get_system() {
                    longhands::system_font::resolve_system_font(sf, context)
                }
                % endif
                % if boxed:
                let value = (**value).to_computed_value(context);
                % else:
                let value = value.to_computed_value(context);
                % endif
                % if to_animated:
                let value = value.to_animated_value();
                % endif

                unsafe {
                    let mut out = mem::MaybeUninit::uninit();
                    ptr::write(
                        out.as_mut_ptr() as *mut AnimationValueVariantRepr<${ty}>,
                        AnimationValueVariantRepr {
                            tag: longhand_id.to_physical(context.builder.writing_mode) as u16,
                            value,
                        },
                    );
                    out.assume_init()
                }
            }
            % endfor
            PropertyDeclaration::CSSWideKeyword(ref declaration) => {
                match declaration.id {
                    // We put all the animatable properties first in the hopes
                    // that it might increase match locality.
                    % for prop in data.longhands:
                    % if prop.animatable:
                    LonghandId::${prop.camel_case} => {
                        // FIXME(emilio, bug 1533327): I think
                        // CSSWideKeyword::Revert handling is not fine here, but
                        // what to do instead?
                        //
                        // Seems we'd need the computed value as if it was
                        // revert, somehow. Treating it as `unset` seems fine
                        // for now...
                        let style_struct = match declaration.keyword {
                            % if not prop.style_struct.inherited:
                            CSSWideKeyword::Revert |
                            CSSWideKeyword::Unset |
                            % endif
                            CSSWideKeyword::Initial => {
                                initial.get_${prop.style_struct.name_lower}()
                            },
                            % if prop.style_struct.inherited:
                            CSSWideKeyword::Revert |
                            CSSWideKeyword::Unset |
                            % endif
                            CSSWideKeyword::Inherit => {
                                context.builder
                                       .get_parent_${prop.style_struct.name_lower}()
                            },
                        };
                        let computed = style_struct
                        % if prop.logical:
                            .clone_${prop.ident}(context.builder.writing_mode);
                        % else:
                            .clone_${prop.ident}();
                        % endif

                        % if not prop.is_animatable_with_computed_value:
                        let computed = computed.to_animated_value();
                        % endif

                        % if prop.logical:
                        let wm = context.builder.writing_mode;
                        <%helpers:logical_setter_helper name="${prop.name}">
                        <%def name="inner(physical_ident)">
                            AnimationValue::${to_camel_case(physical_ident)}(computed)
                        </%def>
                        </%helpers:logical_setter_helper>
                        % else:
                            AnimationValue::${prop.camel_case}(computed)
                        % endif
                    },
                    % endif
                    % endfor
                    % for prop in data.longhands:
                    % if not prop.animatable:
                    LonghandId::${prop.camel_case} => return None,
                    % endif
                    % endfor
                }
            },
            PropertyDeclaration::WithVariables(ref declaration) => {
                let substituted = {
                    let custom_properties =
                        extra_custom_properties.or_else(|| context.style().custom_properties());

                    declaration.value.substitute_variables(
                        declaration.id,
                        custom_properties,
                        context.quirks_mode,
                        context.device(),
                    )
                };
                return AnimationValue::from_declaration(
                    &substituted,
                    context,
                    extra_custom_properties,
                    initial,
                )
            },
            _ => return None // non animatable properties will get included because of shorthands. ignore.
        };
        Some(animatable)
    }

    /// Get an AnimationValue for an AnimatableLonghand from a given computed values.
    pub fn from_computed_values(
        property: LonghandId,
        style: &ComputedValues,
    ) -> Option<Self> {
        let property = property.to_physical(style.writing_mode);
        Some(match property {
            % for prop in data.longhands:
            % if prop.animatable and not prop.logical:
            LonghandId::${prop.camel_case} => {
                let computed = style.clone_${prop.ident}();
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
            _ => return None,
        })
    }
}

fn animate_discrete<T: Clone>(this: &T, other: &T, procedure: Procedure) -> Result<T, ()> {
    if let Procedure::Interpolate { progress } = procedure {
        Ok(if progress < 0.5 { this.clone() } else { other.clone() })
    } else {
        Err(())
    }
}

impl Animate for AnimationValue {
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        Ok(unsafe {
            use self::AnimationValue::*;

            let this_tag = *(self as *const _ as *const u16);
            let other_tag = *(other as *const _ as *const u16);
            if this_tag != other_tag {
                panic!("Unexpected AnimationValue::animate call");
            }

            match *self {
                <% keyfunc = lambda x: (x.animated_type(), x.animation_value_type == "discrete") %>
                % for (ty, discrete), props in groupby(animated, key=keyfunc):
                ${" |\n".join("{}(ref this)".format(prop.camel_case) for prop in props)} => {
                    let other_repr =
                        &*(other as *const _ as *const AnimationValueVariantRepr<${ty}>);
                    % if discrete:
                    let value = animate_discrete(this, &other_repr.value, procedure)?;
                    % else:
                    let value = this.animate(&other_repr.value, procedure)?;
                    % endif

                    let mut out = mem::MaybeUninit::uninit();
                    ptr::write(
                        out.as_mut_ptr() as *mut AnimationValueVariantRepr<${ty}>,
                        AnimationValueVariantRepr {
                            tag: this_tag,
                            value,
                        },
                    );
                    out.assume_init()
                }
                % endfor
                ${" |\n".join("{}(void)".format(prop.camel_case) for prop in unanimated)} => {
                    void::unreachable(void)
                }
            }
        })
    }
}

<%
    nondiscrete = []
    for prop in animated:
        if prop.animation_value_type != "discrete":
            nondiscrete.append(prop)
%>

impl ComputeSquaredDistance for AnimationValue {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        unsafe {
            use self::AnimationValue::*;

            let this_tag = *(self as *const _ as *const u16);
            let other_tag = *(other as *const _ as *const u16);
            if this_tag != other_tag {
                panic!("Unexpected AnimationValue::compute_squared_distance call");
            }

            match *self {
                % for ty, props in groupby(nondiscrete, key=lambda x: x.animated_type()):
                ${" |\n".join("{}(ref this)".format(prop.camel_case) for prop in props)} => {
                    let other_repr =
                        &*(other as *const _ as *const AnimationValueVariantRepr<${ty}>);

                    this.compute_squared_distance(&other_repr.value)
                }
                % endfor
                _ => Err(()),
            }
        }
    }
}

impl ToAnimatedZero for AnimationValue {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            % for prop in data.longhands:
            % if prop.animatable and not prop.logical and prop.animation_value_type != "discrete":
            AnimationValue::${prop.camel_case}(ref base) => {
                Ok(AnimationValue::${prop.camel_case}(base.to_animated_zero()?))
            },
            % endif
            % endfor
            _ => Err(()),
        }
    }
}

/// A trait to abstract away the different kind of animations over a list that
/// there may be.
pub trait ListAnimation<T> : Sized {
    /// <https://drafts.csswg.org/css-transitions/#animtype-repeatable-list>
    fn animate_repeatable_list(&self, other: &Self, procedure: Procedure) -> Result<Self, ()>
    where
        T: Animate;

    /// <https://drafts.csswg.org/css-transitions/#animtype-repeatable-list>
    fn squared_distance_repeatable_list(&self, other: &Self) -> Result<SquaredDistance, ()>
    where
        T: ComputeSquaredDistance;

    /// This is the animation used for some of the types like shadows and
    /// filters, where the interpolation happens with the zero value if one of
    /// the sides is not present.
    fn animate_with_zero(&self, other: &Self, procedure: Procedure) -> Result<Self, ()>
    where
        T: Animate + Clone + ToAnimatedZero;

    /// This is the animation used for some of the types like shadows and
    /// filters, where the interpolation happens with the zero value if one of
    /// the sides is not present.
    fn squared_distance_with_zero(&self, other: &Self) -> Result<SquaredDistance, ()>
    where
        T: ToAnimatedZero + ComputeSquaredDistance;
}

macro_rules! animated_list_impl {
    (<$t:ident> for $ty:ty) => {
        impl<$t> ListAnimation<$t> for $ty {
            fn animate_repeatable_list(
                &self,
                other: &Self,
                procedure: Procedure,
            ) -> Result<Self, ()>
            where
                T: Animate,
            {
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

            fn squared_distance_repeatable_list(
                &self,
                other: &Self,
            ) -> Result<SquaredDistance, ()>
            where
                T: ComputeSquaredDistance,
            {
                if self.is_empty() || other.is_empty() {
                    return Err(());
                }
                use num_integer::lcm;
                let len = lcm(self.len(), other.len());
                self.iter().cycle().zip(other.iter().cycle()).take(len).map(|(this, other)| {
                    this.compute_squared_distance(other)
                }).sum()
            }

            fn animate_with_zero(
                &self,
                other: &Self,
                procedure: Procedure,
            ) -> Result<Self, ()>
            where
                T: Animate + Clone + ToAnimatedZero
            {
                if procedure == Procedure::Add {
                    return Ok(
                        self.iter().chain(other.iter()).cloned().collect()
                    );
                }
                self.iter().zip_longest(other.iter()).map(|it| {
                    match it {
                        EitherOrBoth::Both(this, other) => {
                            this.animate(other, procedure)
                        },
                        EitherOrBoth::Left(this) => {
                            this.animate(&this.to_animated_zero()?, procedure)
                        },
                        EitherOrBoth::Right(other) => {
                            other.to_animated_zero()?.animate(other, procedure)
                        }
                    }
                }).collect()
            }

            fn squared_distance_with_zero(
                &self,
                other: &Self,
            ) -> Result<SquaredDistance, ()>
            where
                T: ToAnimatedZero + ComputeSquaredDistance
            {
                self.iter().zip_longest(other.iter()).map(|it| {
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
    }
}

animated_list_impl!(<T> for crate::OwnedSlice<T>);
animated_list_impl!(<T> for SmallVec<[T; 1]>);
animated_list_impl!(<T> for Vec<T>);

/// <https://drafts.csswg.org/web-animations-1/#animating-visibility>
impl Animate for Visibility {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match procedure {
            Procedure::Interpolate { .. } => {
                let (this_weight, other_weight) = procedure.weights();
                match (*self, *other) {
                    (Visibility::Visible, _) => {
                        Ok(if this_weight > 0.0 { *self } else { *other })
                    },
                    (_, Visibility::Visible) => {
                        Ok(if other_weight > 0.0 { *other } else { *self })
                    },
                    _ => Err(()),
                }
            },
            _ => Err(()),
        }
    }
}

impl ComputeSquaredDistance for Visibility {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        Ok(SquaredDistance::from_sqrt(if *self == *other { 0. } else { 1. }))
    }
}

impl ToAnimatedZero for Visibility {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Err(())
    }
}

/// <https://drafts.csswg.org/css-transitions/#animtype-rect>
impl Animate for ClipRect {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use crate::values::computed::LengthOrAuto;
        let animate_component = |this: &LengthOrAuto, other: &LengthOrAuto| {
            let result = this.animate(other, procedure)?;
            if let Procedure::Interpolate { .. } = procedure {
                return Ok(result);
            }
            if result.is_auto() {
                // FIXME(emilio): Why? A couple SMIL tests fail without this,
                // but it seems extremely fishy.
                return Err(());
            }
            Ok(result)
        };

        Ok(ClipRect {
            top: animate_component(&self.top, &other.top)?,
            right: animate_component(&self.right, &other.right)?,
            bottom: animate_component(&self.bottom, &other.bottom)?,
            left: animate_component(&self.left, &other.left)?,
        })
    }
}

<%
    FILTER_FUNCTIONS = [ 'Blur', 'Brightness', 'Contrast', 'Grayscale',
                         'HueRotate', 'Invert', 'Opacity', 'Saturate',
                         'Sepia' ]
%>

/// <https://drafts.fxtf.org/filters/#animation-of-filters>
impl Animate for AnimatedFilter {
    fn animate(
        &self,
        other: &Self,
        procedure: Procedure,
    ) -> Result<Self, ()> {
        use crate::values::animated::animate_multiplicative_factor;
        match (self, other) {
            % for func in ['Blur', 'Grayscale', 'HueRotate', 'Invert', 'Sepia']:
            (&Filter::${func}(ref this), &Filter::${func}(ref other)) => {
                Ok(Filter::${func}(this.animate(other, procedure)?))
            },
            % endfor
            % for func in ['Brightness', 'Contrast', 'Opacity', 'Saturate']:
            (&Filter::${func}(this), &Filter::${func}(other)) => {
                Ok(Filter::${func}(animate_multiplicative_factor(this, other, procedure)?))
            },
            % endfor
            % if engine == "gecko":
            (&Filter::DropShadow(ref this), &Filter::DropShadow(ref other)) => {
                Ok(Filter::DropShadow(this.animate(other, procedure)?))
            },
            % endif
            _ => Err(()),
        }
    }
}

/// <http://dev.w3.org/csswg/css-transforms/#none-transform-animation>
impl ToAnimatedZero for AnimatedFilter {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            % for func in ['Blur', 'Grayscale', 'HueRotate', 'Invert', 'Sepia']:
            Filter::${func}(ref this) => Ok(Filter::${func}(this.to_animated_zero()?)),
            % endfor
            % for func in ['Brightness', 'Contrast', 'Opacity', 'Saturate']:
            Filter::${func}(_) => Ok(Filter::${func}(1.)),
            % endfor
            % if engine == "gecko":
            Filter::DropShadow(ref this) => Ok(Filter::DropShadow(this.to_animated_zero()?)),
            % endif
            _ => Err(()),
        }
    }
}

/// An iterator over all the properties that transition on a given style.
pub struct TransitionPropertyIterator<'a> {
    style: &'a ComputedValues,
    index_range: core::ops::Range<usize>,
    longhand_iterator: Option<NonCustomPropertyIterator<LonghandId>>,
}

impl<'a> TransitionPropertyIterator<'a> {
    /// Create a `TransitionPropertyIterator` for the given style.
    pub fn from_style(style: &'a ComputedValues) -> Self {
        Self {
            style,
            index_range: 0..style.get_box().transition_property_count(),
            longhand_iterator: None,
        }
    }
}

/// A single iteration of the TransitionPropertyIterator.
pub struct TransitionPropertyIteration {
    /// The id of the longhand for this property.
    pub longhand_id: LonghandId,

    /// The index of this property in the list of transition properties for this
    /// iterator's style.
    pub index: usize,
}

impl<'a> Iterator for TransitionPropertyIterator<'a> {
    type Item = TransitionPropertyIteration;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::values::computed::TransitionProperty;
        loop {
            if let Some(ref mut longhand_iterator) = self.longhand_iterator {
                if let Some(longhand_id) = longhand_iterator.next() {
                    return Some(TransitionPropertyIteration {
                        longhand_id,
                        index: self.index_range.start,
                    });
                }
                self.longhand_iterator = None;
            }

            let index = self.index_range.next()?;
            match self.style.get_box().transition_property_at(index) {
                TransitionProperty::Longhand(longhand_id) => {
                    return Some(TransitionPropertyIteration {
                        longhand_id,
                        index,
                    })
                }
                // In the other cases, we set up our state so that we are ready to
                // compute the next value of the iterator and then loop (equivalent
                // to calling self.next()).
                TransitionProperty::Shorthand(ref shorthand_id) =>
                    self.longhand_iterator = Some(shorthand_id.longhands()),
                TransitionProperty::Custom(..) | TransitionProperty::Unsupported(..) => {}
            }
        }
    }
}
