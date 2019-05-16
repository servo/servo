/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

<%!
    from data import Keyword, to_rust_ident, to_camel_case, SYSTEM_FONT_LONGHANDS
    from data import LOGICAL_CORNERS, PHYSICAL_CORNERS, LOGICAL_SIDES, PHYSICAL_SIDES, LOGICAL_SIZES
%>

<%def name="predefined_type(name, type, initial_value, parse_method='parse',
            needs_context=True, vector=False,
            computed_type=None, initial_specified_value=None,
            allow_quirks=False, allow_empty=False, **kwargs)">
    <%def name="predefined_type_inner(name, type, initial_value, parse_method)">
        #[allow(unused_imports)]
        use app_units::Au;
        #[allow(unused_imports)]
        use cssparser::{Color as CSSParserColor, RGBA};
        #[allow(unused_imports)]
        use crate::values::specified::AllowQuirks;
        #[allow(unused_imports)]
        use crate::Zero;
        #[allow(unused_imports)]
        use smallvec::SmallVec;
        pub use crate::values::specified::${type} as SpecifiedValue;
        pub mod computed_value {
            % if computed_type:
            pub use ${computed_type} as T;
            % else:
            pub use crate::values::computed::${type} as T;
            % endif
        }
        % if initial_value:
        #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
        % endif
        % if initial_specified_value:
        #[inline] pub fn get_initial_specified_value() -> SpecifiedValue { ${initial_specified_value} }
        % endif
        #[allow(unused_variables)]
        #[inline]
        pub fn parse<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<SpecifiedValue, ParseError<'i>> {
            % if allow_quirks:
            specified::${type}::${parse_method}_quirky(context, input, AllowQuirks::Yes)
            % elif needs_context:
            specified::${type}::${parse_method}(context, input)
            % else:
            specified::${type}::${parse_method}(input)
            % endif
        }
    </%def>
    % if vector:
        <%call
            expr="vector_longhand(name, predefined_type=type, allow_empty=allow_empty or not initial_value, **kwargs)"
        >
            ${predefined_type_inner(name, type, initial_value, parse_method)}
            % if caller:
            ${caller.body()}
            % endif
        </%call>
    % else:
        <%call expr="longhand(name, predefined_type=type, **kwargs)">
            ${predefined_type_inner(name, type, initial_value, parse_method)}
            % if caller:
            ${caller.body()}
            % endif
        </%call>
    % endif
</%def>

// FIXME (Manishearth): Add computed_value_as_specified argument
// and handle the empty case correctly
<%doc>
    To be used in cases where we have a grammar like "<thing> [ , <thing> ]*".

    Setting allow_empty to False allows for cases where the vector
    is empty. The grammar for these is usually "none | <thing> [ , <thing> ]*".
    We assume that the default/initial value is an empty vector for these.
    `initial_value` need not be defined for these.
</%doc>

// The setup here is roughly:
//
//  * UnderlyingList is the list that is stored in the computed value. This may
//    be a shared ArcSlice if the property is inherited.
//  * UnderlyingOwnedList is the list that is used for animation.
//  * Specified values always use OwnedSlice, since it's more compact.
//  * computed_value::List is just a convenient alias that you can use for the
//    computed value list, since this is in the computed_value module.
//
// If simple_vector_bindings is true, then we don't use the complex iterator
// machinery and set_foo_from, and just compute the value like any other
// longhand.
<%def name="vector_longhand(name, animation_value_type=None,
                            vector_animation_type=None, allow_empty=False,
                            simple_vector_bindings=False,
                            separator='Comma',
                            **kwargs)">
    <%call expr="longhand(name, animation_value_type=animation_value_type, vector=True,
                          simple_vector_bindings=simple_vector_bindings, **kwargs)">
        #[allow(unused_imports)]
        use smallvec::SmallVec;

        pub mod single_value {
            #[allow(unused_imports)]
            use cssparser::{Parser, BasicParseError};
            #[allow(unused_imports)]
            use crate::parser::{Parse, ParserContext};
            #[allow(unused_imports)]
            use crate::properties::ShorthandId;
            #[allow(unused_imports)]
            use selectors::parser::SelectorParseErrorKind;
            #[allow(unused_imports)]
            use style_traits::{ParseError, StyleParseErrorKind};
            #[allow(unused_imports)]
            use crate::values::computed::{Context, ToComputedValue};
            #[allow(unused_imports)]
            use crate::values::{computed, specified};
            #[allow(unused_imports)]
            use crate::values::{Auto, Either, None_};
            ${caller.body()}
        }

        /// The definition of the computed value for ${name}.
        pub mod computed_value {
            #[allow(unused_imports)]
            use crate::values::animated::ToAnimatedValue;
            #[allow(unused_imports)]
            use crate::values::resolved::ToResolvedValue;
            pub use super::single_value::computed_value as single_value;
            pub use self::single_value::T as SingleComputedValue;
            % if not allow_empty or allow_empty == "NotInitial":
            use smallvec::SmallVec;
            % endif
            use crate::values::computed::ComputedVecIter;

            <% is_shared_list = allow_empty and allow_empty != "NotInitial" and data.longhands_by_name[name].style_struct.inherited %>

            // FIXME(emilio): Add an OwnedNonEmptySlice type, and figure out
            // something for transition-name, which is the only remaining user
            // of NotInitial.
            pub type UnderlyingList<T> =
                % if allow_empty and allow_empty != "NotInitial":
                % if data.longhands_by_name[name].style_struct.inherited:
                    crate::ArcSlice<T>;
                % else:
                    crate::OwnedSlice<T>;
                % endif
                % else:
                    SmallVec<[T; 1]>;
                % endif

            pub type UnderlyingOwnedList<T> =
                % if allow_empty and allow_empty != "NotInitial":
                    crate::OwnedSlice<T>;
                % else:
                    SmallVec<[T; 1]>;
                % endif


            /// The generic type defining the animated and resolved values for
            /// this property.
            ///
            /// Making this type generic allows the compiler to figure out the
            /// animated value for us, instead of having to implement it
            /// manually for every type we care about.
            % if separator == "Comma":
            #[css(comma)]
            % endif
            #[derive(
                Clone,
                Debug,
                MallocSizeOf,
                PartialEq,
                ToAnimatedValue,
                ToResolvedValue,
                ToCss,
            )]
            pub struct OwnedList<T>(
                % if not allow_empty:
                #[css(iterable)]
                % else:
                #[css(if_empty = "none", iterable)]
                % endif
                pub UnderlyingOwnedList<T>,
            );

            /// The computed value for this property.
            % if not is_shared_list:
            pub type ComputedList = OwnedList<single_value::T>;
            pub use self::OwnedList as List;
            % else:
            pub use self::ComputedList as List;

            % if separator == "Comma":
            #[css(comma)]
            % endif
            #[derive(
                Clone,
                Debug,
                MallocSizeOf,
                PartialEq,
                ToCss,
            )]
            pub struct ComputedList(
                % if not allow_empty:
                #[css(iterable)]
                % else:
                #[css(if_empty = "none", iterable)]
                % endif
                % if is_shared_list:
                #[ignore_malloc_size_of = "Arc"]
                % endif
                pub UnderlyingList<single_value::T>,
            );

            type ResolvedList = OwnedList<<single_value::T as ToResolvedValue>::ResolvedValue>;
            impl ToResolvedValue for ComputedList {
                type ResolvedValue = ResolvedList;

                fn to_resolved_value(self, context: &crate::values::resolved::Context) -> Self::ResolvedValue {
                    OwnedList(
                        self.0
                            .iter()
                            .cloned()
                            .map(|v| v.to_resolved_value(context))
                            .collect()
                    )
                }

                fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
                    % if not is_shared_list:
                    use std::iter::FromIterator;
                    % endif
                    let iter =
                        resolved.0.into_iter().map(ToResolvedValue::from_resolved_value);
                    ComputedList(UnderlyingList::from_iter(iter))
                }
            }
            % endif

            % if simple_vector_bindings:
            impl From<ComputedList> for UnderlyingList<single_value::T> {
                #[inline]
                fn from(l: ComputedList) -> Self {
                    l.0
                }
            }
            impl From<UnderlyingList<single_value::T>> for ComputedList {
                #[inline]
                fn from(l: UnderlyingList<single_value::T>) -> Self {
                    List(l)
                }
            }
            % endif

            % if vector_animation_type:
            % if not animation_value_type:
                Sorry, this is stupid but needed for now.
            % endif

            use crate::properties::animated_properties::ListAnimation;
            use crate::values::animated::{Animate, ToAnimatedZero, Procedure};
            use crate::values::distance::{SquaredDistance, ComputeSquaredDistance};

            // FIXME(emilio): For some reason rust thinks that this alias is
            // unused, even though it's clearly used below?
            #[allow(unused)]
            type AnimatedList = OwnedList<<single_value::T as ToAnimatedValue>::AnimatedValue>;

            % if is_shared_list:
            impl ToAnimatedValue for ComputedList {
                type AnimatedValue = AnimatedList;

                fn to_animated_value(self) -> Self::AnimatedValue {
                    OwnedList(
                        self.0.iter().map(|v| v.clone().to_animated_value()).collect()
                    )
                }

                fn from_animated_value(animated: Self::AnimatedValue) -> Self {
                    let iter =
                        animated.0.into_iter().map(ToAnimatedValue::from_animated_value);
                    ComputedList(UnderlyingList::from_iter(iter))
                }
            }
            % endif

            impl ToAnimatedZero for AnimatedList {
                fn to_animated_zero(&self) -> Result<Self, ()> { Err(()) }
            }

            impl Animate for AnimatedList {
                fn animate(
                    &self,
                    other: &Self,
                    procedure: Procedure,
                ) -> Result<Self, ()> {
                    Ok(OwnedList(
                        self.0.animate_${vector_animation_type}(&other.0, procedure)?
                    ))
                }
            }
            impl ComputeSquaredDistance for AnimatedList {
                fn compute_squared_distance(
                    &self,
                    other: &Self,
                ) -> Result<SquaredDistance, ()> {
                    self.0.squared_distance_${vector_animation_type}(&other.0)
                }
            }
            % endif

            /// The computed value, effectively a list of single values.
            pub use self::ComputedList as T;

            pub type Iter<'a, 'cx, 'cx_a> = ComputedVecIter<'a, 'cx, 'cx_a, super::single_value::SpecifiedValue>;
        }

        /// The specified value of ${name}.
        % if separator == "Comma":
        #[css(comma)]
        % endif
        #[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
        pub struct SpecifiedValue(
            % if not allow_empty:
            #[css(iterable)]
            % else:
            #[css(if_empty = "none", iterable)]
            % endif
            pub crate::OwnedSlice<single_value::SpecifiedValue>,
        );

        pub fn get_initial_value() -> computed_value::T {
            % if allow_empty and allow_empty != "NotInitial":
                computed_value::List(Default::default())
            % else:
                let mut v = SmallVec::new();
                v.push(single_value::get_initial_value());
                computed_value::List(v)
            % endif
        }

        pub fn parse<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<SpecifiedValue, ParseError<'i>> {
            use style_traits::Separator;

            % if allow_empty:
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Default::default()))
            }
            % endif

            let v = style_traits::${separator}::parse(input, |parser| {
                single_value::parse(context, parser)
            })?;
            Ok(SpecifiedValue(v.into()))
        }

        pub use self::single_value::SpecifiedValue as SingleSpecifiedValue;

        % if not simple_vector_bindings:
        impl SpecifiedValue {
            fn compute_iter<'a, 'cx, 'cx_a>(
                &'a self,
                context: &'cx Context<'cx_a>,
            ) -> computed_value::Iter<'a, 'cx, 'cx_a> {
                computed_value::Iter::new(context, &self.0)
            }
        }
        % endif

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                % if not is_shared_list:
                use std::iter::FromIterator;
                % endif
                computed_value::List(computed_value::UnderlyingList::from_iter(
                    self.0.iter().map(|i| i.to_computed_value(context))
                ))
            }

            #[inline]
            fn from_computed_value(computed: &computed_value::T) -> Self {
                let iter = computed.0.iter().map(ToComputedValue::from_computed_value);
                SpecifiedValue(iter.collect())
            }
        }
    </%call>
</%def>
<%def name="longhand(*args, **kwargs)">
    <%
        property = data.declare_longhand(*args, **kwargs)
        if property is None:
            return ""
    %>
    /// ${property.spec}
    pub mod ${property.ident} {
        #[allow(unused_imports)]
        use cssparser::{Parser, BasicParseError, Token};
        #[allow(unused_imports)]
        use crate::parser::{Parse, ParserContext};
        #[allow(unused_imports)]
        use crate::properties::{UnparsedValue, ShorthandId};
        #[allow(unused_imports)]
        use crate::values::{Auto, Either, None_};
        #[allow(unused_imports)]
        use crate::error_reporting::ParseErrorReporter;
        #[allow(unused_imports)]
        use crate::properties::longhands;
        #[allow(unused_imports)]
        use crate::properties::{LonghandId, LonghandIdSet};
        #[allow(unused_imports)]
        use crate::properties::{CSSWideKeyword, ComputedValues, PropertyDeclaration};
        #[allow(unused_imports)]
        use crate::properties::style_structs;
        #[allow(unused_imports)]
        use selectors::parser::SelectorParseErrorKind;
        #[allow(unused_imports)]
        use servo_arc::Arc;
        #[allow(unused_imports)]
        use style_traits::{ParseError, StyleParseErrorKind};
        #[allow(unused_imports)]
        use crate::values::computed::{Context, ToComputedValue};
        #[allow(unused_imports)]
        use crate::values::{computed, generics, specified};
        #[allow(unused_imports)]
        use crate::Atom;
        ${caller.body()}
        #[allow(unused_variables)]
        pub fn cascade_property(
            declaration: &PropertyDeclaration,
            context: &mut computed::Context,
        ) {
            context.for_non_inherited_property =
                % if property.style_struct.inherited:
                    None;
                % else:
                    Some(LonghandId::${property.camel_case});
                % endif

            let specified_value = match *declaration {
                PropertyDeclaration::${property.camel_case}(ref value) => value,
                PropertyDeclaration::CSSWideKeyword(ref declaration) => {
                    debug_assert_eq!(declaration.id, LonghandId::${property.camel_case});
                    match declaration.keyword {
                        % if not property.style_struct.inherited:
                        CSSWideKeyword::Unset |
                        % endif
                        CSSWideKeyword::Initial => {
                            % if not property.style_struct.inherited:
                                debug_assert!(false, "Should be handled in apply_properties");
                            % else:
                                context.builder.reset_${property.ident}();
                            % endif
                        },
                        % if property.style_struct.inherited:
                        CSSWideKeyword::Unset |
                        % endif
                        CSSWideKeyword::Inherit => {
                            % if property.style_struct.inherited:
                                debug_assert!(false, "Should be handled in apply_properties");
                            % else:
                                context.rule_cache_conditions.borrow_mut().set_uncacheable();
                                context.builder.inherit_${property.ident}();
                            % endif
                        }
                        CSSWideKeyword::Revert => unreachable!("Should never get here"),
                    }
                    return;
                }
                PropertyDeclaration::WithVariables(..) => {
                    panic!("variables should already have been substituted")
                }
                _ => panic!("entered the wrong cascade_property() implementation"),
            };

            % if property.ident in SYSTEM_FONT_LONGHANDS and product == "gecko":
                if let Some(sf) = specified_value.get_system() {
                    longhands::system_font::resolve_system_font(sf, context);
                }
            % endif

            % if not property.style_struct.inherited and property.logical:
                context.rule_cache_conditions.borrow_mut()
                    .set_writing_mode_dependency(context.builder.writing_mode);
            % endif

            % if property.is_vector and not property.simple_vector_bindings:
                // In the case of a vector property we want to pass down an
                // iterator so that this can be computed without allocation.
                //
                // However, computing requires a context, but the style struct
                // being mutated is on the context. We temporarily remove it,
                // mutate it, and then put it back. Vector longhands cannot
                // touch their own style struct whilst computing, else this will
                // panic.
                let mut s =
                    context.builder.take_${data.current_style_struct.name_lower}();
                {
                    let iter = specified_value.compute_iter(context);
                    s.set_${property.ident}(iter);
                }
                context.builder.put_${data.current_style_struct.name_lower}(s);
            % else:
                % if property.boxed:
                let computed = (**specified_value).to_computed_value(context);
                % else:
                let computed = specified_value.to_computed_value(context);
                % endif
                context.builder.set_${property.ident}(computed)
            % endif
        }

        pub fn parse_declared<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<PropertyDeclaration, ParseError<'i>> {
            % if property.allow_quirks:
                parse_quirky(context, input, specified::AllowQuirks::Yes)
            % else:
                parse(context, input)
            % endif
            % if property.boxed:
                .map(Box::new)
            % endif
                .map(PropertyDeclaration::${property.camel_case})
        }
    }
</%def>

<%def name="single_keyword_system(name, values, **kwargs)">
    <%
        keyword_kwargs = {a: kwargs.pop(a, None) for a in [
            'gecko_constant_prefix', 'gecko_enum_prefix',
            'extra_gecko_values', 'extra_servo_values',
            'custom_consts', 'gecko_inexhaustive',
        ]}
        keyword = keyword=Keyword(name, values, **keyword_kwargs)
    %>
    <%call expr="longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
        use crate::properties::longhands::system_font::SystemFont;

        pub mod computed_value {
            #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
            #[derive(
                Clone,
                Copy,
                Debug,
                Eq,
                Hash,
                MallocSizeOf,
                Parse,
                PartialEq,
                SpecifiedValueInfo,
                ToCss,
                ToResolvedValue,
                ToShmem,
            )]
            pub enum T {
            % for value in keyword.values_for(product):
                ${to_camel_case(value)},
            % endfor
            }

            ${gecko_keyword_conversion(keyword, keyword.values_for(product), type="T", cast_to="i32")}
        }

        #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
        pub enum SpecifiedValue {
            Keyword(computed_value::T),
            #[css(skip)]
            System(SystemFont),
        }

        pub fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<SpecifiedValue, ParseError<'i>> {
            Ok(SpecifiedValue::Keyword(computed_value::T::parse(input)?))
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;
            fn to_computed_value(&self, _cx: &Context) -> Self::ComputedValue {
                match *self {
                    SpecifiedValue::Keyword(v) => v,
                    SpecifiedValue::System(_) => {
                        % if product == "gecko":
                            _cx.cached_system_font.as_ref().unwrap().${to_rust_ident(name)}
                        % else:
                            unreachable!()
                        % endif
                    }
                }
            }
            fn from_computed_value(other: &computed_value::T) -> Self {
                SpecifiedValue::Keyword(*other)
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_camel_case(values.split()[0])}
        }
        #[inline]
        pub fn get_initial_specified_value() -> SpecifiedValue {
            SpecifiedValue::Keyword(computed_value::T::${to_camel_case(values.split()[0])})
        }

        impl SpecifiedValue {
            pub fn system_font(f: SystemFont) -> Self {
                SpecifiedValue::System(f)
            }
            pub fn get_system(&self) -> Option<SystemFont> {
                if let SpecifiedValue::System(s) = *self {
                    Some(s)
                } else {
                    None
                }
            }
        }
    </%call>
</%def>

<%def name="gecko_keyword_conversion(keyword, values=None, type='SpecifiedValue', cast_to=None)">
    <%
        if not values:
            values = keyword.values_for(product)
        maybe_cast = "as %s" % cast_to if cast_to else ""
        const_type = cast_to if cast_to else "u32"
    %>
    #[cfg(feature = "gecko")]
    impl ${type} {
        /// Obtain a specified value from a Gecko keyword value
        ///
        /// Intended for use with presentation attributes, not style structs
        pub fn from_gecko_keyword(kw: u32) -> Self {
            use crate::gecko_bindings::structs;
            % for value in values:
                // We can't match on enum values if we're matching on a u32
                const ${to_rust_ident(value).upper()}: ${const_type}
                    = structs::${keyword.gecko_constant(value)} as ${const_type};
            % endfor
            match kw ${maybe_cast} {
                % for value in values:
                    ${to_rust_ident(value).upper()} => ${type}::${to_camel_case(value)},
                % endfor
                _ => panic!("Found unexpected value in style struct for ${keyword.name} property"),
            }
        }
    }
</%def>

<%def name="gecko_bitflags_conversion(bit_map, gecko_bit_prefix, type, kw_type='u8')">
    #[cfg(feature = "gecko")]
    impl ${type} {
        /// Obtain a specified value from a Gecko keyword value
        ///
        /// Intended for use with presentation attributes, not style structs
        pub fn from_gecko_keyword(kw: ${kw_type}) -> Self {
            % for gecko_bit in bit_map.values():
            use crate::gecko_bindings::structs::${gecko_bit_prefix}${gecko_bit};
            % endfor

            let mut bits = ${type}::empty();
            % for servo_bit, gecko_bit in bit_map.iteritems():
                if kw & (${gecko_bit_prefix}${gecko_bit} as ${kw_type}) != 0 {
                    bits |= ${servo_bit};
                }
            % endfor
            bits
        }

        pub fn to_gecko_keyword(self) -> ${kw_type} {
            % for gecko_bit in bit_map.values():
            use crate::gecko_bindings::structs::${gecko_bit_prefix}${gecko_bit};
            % endfor

            let mut bits: ${kw_type} = 0;
            // FIXME: if we ensure that the Servo bitflags storage is the same
            // as Gecko's one, we can just copy it.
            % for servo_bit, gecko_bit in bit_map.iteritems():
                if self.contains(${servo_bit}) {
                    bits |= ${gecko_bit_prefix}${gecko_bit} as ${kw_type};
                }
            % endfor
            bits
        }
    }
</%def>

<%def name="single_keyword(name, values, vector=False,
            extra_specified=None, needs_conversion=False, **kwargs)">
    <%
        keyword_kwargs = {a: kwargs.pop(a, None) for a in [
            'gecko_constant_prefix', 'gecko_enum_prefix',
            'extra_gecko_values', 'extra_servo_values',
            'aliases', 'extra_gecko_aliases', 'custom_consts',
            'gecko_inexhaustive', 'gecko_strip_moz_prefix',
        ]}
    %>

    <%def name="inner_body(keyword, extra_specified=None, needs_conversion=False)">
        <%def name="variants(variants, include_aliases)">
            % for variant in variants:
            % if include_aliases:
            <%
                aliases = []
                for alias, v in keyword.aliases_for(product).iteritems():
                    if variant == v:
                        aliases.append(alias)
            %>
            % if aliases:
            #[parse(aliases = "${','.join(aliases)}")]
            % endif
            % endif
            ${to_camel_case(variant)},
            % endfor
        </%def>
        % if extra_specified:
            #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
            #[derive(
                Clone,
                Copy,
                Debug,
                Eq,
                MallocSizeOf,
                Parse,
                PartialEq,
                SpecifiedValueInfo,
                ToCss,
                ToShmem,
            )]
            pub enum SpecifiedValue {
                ${variants(keyword.values_for(product) + extra_specified.split(), bool(extra_specified))}
            }
        % else:
            pub use self::computed_value::T as SpecifiedValue;
        % endif
        pub mod computed_value {
            #[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
            #[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToCss, ToResolvedValue)]
            % if not extra_specified:
            #[derive(Parse, SpecifiedValueInfo, ToComputedValue, ToShmem)]
            % endif
            pub enum T {
                ${variants(data.longhands_by_name[name].keyword.values_for(product), not extra_specified)}
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_camel_case(values.split()[0])}
        }
        #[inline]
        pub fn get_initial_specified_value() -> SpecifiedValue {
            SpecifiedValue::${to_camel_case(values.split()[0])}
        }
        #[inline]
        pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<SpecifiedValue, ParseError<'i>> {
            SpecifiedValue::parse(input)
        }

        % if needs_conversion:
            <%
                conversion_values = keyword.values_for(product)
                if extra_specified:
                    conversion_values += extra_specified.split()
                conversion_values += keyword.aliases_for(product).keys()
            %>
            ${gecko_keyword_conversion(keyword, values=conversion_values)}
        % endif
    </%def>
    % if vector:
        <%call expr="vector_longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
            ${inner_body(Keyword(name, values, **keyword_kwargs))}
            % if caller:
            ${caller.body()}
            % endif
        </%call>
    % else:
        <%call expr="longhand(name, keyword=Keyword(name, values, **keyword_kwargs), **kwargs)">
            ${inner_body(Keyword(name, values, **keyword_kwargs),
                         extra_specified=extra_specified, needs_conversion=needs_conversion)}
            % if caller:
            ${caller.body()}
            % endif
        </%call>
    % endif
</%def>

<%def name="shorthand(name, sub_properties, derive_serialize=False,
                      derive_value_info=True, **kwargs)">
<%
    shorthand = data.declare_shorthand(name, sub_properties.split(), **kwargs)
    # mako doesn't accept non-string value in parameters with <% %> form, so
    # we have to workaround it this way.
    if not isinstance(derive_value_info, bool):
        derive_value_info = eval(derive_value_info)
%>
    % if shorthand:
    /// ${shorthand.spec}
    pub mod ${shorthand.ident} {
        use cssparser::Parser;
        use crate::parser::ParserContext;
        use crate::properties::{PropertyDeclaration, SourcePropertyDeclaration, MaybeBoxed, longhands};
        #[allow(unused_imports)]
        use selectors::parser::SelectorParseErrorKind;
        #[allow(unused_imports)]
        use std::fmt::{self, Write};
        #[allow(unused_imports)]
        use style_traits::{ParseError, StyleParseErrorKind};
        #[allow(unused_imports)]
        use style_traits::{CssWriter, KeywordsCollectFn, SpecifiedValueInfo, ToCss};

        % if derive_value_info:
        #[derive(SpecifiedValueInfo)]
        % endif
        pub struct Longhands {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}:
                    % if sub_property.boxed:
                        Box<
                    % endif
                    longhands::${sub_property.ident}::SpecifiedValue
                    % if sub_property.boxed:
                        >
                    % endif
                    ,
            % endfor
        }

        /// Represents a serializable set of all of the longhand properties that
        /// correspond to a shorthand.
        % if derive_serialize:
        #[derive(ToCss)]
        % endif
        pub struct LonghandsToSerialize<'a> {
            % for sub_property in shorthand.sub_properties:
                pub ${sub_property.ident}:
                % if sub_property.may_be_disabled_in(shorthand, product):
                    Option<
                % endif
                    &'a longhands::${sub_property.ident}::SpecifiedValue,
                % if sub_property.may_be_disabled_in(shorthand, product):
                    >,
                % endif
            % endfor
        }

        impl<'a> LonghandsToSerialize<'a> {
            /// Tries to get a serializable set of longhands given a set of
            /// property declarations.
            pub fn from_iter<I>(iter: I) -> Result<Self, ()>
            where
                I: Iterator<Item=&'a PropertyDeclaration>,
            {
                // Define all of the expected variables that correspond to the shorthand
                % for sub_property in shorthand.sub_properties:
                    let mut ${sub_property.ident} = None;
                % endfor

                // Attempt to assign the incoming declarations to the expected variables
                for longhand in iter {
                    match *longhand {
                        % for sub_property in shorthand.sub_properties:
                            PropertyDeclaration::${sub_property.camel_case}(ref value) => {
                                ${sub_property.ident} = Some(value)
                            },
                        % endfor
                        _ => {}
                    };
                }

                // If any of the expected variables are missing, return an error
                match (
                    % for sub_property in shorthand.sub_properties:
                        ${sub_property.ident},
                    % endfor
                ) {

                    (
                    % for sub_property in shorthand.sub_properties:
                        % if sub_property.may_be_disabled_in(shorthand, product):
                        ${sub_property.ident},
                        % else:
                        Some(${sub_property.ident}),
                        % endif
                    % endfor
                    ) =>
                    Ok(LonghandsToSerialize {
                        % for sub_property in shorthand.sub_properties:
                            ${sub_property.ident},
                        % endfor
                    }),
                    _ => Err(())
                }
            }
        }

        /// Parse the given shorthand and fill the result into the
        /// `declarations` vector.
        pub fn parse_into<'i, 't>(
            declarations: &mut SourcePropertyDeclaration,
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<(), ParseError<'i>> {
            #[allow(unused_imports)]
            use crate::properties::{NonCustomPropertyId, LonghandId};
            input.parse_entirely(|input| parse_value(context, input)).map(|longhands| {
                % for sub_property in shorthand.sub_properties:
                % if sub_property.may_be_disabled_in(shorthand, product):
                if NonCustomPropertyId::from(LonghandId::${sub_property.camel_case}).allowed_in(context) {
                % endif
                    declarations.push(PropertyDeclaration::${sub_property.camel_case}(
                        longhands.${sub_property.ident}
                    ));
                % if sub_property.may_be_disabled_in(shorthand, product):
                }
                % endif
                % endfor
            })
        }

        ${caller.body()}
    }
    % endif
</%def>

// A shorthand of kind `<property-1> <property-2>?` where both properties have
// the same type.
<%def name="two_properties_shorthand(
    name,
    first_property,
    second_property,
    parser_function,
    needs_context=True,
    **kwargs
)">
<%call expr="self.shorthand(name, sub_properties=' '.join([first_property, second_property]), **kwargs)">
    #[allow(unused_imports)]
    use crate::parser::Parse;
    use crate::values::specified;

    pub fn parse_value<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Longhands, ParseError<'i>> {
        let parse_one = |_c: &ParserContext, input: &mut Parser<'i, 't>| {
            % if needs_context:
            ${parser_function}(_c, input)
            % else:
            ${parser_function}(input)
            % endif
        };

        let first = parse_one(context, input)?;
        let second =
            input.try(|input| parse_one(context, input)).unwrap_or_else(|_| first.clone());
        Ok(expanded! {
            ${to_rust_ident(first_property)}: first,
            ${to_rust_ident(second_property)}: second,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result where W: fmt::Write {
            let first = &self.${to_rust_ident(first_property)};
            let second = &self.${to_rust_ident(second_property)};

            first.to_css(dest)?;
            if first != second {
                dest.write_str(" ")?;
                second.to_css(dest)?;
            }
            Ok(())
        }
    }
</%call>
</%def>

<%def name="four_sides_shorthand(name, sub_property_pattern, parser_function,
                                 needs_context=True, allow_quirks=False, **kwargs)">
    <% sub_properties=' '.join(sub_property_pattern % side for side in PHYSICAL_SIDES) %>
    <%call expr="self.shorthand(name, sub_properties=sub_properties, **kwargs)">
        #[allow(unused_imports)]
        use crate::parser::Parse;
        use crate::values::generics::rect::Rect;
        use crate::values::specified;

        pub fn parse_value<'i, 't>(
            context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<Longhands, ParseError<'i>> {
            let rect = Rect::parse_with(context, input, |_c, i| {
            % if allow_quirks:
                ${parser_function}_quirky(_c, i, specified::AllowQuirks::Yes)
            % elif needs_context:
                ${parser_function}(_c, i)
            % else:
                ${parser_function}(i)
            % endif
            })?;
            Ok(expanded! {
                % for index, side in enumerate(["top", "right", "bottom", "left"]):
                    ${to_rust_ident(sub_property_pattern % side)}: rect.${index},
                % endfor
            })
        }

        impl<'a> ToCss for LonghandsToSerialize<'a> {
            fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
            where
                W: Write,
            {
                let rect = Rect::new(
                    % for side in ["top", "right", "bottom", "left"]:
                    &self.${to_rust_ident(sub_property_pattern % side)},
                    % endfor
                );
                rect.to_css(dest)
            }
        }
    </%call>
</%def>

<%def name="logical_setter_helper(name)">
    <%
        side = None
        size = None
        corner = None
        maybe_side = [s for s in LOGICAL_SIDES if s in name]
        maybe_size = [s for s in LOGICAL_SIZES if s in name]
        maybe_corner = [s for s in LOGICAL_CORNERS if s in name]
        if len(maybe_side) == 1:
            side = maybe_side[0]
        elif len(maybe_size) == 1:
            size = maybe_size[0]
        elif len(maybe_corner) == 1:
            corner = maybe_corner[0]
        def phys_ident(side, phy_side):
            return to_rust_ident(name.replace(side, phy_side).replace("inset-", ""))
    %>
    % if side is not None:
        use crate::logical_geometry::PhysicalSide;
        match wm.${to_rust_ident(side)}_physical_side() {
            % for phy_side in PHYSICAL_SIDES:
                PhysicalSide::${phy_side.title()} => {
                    ${caller.inner(physical_ident=phys_ident(side, phy_side))}
                }
            % endfor
        }
    % elif corner is not None:
        use crate::logical_geometry::PhysicalCorner;
        match wm.${to_rust_ident(corner)}_physical_corner() {
            % for phy_corner in PHYSICAL_CORNERS:
                PhysicalCorner::${to_camel_case(phy_corner)} => {
                    ${caller.inner(physical_ident=phys_ident(corner, phy_corner))}
                }
            % endfor
        }
    % elif size is not None:
        <%
            # (horizontal, vertical)
            physical_size = ("height", "width")
            if size == "inline-size":
                physical_size = ("width", "height")
        %>
        if wm.is_vertical() {
            ${caller.inner(physical_ident=phys_ident(size, physical_size[1]))}
        } else {
            ${caller.inner(physical_ident=phys_ident(size, physical_size[0]))}
        }
    % else:
        <% raise Exception("Don't know what to do with logical property %s" % name) %>
    % endif
</%def>

<%def name="logical_setter(name)">
    /// Set the appropriate physical property for ${name} given a writing mode.
    pub fn set_${to_rust_ident(name)}(&mut self,
                                      v: longhands::${to_rust_ident(name)}::computed_value::T,
                                      wm: WritingMode) {
        <%self:logical_setter_helper name="${name}">
            <%def name="inner(physical_ident)">
                self.set_${physical_ident}(v)
            </%def>
        </%self:logical_setter_helper>
    }

    /// Copy the appropriate physical property from another struct for ${name}
    /// given a writing mode.
    pub fn copy_${to_rust_ident(name)}_from(&mut self,
                                            other: &Self,
                                            wm: WritingMode) {
        <%self:logical_setter_helper name="${name}">
            <%def name="inner(physical_ident)">
                self.copy_${physical_ident}_from(other)
            </%def>
        </%self:logical_setter_helper>
    }

    /// Copy the appropriate physical property from another struct for ${name}
    /// given a writing mode.
    pub fn reset_${to_rust_ident(name)}(&mut self,
                                        other: &Self,
                                        wm: WritingMode) {
        self.copy_${to_rust_ident(name)}_from(other, wm)
    }

    /// Get the computed value for the appropriate physical property for
    /// ${name} given a writing mode.
    pub fn clone_${to_rust_ident(name)}(&self, wm: WritingMode)
        -> longhands::${to_rust_ident(name)}::computed_value::T {
    <%self:logical_setter_helper name="${name}">
        <%def name="inner(physical_ident)">
            self.clone_${physical_ident}()
        </%def>
    </%self:logical_setter_helper>
    }
</%def>
