/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

// Please note that valid Rust syntax may be mangled by the Mako parser.
// For example, Vec<&Foo> will be mangled as Vec&Foo>. To work around these issues, the code
// can be escaped. In the above example, Vec<<&Foo> or Vec< &Foo> achieves the desired result of Vec<&Foo>.

<%namespace name="helpers" file="/helpers.mako.rs" />

#[cfg(feature = "servo")]
use app_units::Au;
use custom_properties::CustomPropertiesBuilder;
use servo_arc::{Arc, UniqueArc};
use smallbitvec::SmallBitVec;
use std::borrow::Cow;
use std::{fmt, mem, ops};
use std::cell::RefCell;

#[cfg(feature = "servo")] use cssparser::RGBA;
use cssparser::{CowRcStr, Parser, TokenSerializationType, serialize_identifier};
use cssparser::ParserInput;
#[cfg(feature = "servo")] use euclid::SideOffsets2D;
use computed_values;
use context::QuirksMode;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")] use gecko_bindings::bindings;
#[cfg(feature = "gecko")] use gecko_bindings::structs::{self, nsCSSPropertyID};
#[cfg(feature = "servo")] use logical_geometry::LogicalMargin;
use logical_geometry::WritingMode;
use media_queries::Device;
use parser::ParserContext;
#[cfg(feature = "gecko")] use properties::longhands::system_font::SystemFont;
use rule_cache::{RuleCache, RuleCacheConditions};
use selector_parser::PseudoElement;
use selectors::parser::SelectorParseError;
#[cfg(feature = "servo")] use servo_config::prefs::PREFS;
use shared_lock::StylesheetGuards;
use style_traits::{PARSING_MODE_DEFAULT, ToCss, ParseError};
use style_traits::{PropertyDeclarationParseError, StyleParseError, ValueParseError};
use stylesheets::{CssRuleType, Origin, UrlExtraData};
#[cfg(feature = "servo")] use values::Either;
use values::generics::text::LineHeight;
use values::computed;
use values::computed::NonNegativeLength;
use rule_tree::{CascadeLevel, StrongRuleNode};
use self::computed_value_flags::*;
use style_adjuster::StyleAdjuster;

pub use self::declaration_block::*;

#[cfg(feature = "gecko")]
#[macro_export]
macro_rules! property_name {
    ($s: tt) => { atom!($s) }
}

#[cfg(feature = "gecko")]
macro_rules! impl_bitflags_conversions {
    ($name: ident) => {
        impl From<u8> for $name {
            fn from(bits: u8) -> $name {
                $name::from_bits(bits).expect("bits contain valid flag")
            }
        }

        impl From<$name> for u8 {
            fn from(v: $name) -> u8 {
                v.bits()
            }
        }
    };
}

<%!
    from data import Method, Keyword, to_rust_ident, to_camel_case, SYSTEM_FONT_LONGHANDS
    import os.path
%>

#[path="${repr(os.path.join(os.path.dirname(__file__), 'computed_value_flags.rs'))[1:-1]}"]
pub mod computed_value_flags;
#[path="${repr(os.path.join(os.path.dirname(__file__), 'declaration_block.rs'))[1:-1]}"]
pub mod declaration_block;

/// Conversion with fewer impls than From/Into
pub trait MaybeBoxed<Out> {
    /// Convert
    fn maybe_boxed(self) -> Out;
}

impl<T> MaybeBoxed<T> for T {
    #[inline]
    fn maybe_boxed(self) -> T { self }
}

impl<T> MaybeBoxed<Box<T>> for T {
    #[inline]
    fn maybe_boxed(self) -> Box<T> { Box::new(self) }
}

macro_rules! expanded {
    ( $( $name: ident: $value: expr ),+ ) => {
        expanded!( $( $name: $value, )+ )
    };
    ( $( $name: ident: $value: expr, )+ ) => {
        Longhands {
            $(
                $name: MaybeBoxed::maybe_boxed($value),
            )+
        }
    }
}

/// A module with all the code for longhand properties.
#[allow(missing_docs)]
pub mod longhands {
    <%include file="/longhand/background.mako.rs" />
    <%include file="/longhand/border.mako.rs" />
    <%include file="/longhand/box.mako.rs" />
    <%include file="/longhand/color.mako.rs" />
    <%include file="/longhand/column.mako.rs" />
    <%include file="/longhand/counters.mako.rs" />
    <%include file="/longhand/effects.mako.rs" />
    <%include file="/longhand/font.mako.rs" />
    <%include file="/longhand/inherited_box.mako.rs" />
    <%include file="/longhand/inherited_table.mako.rs" />
    <%include file="/longhand/inherited_text.mako.rs" />
    <%include file="/longhand/list.mako.rs" />
    <%include file="/longhand/margin.mako.rs" />
    <%include file="/longhand/outline.mako.rs" />
    <%include file="/longhand/padding.mako.rs" />
    <%include file="/longhand/pointing.mako.rs" />
    <%include file="/longhand/position.mako.rs" />
    <%include file="/longhand/table.mako.rs" />
    <%include file="/longhand/text.mako.rs" />
    <%include file="/longhand/ui.mako.rs" />
    <%include file="/longhand/inherited_svg.mako.rs" />
    <%include file="/longhand/svg.mako.rs" />
    <%include file="/longhand/xul.mako.rs" />
}

macro_rules! unwrap_or_initial {
    ($prop: ident) => (unwrap_or_initial!($prop, $prop));
    ($prop: ident, $expr: expr) =>
        ($expr.unwrap_or_else(|| $prop::get_initial_specified_value()));
}

/// A module with code for all the shorthand css properties, and a few
/// serialization helpers.
#[allow(missing_docs)]
pub mod shorthands {
    use cssparser::Parser;
    use parser::{Parse, ParserContext};
    use style_traits::{ParseError, StyleParseError};
    use values::specified;

    <%include file="/shorthand/serialize.mako.rs" />
    <%include file="/shorthand/background.mako.rs" />
    <%include file="/shorthand/border.mako.rs" />
    <%include file="/shorthand/box.mako.rs" />
    <%include file="/shorthand/column.mako.rs" />
    <%include file="/shorthand/font.mako.rs" />
    <%include file="/shorthand/inherited_text.mako.rs" />
    <%include file="/shorthand/list.mako.rs" />
    <%include file="/shorthand/margin.mako.rs" />
    <%include file="/shorthand/mask.mako.rs" />
    <%include file="/shorthand/outline.mako.rs" />
    <%include file="/shorthand/padding.mako.rs" />
    <%include file="/shorthand/position.mako.rs" />
    <%include file="/shorthand/inherited_svg.mako.rs" />
    <%include file="/shorthand/text.mako.rs" />

    // We don't defined the 'all' shorthand using the regular helpers:shorthand
    // mechanism, since it causes some very large types to be generated.
    <% data.declare_shorthand("all",
                              [p.name for p in data.longhands
                                if p.name not in ['direction', 'unicode-bidi']
                                      and not p.internal],
                              spec="https://drafts.csswg.org/css-cascade-3/#all-shorthand") %>
}

/// A module with all the code related to animated properties.
///
/// This needs to be "included" by mako at least after all longhand modules,
/// given they populate the global data.
pub mod animated_properties {
    <%include file="/helpers/animated_properties.mako.rs" />
}

/// A longhand or shorthand porperty
#[derive(Clone, Copy, Debug)]
pub struct NonCustomPropertyId(usize);

impl From<LonghandId> for NonCustomPropertyId {
    fn from(id: LonghandId) -> Self {
        NonCustomPropertyId(id as usize)
    }
}

impl From<ShorthandId> for NonCustomPropertyId {
    fn from(id: ShorthandId) -> Self {
        NonCustomPropertyId((id as usize) + ${len(data.longhands)})
    }
}

impl From<AliasId> for NonCustomPropertyId {
    fn from(id: AliasId) -> Self {
        NonCustomPropertyId(id as usize + ${len(data.longhands) + len(data.shorthands)})
    }
}

/// A set of all properties
#[derive(Clone, PartialEq)]
pub struct NonCustomPropertyIdSet {
    storage: [u32; (${len(data.longhands) + len(data.shorthands) + len(data.all_aliases())} - 1 + 32) / 32]
}

impl NonCustomPropertyIdSet {
    /// Return whether the given property is in the set
    #[inline]
    pub fn contains(&self, id: NonCustomPropertyId) -> bool {
        let bit = id.0;
        (self.storage[bit / 32] & (1 << (bit % 32))) != 0
    }
}

<%def name="static_non_custom_property_id_set(name, is_member)">
static ${name}: NonCustomPropertyIdSet = NonCustomPropertyIdSet {
    <%
        storage = [0] * ((len(data.longhands) + len(data.shorthands) + len(data.all_aliases()) - 1 + 32) / 32)
        for i, property in enumerate(data.longhands + data.shorthands):
            if is_member(property):
                storage[i / 32] |= 1 << (i % 32)
    %>
    storage: [${", ".join("0x%x" % word for word in storage)}]
};
</%def>

<%def name="static_longhand_id_set(name, is_member)">
static ${name}: LonghandIdSet = LonghandIdSet {
    <%
        storage = [0] * ((len(data.longhands) - 1 + 32) / 32)
        for i, property in enumerate(data.longhands):
            if is_member(property):
                storage[i / 32] |= 1 << (i % 32)
    %>
    storage: [${", ".join("0x%x" % word for word in storage)}]
};
</%def>

/// A set of longhand properties
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct LonghandIdSet {
    storage: [u32; (${len(data.longhands)} - 1 + 32) / 32]
}

/// An iterator over a set of longhand ids.
pub struct LonghandIdSetIterator<'a> {
    longhands: &'a LonghandIdSet,
    cur: usize,
}

impl<'a> Iterator for LonghandIdSetIterator<'a> {
    type Item = LonghandId;

    fn next(&mut self) -> Option<Self::Item> {
        use std::mem;

        loop {
            if self.cur >= ${len(data.longhands)} {
                return None;
            }

            let id: LonghandId = unsafe { mem::transmute(self.cur as ${"u16" if product == "gecko" else "u8"}) };
            self.cur += 1;

            if self.longhands.contains(id) {
                return Some(id);
            }
        }
    }
}

impl LonghandIdSet {
    /// Iterate over the current longhand id set.
    pub fn iter(&self) -> LonghandIdSetIterator {
        LonghandIdSetIterator { longhands: self, cur: 0, }
    }

    /// Returns whether this set contains at least every longhand that `other`
    /// also contains.
    pub fn contains_all(&self, other: &Self) -> bool {
        for (self_cell, other_cell) in self.storage.iter().zip(other.storage.iter()) {
            if (*self_cell & *other_cell) != *other_cell {
                return false;
            }
        }
        true
    }

    /// Create an empty set
    #[inline]
    pub fn new() -> LonghandIdSet {
        LonghandIdSet { storage: [0; (${len(data.longhands)} - 1 + 32) / 32] }
    }

    /// Return whether the given property is in the set
    #[inline]
    pub fn contains(&self, id: LonghandId) -> bool {
        let bit = id as usize;
        (self.storage[bit / 32] & (1 << (bit % 32))) != 0
    }

    /// Add the given property to the set
    #[inline]
    pub fn insert(&mut self, id: LonghandId) {
        let bit = id as usize;
        self.storage[bit / 32] |= 1 << (bit % 32);
    }

    /// Remove the given property from the set
    #[inline]
    pub fn remove(&mut self, id: LonghandId) {
        let bit = id as usize;
        self.storage[bit / 32] &= !(1 << (bit % 32));
    }

    /// Clear all bits
    #[inline]
    pub fn clear(&mut self) {
        for cell in &mut self.storage {
            *cell = 0
        }
    }

    /// Returns whether the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage.iter().all(|c| *c == 0)
    }
}

/// A specialized set of PropertyDeclarationId
pub struct PropertyDeclarationIdSet {
    longhands: LonghandIdSet,

    // FIXME: Use a HashSet instead? This Vec is usually small, so linear scan might be ok.
    custom: Vec<::custom_properties::Name>,
}

impl PropertyDeclarationIdSet {
    /// Empty set
    pub fn new() -> Self {
        PropertyDeclarationIdSet {
            longhands: LonghandIdSet::new(),
            custom: Vec::new(),
        }
    }

    /// Returns all the longhands that this set contains.
    pub fn longhands(&self) -> &LonghandIdSet {
        &self.longhands
    }

    /// Returns whether the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.longhands.is_empty() && self.custom.is_empty()
    }

    /// Clears the set.
    #[inline]
    pub fn clear(&mut self) {
        self.longhands.clear();
        self.custom.clear();
    }

    /// Returns whether the given ID is in the set
    pub fn contains(&mut self, id: PropertyDeclarationId) -> bool {
        match id {
            PropertyDeclarationId::Longhand(id) => self.longhands.contains(id),
            PropertyDeclarationId::Custom(name) => self.custom.contains(name),
        }
    }

    /// Insert the given ID in the set
    pub fn insert(&mut self, id: PropertyDeclarationId) {
        match id {
            PropertyDeclarationId::Longhand(id) => self.longhands.insert(id),
            PropertyDeclarationId::Custom(name) => {
                if !self.custom.contains(name) {
                    self.custom.push(name.clone())
                }
            }
        }
    }
}

/// An enum to represent a CSS Wide keyword.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToCss)]
pub enum CSSWideKeyword {
    /// The `initial` keyword.
    Initial,
    /// The `inherit` keyword.
    Inherit,
    /// The `unset` keyword.
    Unset,
}

impl CSSWideKeyword {
    fn to_str(&self) -> &'static str {
        match *self {
            CSSWideKeyword::Initial => "initial",
            CSSWideKeyword::Inherit => "inherit",
            CSSWideKeyword::Unset => "unset",
        }
    }

    /// Takes the result of cssparser::Parser::expect_ident() and converts it
    /// to a CSSWideKeyword.
    pub fn from_ident<'i>(ident: &str) -> Option<Self> {
        match_ignore_ascii_case! { ident,
            // If modifying this set of keyword, also update values::CustomIdent::from_ident
            "initial" => Some(CSSWideKeyword::Initial),
            "inherit" => Some(CSSWideKeyword::Inherit),
            "unset" => Some(CSSWideKeyword::Unset),
            _ => None
        }
    }
}

impl CSSWideKeyword {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        let ident = input.expect_ident().map_err(|_| ())?.clone();
        input.expect_exhausted().map_err(|_| ())?;
        CSSWideKeyword::from_ident(&ident).ok_or(())
    }
}

bitflags! {
    /// A set of flags for properties.
    pub flags PropertyFlags: u8 {
        /// This property requires a stacking context.
        const CREATES_STACKING_CONTEXT = 1 << 0,
        /// This property has values that can establish a containing block for
        /// fixed positioned and absolutely positioned elements.
        const FIXPOS_CB = 1 << 1,
        /// This property has values that can establish a containing block for
        /// absolutely positioned elements.
        const ABSPOS_CB = 1 << 2,
        /// This shorthand property is an alias of another property.
        const SHORTHAND_ALIAS_PROPERTY = 1 << 3,
        /// This longhand property applies to ::first-letter.
        const APPLIES_TO_FIRST_LETTER = 1 << 4,
        /// This longhand property applies to ::first-line.
        const APPLIES_TO_FIRST_LINE = 1 << 5,
        /// This longhand property applies to ::placeholder.
        const APPLIES_TO_PLACEHOLDER = 1 << 6,
    }
}

/// An identifier for a given longhand property.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LonghandId {
    % for i, property in enumerate(data.longhands):
        /// ${property.name}
        ${property.camel_case} = ${i},
    % endfor
}

impl fmt::Debug for LonghandId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl LonghandId {
    /// Get the name of this longhand property.
    pub fn name(&self) -> &'static str {
        match *self {
            % for property in data.longhands:
                LonghandId::${property.camel_case} => "${property.name}",
            % endfor
        }
    }

    fn inherited(&self) -> bool {
        ${static_longhand_id_set("INHERITED", lambda p: p.style_struct.inherited)}
        INHERITED.contains(*self)
    }

    fn shorthands(&self) -> &'static [ShorthandId] {
        // first generate longhand to shorthands lookup map
        //
        // NOTE(emilio): This currently doesn't exclude the "all" shorthand. It
        // could potentially do so, which would speed up serialization
        // algorithms and what not, I guess.
        <%
            longhand_to_shorthand_map = {}
            num_sub_properties = {}
            for shorthand in data.shorthands:
                num_sub_properties[shorthand.camel_case] = len(shorthand.sub_properties)
                for sub_property in shorthand.sub_properties:
                    if sub_property.ident not in longhand_to_shorthand_map:
                        longhand_to_shorthand_map[sub_property.ident] = []

                    longhand_to_shorthand_map[sub_property.ident].append(shorthand.camel_case)

            def preferred_order(x, y):
                # Since we want properties in order from most subproperties to least,
                # reverse the arguments to cmp from the expected order.
                result = cmp(num_sub_properties.get(y, 0), num_sub_properties.get(x, 0))
                if result:
                    return result
                # Fall back to lexicographic comparison.
                return cmp(x, y)

            # Sort the lists of shorthand properties according to preferred order:
            # https://drafts.csswg.org/cssom/#concept-shorthands-preferred-order
            for shorthand_list in longhand_to_shorthand_map.itervalues():
                shorthand_list.sort(cmp=preferred_order)
        %>

        // based on lookup results for each longhand, create result arrays
        % for property in data.longhands:
            static ${property.ident.upper()}: &'static [ShorthandId] = &[
                % for shorthand in longhand_to_shorthand_map.get(property.ident, []):
                    ShorthandId::${shorthand},
                % endfor
            ];
        % endfor

        match *self {
            % for property in data.longhands:
                LonghandId::${property.camel_case} => ${property.ident.upper()},
            % endfor
        }
    }

    fn parse_value<'i, 't>(&self, context: &ParserContext, input: &mut Parser<'i, 't>)
                           -> Result<PropertyDeclaration, ParseError<'i>> {
        match *self {
            % for property in data.longhands:
                LonghandId::${property.camel_case} => {
                    % if not property.derived_from:
                        longhands::${property.ident}::parse_declared(context, input)
                    % else:
                        Err(PropertyDeclarationParseError::UnknownProperty("${property.ident}".into()).into())
                    % endif
                }
            % endfor
        }
    }

    /// Returns whether this property is animatable.
    pub fn is_animatable(self) -> bool {
        match self {
            % for property in data.longhands:
            LonghandId::${property.camel_case} => {
                ${str(property.animatable).lower()}
            }
            % endfor
        }
    }

    /// Returns whether this property is animatable in a discrete way.
    pub fn is_discrete_animatable(self) -> bool {
        match self {
            % for property in data.longhands:
            LonghandId::${property.camel_case} => {
                ${str(property.animation_value_type == "discrete").lower()}
            }
            % endfor
        }
    }

    /// Converts from a LonghandId to an adequate nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    pub fn to_nscsspropertyid(self) -> nsCSSPropertyID {
        match self {
            % for property in data.longhands:
            LonghandId::${property.camel_case} => {
                ${helpers.to_nscsspropertyid(property.ident)}
            }
            % endfor
        }
    }

    #[cfg(feature = "gecko")]
    #[allow(non_upper_case_globals)]
    /// Returns a longhand id from Gecko's nsCSSPropertyID.
    pub fn from_nscsspropertyid(id: nsCSSPropertyID) -> Result<Self, ()> {
        match PropertyId::from_nscsspropertyid(id) {
            Ok(PropertyId::Longhand(id)) => Ok(id),
            _ => Err(()),
        }
    }

    /// If this is a logical property, return the corresponding physical one in the given writing mode.
    /// Otherwise, return unchanged.
    pub fn to_physical(&self, wm: WritingMode) -> Self {
        match *self {
            % for property in data.longhands:
                % if property.logical:
                    LonghandId::${property.camel_case} => {
                        <%helpers:logical_setter_helper name="${property.name}">
                            <%def name="inner(physical_ident)">
                                LonghandId::${to_camel_case(physical_ident)}
                            </%def>
                        </%helpers:logical_setter_helper>
                    }
                % endif
            % endfor
            _ => *self
        }
    }

    /// Returns PropertyFlags for given longhand property.
    pub fn flags(&self) -> PropertyFlags {
        match *self {
            % for property in data.longhands:
                LonghandId::${property.camel_case} =>
                    % for flag in property.flags:
                        ${flag} |
                    % endfor
                    PropertyFlags::empty(),
            % endfor
        }
    }

    /// Only a few properties are allowed to depend on the visited state of
    /// links. When cascading visited styles, we can save time by only
    /// processing these properties.
    fn is_visited_dependent(&self) -> bool {
        matches!(*self,
            % if product == "gecko":
            LonghandId::ColumnRuleColor |
            LonghandId::TextEmphasisColor |
            LonghandId::WebkitTextFillColor |
            LonghandId::WebkitTextStrokeColor |
            LonghandId::TextDecorationColor |
            LonghandId::Fill |
            LonghandId::Stroke |
            LonghandId::CaretColor |
            % endif
            LonghandId::Color |
            LonghandId::BackgroundColor |
            LonghandId::BorderTopColor |
            LonghandId::BorderRightColor |
            LonghandId::BorderBottomColor |
            LonghandId::BorderLeftColor |
            LonghandId::OutlineColor
        )
    }

    /// Returns true if the property is one that is ignored when document
    /// colors are disabled.
    fn is_ignored_when_document_colors_disabled(&self) -> bool {
        matches!(*self,
            ${" | ".join([("LonghandId::" + p.camel_case)
                          for p in data.longhands if p.ignored_when_colors_disabled])}
        )
    }

    /// The computed value of some properties depends on the (sometimes
    /// computed) value of *other* properties.
    ///
    /// So we classify properties into "early" and "other", such that the only
    /// dependencies can be from "other" to "early".
    ///
    /// Unfortunately, it’s not easy to check that this classification is
    /// correct.
    fn is_early_property(&self) -> bool {
        matches!(*self,
            % if product == 'gecko':
            LonghandId::TextOrientation |
            LonghandId::AnimationName |
            LonghandId::TransitionProperty |
            LonghandId::XLang |
            LonghandId::XTextZoom |
            LonghandId::MozScriptLevel |
            LonghandId::MozMinFontSizeRatio |
            % endif
            LonghandId::FontSize |
            LonghandId::FontFamily |
            LonghandId::Color |
            LonghandId::TextDecorationLine |
            LonghandId::WritingMode |
            LonghandId::Direction
        )
    }

    /// Whether computed values of this property lossily convert any complex
    /// colors into RGBA colors.
    ///
    /// In Gecko, there are some properties still that compute currentcolor
    /// down to an RGBA color at computed value time, instead of as
    /// `StyleComplexColor`s. For these properties, we must return `false`,
    /// so that we correctly avoid caching style data in the rule tree.
    pub fn stores_complex_colors_lossily(&self) -> bool {
        % if product == "gecko":
        matches!(*self,
            % for property in data.longhands:
            % if property.predefined_type == "RGBAColor":
            LonghandId::${property.camel_case} |
            % endif
            % endfor
            LonghandId::BackgroundImage |
            LonghandId::BorderImageSource |
            LonghandId::BoxShadow |
            LonghandId::MaskImage |
            LonghandId::MozBorderBottomColors |
            LonghandId::MozBorderLeftColors |
            LonghandId::MozBorderRightColors |
            LonghandId::MozBorderTopColors |
            LonghandId::TextShadow
        )
        % else:
        false
        % endif
    }
}

/// An identifier for a given shorthand property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ToCss)]
pub enum ShorthandId {
    % for property in data.shorthands:
        /// ${property.name}
        ${property.camel_case},
    % endfor
}

impl ShorthandId {
    /// Get the name for this shorthand property.
    pub fn name(&self) -> &'static str {
        match *self {
            % for property in data.shorthands:
                ShorthandId::${property.camel_case} => "${property.name}",
            % endfor
        }
    }

    /// Converts from a ShorthandId to an adequate nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    pub fn to_nscsspropertyid(self) -> nsCSSPropertyID {
        match self {
            % for property in data.shorthands:
            ShorthandId::${property.camel_case} => {
                ${helpers.to_nscsspropertyid(property.ident)}
            }
            % endfor
        }
    }

    /// Get the longhand ids that form this shorthand.
    pub fn longhands(&self) -> &'static [LonghandId] {
        % for property in data.shorthands:
            static ${property.ident.upper()}: &'static [LonghandId] = &[
                % for sub in property.sub_properties:
                    LonghandId::${sub.camel_case},
                % endfor
            ];
        % endfor
        match *self {
            % for property in data.shorthands:
                ShorthandId::${property.camel_case} => ${property.ident.upper()},
            % endfor
        }
    }

    /// Try to serialize the given declarations as this shorthand.
    ///
    /// Returns an error if writing to the stream fails, or if the declarations
    /// do not map to a shorthand.
    pub fn longhands_to_css<'a, W, I>(&self, declarations: I, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
              I: Iterator<Item=&'a PropertyDeclaration>,
    {
        match *self {
            ShorthandId::All => {
                // No need to try to serialize the declarations as the 'all'
                // shorthand, since it only accepts CSS-wide keywords (and
                // variable references), which will be handled in
                // get_shorthand_appendable_value.
                Err(fmt::Error)
            }
            % for property in data.shorthands_except_all():
                ShorthandId::${property.camel_case} => {
                    match shorthands::${property.ident}::LonghandsToSerialize::from_iter(declarations) {
                        Ok(longhands) => longhands.to_css(dest),
                        Err(_) => Err(fmt::Error)
                    }
                },
            % endfor
        }
    }

    /// Finds and returns an appendable value for the given declarations.
    ///
    /// Returns the optional appendable value.
    pub fn get_shorthand_appendable_value<'a, I>(self,
                                                 declarations: I)
                                                 -> Option<AppendableValue<'a, I::IntoIter>>
        where I: IntoIterator<Item=&'a PropertyDeclaration>,
              I::IntoIter: Clone,
    {
        let declarations = declarations.into_iter();

        // Only cloning iterators (a few pointers each) not declarations.
        let mut declarations2 = declarations.clone();
        let mut declarations3 = declarations.clone();

        let first_declaration = match declarations2.next() {
            Some(declaration) => declaration,
            None => return None
        };

        // https://drafts.csswg.org/css-variables/#variables-in-shorthands
        if let Some(css) = first_declaration.with_variables_from_shorthand(self) {
            if declarations2.all(|d| d.with_variables_from_shorthand(self) == Some(css)) {
               return Some(AppendableValue::Css {
                   css: css,
                   with_variables: true,
               });
            }
            return None;
        }

        // Check whether they are all the same CSS-wide keyword.
        if let Some(keyword) = first_declaration.get_css_wide_keyword() {
            if declarations2.all(|d| d.get_css_wide_keyword() == Some(keyword)) {
                return Some(AppendableValue::Css {
                    css: keyword.to_str(),
                    with_variables: false,
                });
            }
            return None;
        }

        // Check whether all declarations can be serialized as part of shorthand.
        if declarations3.all(|d| d.may_serialize_as_part_of_shorthand()) {
            return Some(AppendableValue::DeclarationsForShorthand(self, declarations));
        }

        None
    }

    /// Returns PropertyFlags for given shorthand property.
    pub fn flags(&self) -> PropertyFlags {
        match *self {
            % for property in data.shorthands:
                ShorthandId::${property.camel_case} =>
                    % for flag in property.flags:
                        ${flag} |
                    % endfor
                    PropertyFlags::empty(),
            % endfor
        }
    }

    fn parse_into<'i, 't>(&self, declarations: &mut SourcePropertyDeclaration,
                          context: &ParserContext, input: &mut Parser<'i, 't>)
                          -> Result<(), ParseError<'i>> {
        match *self {
            % for shorthand in data.shorthands_except_all():
                ShorthandId::${shorthand.camel_case} => {
                    shorthands::${shorthand.ident}::parse_into(declarations, context, input)
                }
            % endfor
            // 'all' accepts no value other than CSS-wide keywords
            ShorthandId::All => Err(StyleParseError::UnspecifiedError.into())
        }
    }
}

/// Servo's representation of a declared value for a given `T`, which is the
/// declared value for that property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredValue<'a, T: 'a> {
    /// A known specified value from the stylesheet.
    Value(&'a T),
    /// An unparsed value that contains `var()` functions.
    WithVariables(&'a Arc<UnparsedValue>),
    /// An CSS-wide keyword.
    CSSWideKeyword(CSSWideKeyword),
}

/// A variant of DeclaredValue that owns its data. This separation exists so
/// that PropertyDeclaration can avoid embedding a DeclaredValue (and its
/// extra discriminant word) and synthesize dependent DeclaredValues for
/// PropertyDeclaration instances as needed.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredValueOwned<T> {
    /// A known specified value from the stylesheet.
    Value(T),
    /// An unparsed value that contains `var()` functions.
    WithVariables(
        #[cfg_attr(feature = "gecko", ignore_malloc_size_of = "XXX: how to handle this?")]
        Arc<UnparsedValue>
    ),
    /// An CSS-wide keyword.
    CSSWideKeyword(CSSWideKeyword),
}

impl<T> DeclaredValueOwned<T> {
    /// Creates a dependent DeclaredValue from this DeclaredValueOwned.
    fn borrow(&self) -> DeclaredValue<T> {
        match *self {
            DeclaredValueOwned::Value(ref v) => DeclaredValue::Value(v),
            DeclaredValueOwned::WithVariables(ref v) => DeclaredValue::WithVariables(v),
            DeclaredValueOwned::CSSWideKeyword(v) => DeclaredValue::CSSWideKeyword(v),
        }
    }
}

/// An unparsed property value that contains `var()` functions.
#[derive(Debug, Eq, PartialEq)]
pub struct UnparsedValue {
    /// The css serialization for this value.
    css: String,
    /// The first token type for this serialization.
    first_token_type: TokenSerializationType,
    /// The url data for resolving url values.
    url_data: UrlExtraData,
    /// The shorthand this came from.
    from_shorthand: Option<ShorthandId>,
}

impl UnparsedValue {
    fn substitute_variables(
        &self,
        longhand_id: LonghandId,
        custom_properties: Option<<&Arc<::custom_properties::CustomPropertiesMap>>,
        quirks_mode: QuirksMode,
    ) -> PropertyDeclaration {
        ::custom_properties::substitute(&self.css, self.first_token_type, custom_properties)
        .ok()
        .and_then(|css| {
            // As of this writing, only the base URL is used for property values:
            let context = ParserContext::new(
                Origin::Author,
                &self.url_data,
                None,
                PARSING_MODE_DEFAULT,
                quirks_mode,
            );
            let mut input = ParserInput::new(&css);
            Parser::new(&mut input).parse_entirely(|input| {
                match self.from_shorthand {
                    None => longhand_id.parse_value(&context, input),
                    Some(ShorthandId::All) => {
                        // No need to parse the 'all' shorthand as anything other than a CSS-wide
                        // keyword, after variable substitution.
                        Err(SelectorParseError::UnexpectedIdent("all".into()).into())
                    }
                    % for shorthand in data.shorthands_except_all():
                        Some(ShorthandId::${shorthand.camel_case}) => {
                            shorthands::${shorthand.ident}::parse_value(&context, input)
                            .map(|longhands| {
                                match longhand_id {
                                    % for property in shorthand.sub_properties:
                                        LonghandId::${property.camel_case} => {
                                            PropertyDeclaration::${property.camel_case}(
                                                longhands.${property.ident}
                                            )
                                        }
                                    % endfor
                                    _ => unreachable!()
                                }
                            })
                        }
                    % endfor
                }
            })
            .ok()
        })
        .unwrap_or_else(|| {
            // Invalid at computed-value time.
            let keyword = if longhand_id.inherited() {
                CSSWideKeyword::Inherit
            } else {
                CSSWideKeyword::Initial
            };
            PropertyDeclaration::CSSWideKeyword(longhand_id, keyword)
        })
    }
}

impl<'a, T: ToCss> ToCss for DeclaredValue<'a, T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            DeclaredValue::Value(ref inner) => inner.to_css(dest),
            DeclaredValue::WithVariables(ref with_variables) => {
                // https://drafts.csswg.org/css-variables/#variables-in-shorthands
                if with_variables.from_shorthand.is_none() {
                    dest.write_str(&*with_variables.css)?
                }
                Ok(())
            },
            DeclaredValue::CSSWideKeyword(ref keyword) => keyword.to_css(dest),
        }
    }
}

/// An identifier for a given property declaration, which can be either a
/// longhand or a custom property.
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum PropertyDeclarationId<'a> {
    /// A longhand.
    Longhand(LonghandId),
    /// A custom property declaration.
    Custom(&'a ::custom_properties::Name),
}

impl<'a> ToCss for PropertyDeclarationId<'a> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            PropertyDeclarationId::Longhand(id) => dest.write_str(id.name()),
            PropertyDeclarationId::Custom(_) => {
                serialize_identifier(&self.name(), dest)
            }
        }
    }
}

impl<'a> PropertyDeclarationId<'a> {
    /// Whether a given declaration id is either the same as `other`, or a
    /// longhand of it.
    pub fn is_or_is_longhand_of(&self, other: &PropertyId) -> bool {
        match *self {
            PropertyDeclarationId::Longhand(id) => {
                match *other {
                    PropertyId::Longhand(other_id) => id == other_id,
                    PropertyId::Shorthand(shorthand) => self.is_longhand_of(shorthand),
                    PropertyId::Custom(_) => false,
                }
            }
            PropertyDeclarationId::Custom(name) => {
                matches!(*other, PropertyId::Custom(ref other_name) if name == other_name)
            }
        }
    }

    /// Whether a given declaration id is a longhand belonging to this
    /// shorthand.
    pub fn is_longhand_of(&self, shorthand: ShorthandId) -> bool {
        match *self {
            PropertyDeclarationId::Longhand(ref id) => id.shorthands().contains(&shorthand),
            _ => false,
        }
    }

    /// Returns the name of the property without CSS escaping.
    pub fn name(&self) -> Cow<'static, str> {
        match *self {
            PropertyDeclarationId::Longhand(id) => id.name().into(),
            PropertyDeclarationId::Custom(name) => {
                use std::fmt::Write;
                let mut s = String::new();
                write!(&mut s, "--{}", name).unwrap();
                s.into()
            }
        }
    }
}

/// Servo's representation of a CSS property, that is, either a longhand, a
/// shorthand, or a custom property.
#[derive(Clone, Eq, PartialEq)]
pub enum PropertyId {
    /// A longhand property.
    Longhand(LonghandId),
    /// A shorthand property.
    Shorthand(ShorthandId),
    /// A custom property.
    Custom(::custom_properties::Name),
}

impl fmt::Debug for PropertyId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.to_css(formatter)
    }
}

impl ToCss for PropertyId {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            PropertyId::Longhand(id) => dest.write_str(id.name()),
            PropertyId::Shorthand(id) => dest.write_str(id.name()),
            PropertyId::Custom(_) => {
                serialize_identifier(&self.name(), dest)
            }
        }
    }
}

impl PropertyId {
    /// Returns a given property from the string `s`.
    ///
    /// Returns Err(()) for unknown non-custom properties
    /// If caller wants to provide a different context, it can be provided with
    /// Some(context), if None is given, default setting for PropertyParserContext
    /// will be used. It is `Origin::Author` for stylesheet_origin and
    /// `CssRuleType::Style` for rule_type.
    pub fn parse(property_name: &str, context: Option< &PropertyParserContext>) -> Result<Self, ()> {
        // FIXME(https://github.com/rust-lang/rust/issues/33156): remove this enum and use PropertyId
        // when stable Rust allows destructors in statics.
        // ShorthandAlias is not used in servo build. That's why we need to allow dead_code.
        #[allow(dead_code)]
        pub enum StaticId {
            Longhand(LonghandId),
            Shorthand(ShorthandId),
            LonghandAlias(LonghandId, AliasId),
            ShorthandAlias(ShorthandId, AliasId),
        }
        ascii_case_insensitive_phf_map! {
            static_id -> StaticId = {
                % for (kind, properties) in [("Longhand", data.longhands), ("Shorthand", data.shorthands)]:
                    % for property in properties:
                        "${property.name}" => StaticId::${kind}(${kind}Id::${property.camel_case}),
                        % for name in property.alias:
                            "${name}" => {
                                StaticId::${kind}Alias(${kind}Id::${property.camel_case},
                                                       AliasId::${to_camel_case(name)})
                            },
                        % endfor
                    % endfor
                % endfor
            }
        }

        let default;
        let context = match context {
            Some(context) => context,
            None => {
                default = PropertyParserContext {
                    stylesheet_origin: Origin::Author,
                    rule_type: CssRuleType::Style,
                };
                &default
            }
        };
        let rule_type = context.rule_type;
        debug_assert!(matches!(rule_type, CssRuleType::Keyframe |
                                          CssRuleType::Page |
                                          CssRuleType::Style),
                      "Declarations are only expected inside a keyframe, page, or style rule.");

        let (id, alias) = match static_id(property_name) {
            Some(&StaticId::Longhand(id)) => {
                (PropertyId::Longhand(id), None)
            },
            Some(&StaticId::Shorthand(id)) => {
                (PropertyId::Shorthand(id), None)
            },
            Some(&StaticId::LonghandAlias(id, alias)) => {
                (PropertyId::Longhand(id), Some(alias))
            },
            Some(&StaticId::ShorthandAlias(id, alias)) => {
                (PropertyId::Shorthand(id), Some(alias))
            },
            None => return ::custom_properties::parse_name(property_name)
                .map(|name| PropertyId::Custom(::custom_properties::Name::from(name))),
        };
        id.check_allowed_in(alias, context)?;
        Ok(id)
    }

    /// Returns a property id from Gecko's nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    #[allow(non_upper_case_globals)]
    pub fn from_nscsspropertyid(id: nsCSSPropertyID) -> Result<Self, ()> {
        use gecko_bindings::structs::*;
        match id {
            % for property in data.longhands:
                ${helpers.to_nscsspropertyid(property.ident)} => {
                    Ok(PropertyId::Longhand(LonghandId::${property.camel_case}))
                }
                % for alias in property.alias:
                    ${helpers.alias_to_nscsspropertyid(alias)} => {
                        Ok(PropertyId::Longhand(LonghandId::${property.camel_case}))
                    }
                % endfor
            % endfor
            % for property in data.shorthands:
                ${helpers.to_nscsspropertyid(property.ident)} => {
                    Ok(PropertyId::Shorthand(ShorthandId::${property.camel_case}))
                }
                % for alias in property.alias:
                    ${helpers.alias_to_nscsspropertyid(alias)} => {
                        Ok(PropertyId::Shorthand(ShorthandId::${property.camel_case}))
                    }
                % endfor
            % endfor
            _ => Err(())
        }
    }

    /// Returns an nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    #[allow(non_upper_case_globals)]
    pub fn to_nscsspropertyid(&self) -> Result<nsCSSPropertyID, ()> {
        match *self {
            PropertyId::Longhand(id) => Ok(id.to_nscsspropertyid()),
            PropertyId::Shorthand(id) => Ok(id.to_nscsspropertyid()),
            _ => Err(())
        }
    }

    /// Given this property id, get it either as a shorthand or as a
    /// `PropertyDeclarationId`.
    pub fn as_shorthand(&self) -> Result<ShorthandId, PropertyDeclarationId> {
        match *self {
            PropertyId::Shorthand(id) => Ok(id),
            PropertyId::Longhand(id) => Err(PropertyDeclarationId::Longhand(id)),
            PropertyId::Custom(ref name) => Err(PropertyDeclarationId::Custom(name)),
        }
    }

    /// Returns the name of the property without CSS escaping.
    pub fn name(&self) -> Cow<'static, str> {
        match *self {
            PropertyId::Shorthand(id) => id.name().into(),
            PropertyId::Longhand(id) => id.name().into(),
            PropertyId::Custom(ref name) => {
                use std::fmt::Write;
                let mut s = String::new();
                write!(&mut s, "--{}", name).unwrap();
                s.into()
            }
        }
    }

    fn check_allowed_in(
        &self,
        alias: Option<AliasId>,
        context: &PropertyParserContext,
    ) -> Result<(), ()> {
        let id: NonCustomPropertyId;
        if let Some(alias_id) = alias {
            id = alias_id.into();
        } else {
            match *self {
                // Custom properties are allowed everywhere
                PropertyId::Custom(_) => return Ok(()),

                PropertyId::Shorthand(shorthand_id) => id = shorthand_id.into(),
                PropertyId::Longhand(longhand_id) => id = longhand_id.into(),
            }
        }

        <% id_set = static_non_custom_property_id_set %>

        ${id_set("DISALLOWED_IN_KEYFRAME_BLOCK", lambda p: not p.allowed_in_keyframe_block)}
        ${id_set("DISALLOWED_IN_PAGE_RULE", lambda p: not p.allowed_in_page_rule)}
        match context.rule_type {
            CssRuleType::Keyframe if DISALLOWED_IN_KEYFRAME_BLOCK.contains(id) => {
                return Err(());
            }
            CssRuleType::Page if DISALLOWED_IN_PAGE_RULE.contains(id) => {
                return Err(())
            }
            _ => {}
        }

        // For properties that are experimental but not internal, the pref will
        // control its availability in all sheets.   For properties that are
        // both experimental and internal, the pref only controls its
        // availability in non-UA sheets (and in UA sheets it is always available).
        ${id_set("INTERNAL", lambda p: p.internal)}

        % if product == "servo":
            ${id_set("EXPERIMENTAL", lambda p: p.experimental)}
        % endif
        % if product == "gecko":
            use gecko_bindings::structs::root::mozilla;
            static EXPERIMENTAL: NonCustomPropertyIdSet = NonCustomPropertyIdSet {
                <%
                    grouped = []
                    properties = data.longhands + data.shorthands + data.all_aliases()
                    while properties:
                        grouped.append(properties[:32])
                        properties = properties[32:]
                %>
                storage: [
                    % for group in grouped:
                        (0
                        % for i, property in enumerate(group):
                            | ((mozilla::SERVO_PREF_ENABLED_${property.gecko_pref_ident} as u32) << ${i})
                        % endfor
                        ),
                    % endfor
                ]
            };
        % endif

        let passes_pref_check = || {
            % if product == "servo":
                static PREF_NAME: [Option< &str>; ${len(data.longhands) + len(data.shorthands)}] = [
                    % for property in data.longhands + data.shorthands:
                        % if property.experimental:
                            Some("${property.experimental}"),
                        % else:
                            None,
                        % endif
                    % endfor
                ];
                match PREF_NAME[id.0] {
                    None => true,
                    Some(pref) => PREFS.get(pref).as_boolean().unwrap_or(false)
                }
            % endif
            % if product == "gecko":
                let id = match alias {
                    Some(alias_id) => alias_id.to_nscsspropertyid().unwrap(),
                    None => self.to_nscsspropertyid().unwrap(),
                };
                unsafe { structs::nsCSSProps_gPropertyEnabled[id as usize] }
            % endif
        };

        if INTERNAL.contains(id) {
            if context.stylesheet_origin != Origin::UserAgent {
                if EXPERIMENTAL.contains(id) {
                    if !passes_pref_check() {
                        return Err(())
                    }
                } else {
                    return Err(())
                }
            }
        } else {
            if EXPERIMENTAL.contains(id) && !passes_pref_check() {
                return Err(());
            }
        }

        Ok(())
    }
}

/// Parsing Context for PropertyId.
pub struct PropertyParserContext {
    /// The Origin of the stylesheet, whether it's a user,
    /// author or user-agent stylesheet.
    pub stylesheet_origin: Origin,
    /// The current rule type, if any.
    pub rule_type: CssRuleType,
}

impl PropertyParserContext {
    /// Creates a PropertyParserContext with given stylesheet origin and rule type.
    pub fn new(context: &ParserContext) -> Self {
        Self {
            stylesheet_origin: context.stylesheet_origin,
            rule_type: context.rule_type(),
        }
    }
}

/// Servo's representation for a property declaration.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, PartialEq)]
pub enum PropertyDeclaration {
    % for property in data.longhands:
        /// ${property.name}
        % if property.boxed:
            ${property.camel_case}(Box<longhands::${property.ident}::SpecifiedValue>),
        % else:
            ${property.camel_case}(longhands::${property.ident}::SpecifiedValue),
        % endif
    % endfor
    /// A css-wide keyword.
    CSSWideKeyword(LonghandId, CSSWideKeyword),
    /// An unparsed value that contains `var()` functions.
    WithVariables(
        LonghandId,
        #[cfg_attr(feature = "gecko", ignore_malloc_size_of = "XXX: how to handle this?")]
        Arc<UnparsedValue>
    ),
    /// A custom property declaration, with the property name and the declared
    /// value.
    #[cfg_attr(feature = "gecko", ignore_malloc_size_of = "XXX: how to handle this?")]
    Custom(
        ::custom_properties::Name,
        #[cfg_attr(feature = "gecko", ignore_malloc_size_of = "XXX: how to handle this?")]
        DeclaredValueOwned<Arc<::custom_properties::SpecifiedValue>>
    ),
}

impl fmt::Debug for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.id().to_css(f)?;
        f.write_str(": ")?;
        self.to_css(f)
    }
}

impl ToCss for PropertyDeclaration {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            % for property in data.longhands:
                % if not property.derived_from:
                    PropertyDeclaration::${property.camel_case}(ref value) =>
                        value.to_css(dest),
                % endif
            % endfor
            PropertyDeclaration::CSSWideKeyword(_, keyword) => keyword.to_css(dest),
            PropertyDeclaration::WithVariables(_, ref with_variables) => {
                // https://drafts.csswg.org/css-variables/#variables-in-shorthands
                match with_variables.from_shorthand {
                    // Normally, we shouldn't be printing variables here if they came from
                    // shorthands. But we should allow properties that came from shorthand
                    // aliases. That also matches with the Gecko behavior.
                    Some(shorthand) if shorthand.flags().contains(SHORTHAND_ALIAS_PROPERTY) =>
                        dest.write_str(&*with_variables.css)?,
                    None => dest.write_str(&*with_variables.css)?,
                    _ => {},
                }
                Ok(())
            },
            PropertyDeclaration::Custom(_, ref value) => value.borrow().to_css(dest),
            % if any(property.derived_from for property in data.longhands):
                _ => Err(fmt::Error),
            % endif
        }
    }
}

impl PropertyDeclaration {
    /// Given a property declaration, return the property declaration id.
    pub fn id(&self) -> PropertyDeclarationId {
        match *self {
            PropertyDeclaration::Custom(ref name, _) => {
                return PropertyDeclarationId::Custom(name)
            }
            PropertyDeclaration::CSSWideKeyword(id, _) |
            PropertyDeclaration::WithVariables(id, _) => {
                return PropertyDeclarationId::Longhand(id)
            }
            _ => {}
        }
        let longhand_id = match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(..) => {
                    LonghandId::${property.camel_case}
                }
            % endfor
            PropertyDeclaration::CSSWideKeyword(..) |
            PropertyDeclaration::WithVariables(..) |
            PropertyDeclaration::Custom(..) => {
                debug_assert!(false, "unreachable");
                // This value is never used, but having an expression of the same "shape"
                // as for other variants helps the optimizer compile this `match` expression
                // to a lookup table.
                LonghandId::BackgroundColor
            }
        };
        PropertyDeclarationId::Longhand(longhand_id)
    }

    fn with_variables_from_shorthand(&self, shorthand: ShorthandId) -> Option< &str> {
        match *self {
            PropertyDeclaration::WithVariables(_, ref with_variables) => {
                if let Some(s) = with_variables.from_shorthand {
                    if s == shorthand {
                        Some(&*with_variables.css)
                    } else { None }
                } else {
                    // Normally, longhand property that doesn't come from a shorthand
                    // should return None here. But we return Some to longhands if they
                    // came from a shorthand alias. Because for example, we should be able to
                    // get -moz-transform's value from transform.
                    if shorthand.flags().contains(SHORTHAND_ALIAS_PROPERTY) {
                        return Some(&*with_variables.css);
                    }
                    None
                }
            },
            _ => None,
        }
    }

    /// Returns a CSS-wide keyword if the declaration's value is one.
    pub fn get_css_wide_keyword(&self) -> Option<CSSWideKeyword> {
        match *self {
            PropertyDeclaration::CSSWideKeyword(_, keyword) => Some(keyword),
            _ => None,
        }
    }

    /// Returns whether or not the property is set by a system font
    #[cfg(feature = "gecko")]
    pub fn get_system(&self) -> Option<SystemFont> {
        match *self {
            % for prop in SYSTEM_FONT_LONGHANDS:
                PropertyDeclaration::${to_camel_case(prop)}(ref prop) => {
                    prop.get_system()
                }
            % endfor
            _ => None,
        }
    }

    /// Is it the default value of line-height?
    pub fn is_default_line_height(&self) -> bool {
        match *self {
            PropertyDeclaration::LineHeight(LineHeight::Normal) => true,
            _ => false
        }
    }

    #[cfg(feature = "servo")]
    /// Dummy method to avoid cfg()s
    pub fn get_system(&self) -> Option<()> {
        None
    }

    /// Returns whether the declaration may be serialized as part of a shorthand.
    ///
    /// This method returns false if this declaration contains variable or has a
    /// CSS-wide keyword value, since these values cannot be serialized as part
    /// of a shorthand.
    ///
    /// Caller should check `with_variables_from_shorthand()` and whether all
    /// needed declarations has the same CSS-wide keyword first.
    ///
    /// Note that, serialization of a shorthand may still fail because of other
    /// property-specific requirement even when this method returns true for all
    /// the longhand declarations.
    pub fn may_serialize_as_part_of_shorthand(&self) -> bool {
        match *self {
            PropertyDeclaration::CSSWideKeyword(..) |
            PropertyDeclaration::WithVariables(..) => false,
            PropertyDeclaration::Custom(..) =>
                unreachable!("Serializing a custom property as part of shorthand?"),
            _ => true,
        }
    }

    /// Return whether the value is stored as it was in the CSS source,
    /// preserving whitespace (as opposed to being parsed into a more abstract
    /// data structure).
    ///
    /// This is the case of custom properties and values that contain
    /// unsubstituted variables.
    pub fn value_is_unparsed(&self) -> bool {
      match *self {
          PropertyDeclaration::WithVariables(..) => true,
          PropertyDeclaration::Custom(_, ref value) => {
            !matches!(value.borrow(), DeclaredValue::CSSWideKeyword(..))
          }
          _ => false,
      }
    }

    /// The shorthands that this longhand is part of.
    pub fn shorthands(&self) -> &'static [ShorthandId] {
        match self.id() {
            PropertyDeclarationId::Longhand(id) => id.shorthands(),
            PropertyDeclarationId::Custom(..) => &[],
        }
    }

    /// Returns true if this property declaration is for one of the animatable
    /// properties.
    pub fn is_animatable(&self) -> bool {
        match *self {
            % for property in data.longhands:
            PropertyDeclaration::${property.camel_case}(_) => {
                % if property.animatable:
                    true
                % else:
                    false
                % endif
            }
            % endfor
            PropertyDeclaration::CSSWideKeyword(id, _) |
            PropertyDeclaration::WithVariables(id, _) => match id {
                % for property in data.longhands:
                LonghandId::${property.camel_case} => {
                    % if property.animatable:
                        true
                    % else:
                        false
                    % endif
                }
                % endfor
            },
            PropertyDeclaration::Custom(..) => false,
        }
    }

    /// Returns true if this property is a custom property, false
    /// otherwise.
    pub fn is_custom(&self) -> bool {
        matches!(*self, PropertyDeclaration::Custom(_, _))
    }

    /// The `context` parameter controls this:
    ///
    /// https://drafts.csswg.org/css-animations/#keyframes
    /// > The <declaration-list> inside of <keyframe-block> accepts any CSS property
    /// > except those defined in this specification,
    /// > but does accept the `animation-play-state` property and interprets it specially.
    ///
    /// This will not actually parse Importance values, and will always set things
    /// to Importance::Normal. Parsing Importance values is the job of PropertyDeclarationParser,
    /// we only set them here so that we don't have to reallocate
    pub fn parse_into<'i, 't>(declarations: &mut SourcePropertyDeclaration,
                              id: PropertyId, name: CowRcStr<'i>,
                              context: &ParserContext, input: &mut Parser<'i, 't>)
                              -> Result<(), PropertyDeclarationParseError<'i>> {
        assert!(declarations.is_empty());
        let start = input.state();
        match id {
            PropertyId::Custom(property_name) => {
                // FIXME: fully implement https://github.com/w3c/csswg-drafts/issues/774
                // before adding skip_whitespace here.
                // This probably affects some test results.
                let value = match input.try(|i| CSSWideKeyword::parse(i)) {
                    Ok(keyword) => DeclaredValueOwned::CSSWideKeyword(keyword),
                    Err(()) => match ::custom_properties::SpecifiedValue::parse(input) {
                        Ok(value) => DeclaredValueOwned::Value(value),
                        Err(e) => return Err(PropertyDeclarationParseError::InvalidValue(name.to_string().into(),
                        ValueParseError::from_parse_error(e))),
                    }
                };
                declarations.push(PropertyDeclaration::Custom(property_name, value));
                Ok(())
            }
            PropertyId::Longhand(id) => {
                input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
                input.try(|i| CSSWideKeyword::parse(i)).map(|keyword| {
                    PropertyDeclaration::CSSWideKeyword(id, keyword)
                }).or_else(|()| {
                    input.look_for_var_functions();
                    input.parse_entirely(|input| id.parse_value(context, input))
                    .or_else(|err| {
                        while let Ok(_) = input.next() {}  // Look for var() after the error.
                        if input.seen_var_functions() {
                            input.reset(&start);
                            let (first_token_type, css) =
                                ::custom_properties::parse_non_custom_with_var(input).map_err(|e| {
                                    PropertyDeclarationParseError::InvalidValue(name,
                                        ValueParseError::from_parse_error(e))
                                })?;
                            Ok(PropertyDeclaration::WithVariables(id, Arc::new(UnparsedValue {
                                css: css.into_owned(),
                                first_token_type: first_token_type,
                                url_data: context.url_data.clone(),
                                from_shorthand: None,
                            })))
                        } else {
                            Err(PropertyDeclarationParseError::InvalidValue(name,
                                ValueParseError::from_parse_error(err)))
                        }
                    })
                }).map(|declaration| {
                    declarations.push(declaration)
                })
            }
            PropertyId::Shorthand(id) => {
                input.skip_whitespace();  // Unnecessary for correctness, but may help try() rewind less.
                if let Ok(keyword) = input.try(|i| CSSWideKeyword::parse(i)) {
                    if id == ShorthandId::All {
                        declarations.all_shorthand = AllShorthand::CSSWideKeyword(keyword)
                    } else {
                        for &longhand in id.longhands() {
                            declarations.push(PropertyDeclaration::CSSWideKeyword(longhand, keyword))
                        }
                    }
                    Ok(())
                } else {
                    input.look_for_var_functions();
                    // Not using parse_entirely here: each ${shorthand.ident}::parse_into function
                    // needs to do so *before* pushing to `declarations`.
                    id.parse_into(declarations, context, input).or_else(|err| {
                        while let Ok(_) = input.next() {}  // Look for var() after the error.
                        if input.seen_var_functions() {
                            input.reset(&start);
                            let (first_token_type, css) =
                                ::custom_properties::parse_non_custom_with_var(input).map_err(|e| {
                                    PropertyDeclarationParseError::InvalidValue(name,
                                        ValueParseError::from_parse_error(e))
                                })?;
                            let unparsed = Arc::new(UnparsedValue {
                                css: css.into_owned(),
                                first_token_type: first_token_type,
                                url_data: context.url_data.clone(),
                                from_shorthand: Some(id),
                            });
                            if id == ShorthandId::All {
                                declarations.all_shorthand = AllShorthand::WithVariables(unparsed)
                            } else {
                                for &longhand in id.longhands() {
                                    declarations.push(
                                        PropertyDeclaration::WithVariables(longhand, unparsed.clone())
                                    )
                                }
                            }
                            Ok(())
                        } else {
                            Err(PropertyDeclarationParseError::InvalidValue(name,
                                ValueParseError::from_parse_error(err)))
                        }
                    })
                }
            }
        }
    }
}

const MAX_SUB_PROPERTIES_PER_SHORTHAND_EXCEPT_ALL: usize =
    ${max(len(s.sub_properties) for s in data.shorthands_except_all())};

type SourcePropertyDeclarationArray =
    [PropertyDeclaration; MAX_SUB_PROPERTIES_PER_SHORTHAND_EXCEPT_ALL];

/// A stack-allocated vector of `PropertyDeclaration`
/// large enough to parse one CSS `key: value` declaration.
/// (Shorthands expand to multiple `PropertyDeclaration`s.)
pub struct SourcePropertyDeclaration {
    declarations: ::arrayvec::ArrayVec<SourcePropertyDeclarationArray>,

    /// Stored separately to keep MAX_SUB_PROPERTIES_PER_SHORTHAND_EXCEPT_ALL smaller.
    all_shorthand: AllShorthand,
}

impl SourcePropertyDeclaration {
    /// Create one. It’s big, try not to move it around.
    #[inline]
    pub fn new() -> Self {
        SourcePropertyDeclaration {
            declarations: ::arrayvec::ArrayVec::new(),
            all_shorthand: AllShorthand::NotSet,
        }
    }

    /// Similar to Vec::drain: leaves this empty when the return value is dropped.
    pub fn drain(&mut self) -> SourcePropertyDeclarationDrain {
        SourcePropertyDeclarationDrain {
            declarations: self.declarations.drain(..),
            all_shorthand: mem::replace(&mut self.all_shorthand, AllShorthand::NotSet),
        }
    }

    /// Reset to initial state
    pub fn clear(&mut self) {
        self.declarations.clear();
        self.all_shorthand = AllShorthand::NotSet;
    }

    fn is_empty(&self) -> bool {
        self.declarations.is_empty() && matches!(self.all_shorthand, AllShorthand::NotSet)
    }

    fn push(&mut self, declaration: PropertyDeclaration) {
        let over_capacity = self.declarations.push(declaration).is_some();
        debug_assert!(!over_capacity);
    }
}

/// Return type of SourcePropertyDeclaration::drain
pub struct SourcePropertyDeclarationDrain<'a> {
    declarations: ::arrayvec::Drain<'a, SourcePropertyDeclarationArray>,
    all_shorthand: AllShorthand,
}

enum AllShorthand {
    NotSet,
    CSSWideKeyword(CSSWideKeyword),
    WithVariables(Arc<UnparsedValue>)
}

#[cfg(feature = "gecko")]
pub use gecko_properties::style_structs;

/// The module where all the style structs are defined.
#[cfg(feature = "servo")]
pub mod style_structs {
    use fnv::FnvHasher;
    use super::longhands;
    use std::hash::{Hash, Hasher};
    use logical_geometry::WritingMode;
    use media_queries::Device;
    use values::computed::NonNegativeLength;

    % for style_struct in data.active_style_structs():
        % if style_struct.name == "Font":
        #[derive(Clone, Debug)]
        % else:
        #[derive(Clone, Debug, PartialEq)]
        % endif
        #[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        /// The ${style_struct.name} style struct.
        pub struct ${style_struct.name} {
            % for longhand in style_struct.longhands:
                /// The ${longhand.name} computed value.
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
            % if style_struct.name == "Font":
                /// The font hash, used for font caching.
                pub hash: u64,
            % endif
        }
        % if style_struct.name == "Font":

        impl PartialEq for ${style_struct.name} {
            fn eq(&self, other: &${style_struct.name}) -> bool {
                self.hash == other.hash
                % for longhand in style_struct.longhands:
                    && self.${longhand.ident} == other.${longhand.ident}
                % endfor
            }
        }
        % endif

        impl ${style_struct.name} {
            % for longhand in style_struct.longhands:
                % if longhand.logical:
                    ${helpers.logical_setter(name=longhand.name)}
                % else:
                    % if longhand.is_vector:
                        /// Set ${longhand.name}.
                        #[allow(non_snake_case)]
                        #[inline]
                        pub fn set_${longhand.ident}<I>(&mut self, v: I)
                            where I: IntoIterator<Item = longhands::${longhand.ident}
                                                                  ::computed_value::single_value::T>,
                                  I::IntoIter: ExactSizeIterator
                        {
                            self.${longhand.ident} = longhands::${longhand.ident}::computed_value
                                                              ::T(v.into_iter().collect());
                        }
                    % else:
                        /// Set ${longhand.name}.
                        #[allow(non_snake_case)]
                        #[inline]
                        pub fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T) {
                            self.${longhand.ident} = v;
                        }
                    % endif
                    /// Set ${longhand.name} from other struct.
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn copy_${longhand.ident}_from(&mut self, other: &Self) {
                        self.${longhand.ident} = other.${longhand.ident}.clone();
                    }

                    /// Reset ${longhand.name} from the initial struct.
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn reset_${longhand.ident}(&mut self, other: &Self) {
                        self.copy_${longhand.ident}_from(other)
                    }

                    /// Get the computed value for ${longhand.name}.
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn clone_${longhand.ident}(&self) -> longhands::${longhand.ident}::computed_value::T {
                        self.${longhand.ident}.clone()
                    }
                % endif
                % if longhand.need_index:
                    /// If this longhand is indexed, get the number of elements.
                    #[allow(non_snake_case)]
                    pub fn ${longhand.ident}_count(&self) -> usize {
                        self.${longhand.ident}.0.len()
                    }

                    /// If this longhand is indexed, get the element at given
                    /// index.
                    #[allow(non_snake_case)]
                    pub fn ${longhand.ident}_at(&self, index: usize)
                        -> longhands::${longhand.ident}::computed_value::SingleComputedValue {
                        self.${longhand.ident}.0[index].clone()
                    }
                % endif
            % endfor
            % if style_struct.name == "Border":
                % for side in ["top", "right", "bottom", "left"]:
                    /// Whether the border-${side} property has nonzero width.
                    #[allow(non_snake_case)]
                    pub fn border_${side}_has_nonzero_width(&self) -> bool {
                        self.border_${side}_width != NonNegativeLength::zero()
                    }
                % endfor
            % elif style_struct.name == "Font":
                /// Computes a font hash in order to be able to cache fonts
                /// effectively in GFX and layout.
                pub fn compute_font_hash(&mut self) {
                    // Corresponds to the fields in
                    // `gfx::font_template::FontTemplateDescriptor`.
                    let mut hasher: FnvHasher = Default::default();
                    hasher.write_u16(self.font_weight.0);
                    self.font_stretch.hash(&mut hasher);
                    self.font_family.hash(&mut hasher);
                    self.hash = hasher.finish()
                }

                /// (Servo does not handle MathML, so this just calls copy_font_size_from)
                pub fn inherit_font_size_from(&mut self, parent: &Self,
                                              _: Option<NonNegativeLength>,
                                              _: &Device) {
                    self.copy_font_size_from(parent);
                }
                /// (Servo does not handle MathML, so this just calls set_font_size)
                pub fn apply_font_size(&mut self,
                                       v: longhands::font_size::computed_value::T,
                                       _: &Self,
                                       _: &Device) -> Option<NonNegativeLength> {
                    self.set_font_size(v);
                    None
                }
                /// (Servo does not handle MathML, so this does nothing)
                pub fn apply_unconstrained_font_size(&mut self, _: NonNegativeLength) {
                }

            % elif style_struct.name == "Outline":
                /// Whether the outline-width property is non-zero.
                #[inline]
                pub fn outline_has_nonzero_width(&self) -> bool {
                    self.outline_width != NonNegativeLength::zero()
                }
            % elif style_struct.name == "Text":
                /// Whether the text decoration has an underline.
                #[inline]
                pub fn has_underline(&self) -> bool {
                    self.text_decoration_line.contains(longhands::text_decoration_line::UNDERLINE)
                }

                /// Whether the text decoration has an overline.
                #[inline]
                pub fn has_overline(&self) -> bool {
                    self.text_decoration_line.contains(longhands::text_decoration_line::OVERLINE)
                }

                /// Whether the text decoration has a line through.
                #[inline]
                pub fn has_line_through(&self) -> bool {
                    self.text_decoration_line.contains(longhands::text_decoration_line::LINE_THROUGH)
                }
            % elif style_struct.name == "Box":
                /// Sets the display property, but without touching
                /// __servo_display_for_hypothetical_box, except when the
                /// adjustment comes from root or item display fixups.
                pub fn set_adjusted_display(&mut self,
                                            dpy: longhands::display::computed_value::T,
                                            is_item_or_root: bool) {
                    self.set_display(dpy);
                    if is_item_or_root {
                        self.set__servo_display_for_hypothetical_box(dpy);
                    }
                }
            % endif
        }

    % endfor
}

% for style_struct in data.active_style_structs():
    impl style_structs::${style_struct.name} {
        % for longhand in style_struct.longhands:
            % if longhand.need_index:
                /// Iterate over the values of ${longhand.name}.
                #[allow(non_snake_case)]
                #[inline]
                pub fn ${longhand.ident}_iter(&self) -> ${longhand.camel_case}Iter {
                    ${longhand.camel_case}Iter {
                        style_struct: self,
                        current: 0,
                        max: self.${longhand.ident}_count(),
                    }
                }

                /// Get a value mod `index` for the property ${longhand.name}.
                #[allow(non_snake_case)]
                #[inline]
                pub fn ${longhand.ident}_mod(&self, index: usize)
                    -> longhands::${longhand.ident}::computed_value::SingleComputedValue {
                    self.${longhand.ident}_at(index % self.${longhand.ident}_count())
                }
            % endif
        % endfor

        % if style_struct.name == "Box":
            /// Returns whether there is any animation specified with
            /// animation-name other than `none`.
            pub fn specifies_animations(&self) -> bool {
                self.animation_name_iter().any(|name| name.0.is_some())
            }

            /// Returns whether there are any transitions specified.
            #[cfg(feature = "servo")]
            pub fn specifies_transitions(&self) -> bool {
                self.transition_duration_iter()
                    .take(self.transition_property_count())
                    .any(|t| t.seconds() > 0.)
            }
        % endif
    }

    % for longhand in style_struct.longhands:
        % if longhand.need_index:
            /// An iterator over the values of the ${longhand.name} properties.
            pub struct ${longhand.camel_case}Iter<'a> {
                style_struct: &'a style_structs::${style_struct.name},
                current: usize,
                max: usize,
            }

            impl<'a> Iterator for ${longhand.camel_case}Iter<'a> {
                type Item = longhands::${longhand.ident}::computed_value::SingleComputedValue;

                fn next(&mut self) -> Option<Self::Item> {
                    self.current += 1;
                    if self.current <= self.max {
                        Some(self.style_struct.${longhand.ident}_at(self.current - 1))
                    } else {
                        None
                    }
                }
            }
        % endif
    % endfor
% endfor


#[cfg(feature = "gecko")]
pub use gecko_properties::{ComputedValues, ComputedValuesInner};

#[cfg(feature = "servo")]
#[cfg_attr(feature = "servo", derive(Clone, Debug))]
/// Actual data of ComputedValues, to match up with Gecko
pub struct ComputedValuesInner {
    % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor
    custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
    /// The writing mode of this computed values struct.
    pub writing_mode: WritingMode,

    /// A set of flags we use to store misc information regarding this style.
    pub flags: ComputedValueFlags,

    /// The rule node representing the ordered list of rules matched for this
    /// node.  Can be None for default values and text nodes.  This is
    /// essentially an optimization to avoid referencing the root rule node.
    pub rules: Option<StrongRuleNode>,

    /// The element's computed values if visited, only computed if there's a
    /// relevant link for this element. A element's "relevant link" is the
    /// element being matched if it is a link or the nearest ancestor link.
    visited_style: Option<Arc<ComputedValues>>,
}

/// The struct that Servo uses to represent computed values.
///
/// This struct contains an immutable atomically-reference-counted pointer to
/// every kind of style struct.
///
/// When needed, the structs may be copied in order to get mutated.
#[cfg(feature = "servo")]
#[cfg_attr(feature = "servo", derive(Clone, Debug))]
pub struct ComputedValues {
    /// The actual computed values
    ///
    /// In Gecko the outer ComputedValues is actually a style context,
    /// whereas ComputedValuesInner is the core set of computed values.
    ///
    /// We maintain this distinction in servo to reduce the amount of special casing.
    inner: ComputedValuesInner,
}

impl ComputedValues {
    /// Whether we're a visited style.
    pub fn is_style_if_visited(&self) -> bool {
        self.flags.contains(IS_STYLE_IF_VISITED)
    }

    /// Gets a reference to the rule node. Panic if no rule node exists.
    pub fn rules(&self) -> &StrongRuleNode {
        self.rules.as_ref().unwrap()
    }

    /// Returns the visited style, if any.
    pub fn visited_style(&self) -> Option<<&ComputedValues> {
        self.visited_style.as_ref().map(|s| &**s)
    }

    /// Returns the visited rules, if applicable.
    pub fn visited_rules(&self) -> Option<<&StrongRuleNode> {
        self.visited_style.as_ref().and_then(|s| s.rules.as_ref())
    }

    /// Returns whether we're in a display: none subtree.
    pub fn is_in_display_none_subtree(&self) -> bool {
        use properties::computed_value_flags::IS_IN_DISPLAY_NONE_SUBTREE;

        self.flags.contains(IS_IN_DISPLAY_NONE_SUBTREE)
    }

    /// Gets a reference to the custom properties map (if one exists).
    pub fn custom_properties(&self) -> Option<<&Arc<::custom_properties::CustomPropertiesMap>> {
        self.custom_properties.as_ref()
    }
}

#[cfg(feature = "servo")]
impl ComputedValues {
    /// Create a new refcounted `ComputedValues`
    pub fn new(
        _: &Device,
        _: Option<<&ComputedValues>,
        _: Option<<&PseudoElement>,
        custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
        writing_mode: WritingMode,
        flags: ComputedValueFlags,
        rules: Option<StrongRuleNode>,
        visited_style: Option<Arc<ComputedValues>>,
        % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
        % endfor
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: ComputedValuesInner {
                custom_properties,
                writing_mode,
                rules,
                visited_style,
                flags,
            % for style_struct in data.active_style_structs():
                ${style_struct.ident},
            % endfor
            }
        })
    }

    /// Get the initial computed values.
    pub fn initial_values() -> &'static Self { &*INITIAL_SERVO_VALUES }
}

#[cfg(feature = "servo")]
impl ops::Deref for ComputedValues {
    type Target = ComputedValuesInner;
    fn deref(&self) -> &ComputedValuesInner {
        &self.inner
    }
}

#[cfg(feature = "servo")]
impl ops::DerefMut for ComputedValues {
    fn deref_mut(&mut self) -> &mut ComputedValuesInner {
        &mut self.inner
    }
}

#[cfg(feature = "servo")]
impl ComputedValuesInner {
    % for style_struct in data.active_style_structs():
        /// Clone the ${style_struct.name} struct.
        #[inline]
        pub fn clone_${style_struct.name_lower}(&self) -> Arc<style_structs::${style_struct.name}> {
            self.${style_struct.ident}.clone()
        }

        /// Get a immutable reference to the ${style_struct.name} struct.
        #[inline]
        pub fn get_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
            &self.${style_struct.ident}
        }

        /// Gets an immutable reference to the refcounted value that wraps
        /// `${style_struct.name}`.
        pub fn ${style_struct.name_lower}_arc(&self) -> &Arc<style_structs::${style_struct.name}> {
            &self.${style_struct.ident}
        }

        /// Get a mutable reference to the ${style_struct.name} struct.
        #[inline]
        pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
            Arc::make_mut(&mut self.${style_struct.ident})
        }
    % endfor

    /// Gets a reference to the rule node. Panic if no rule node exists.
    pub fn rules(&self) -> &StrongRuleNode {
        self.rules.as_ref().unwrap()
    }

    /// Whether this style has a -moz-binding value. This is always false for
    /// Servo for obvious reasons.
    pub fn has_moz_binding(&self) -> bool { false }

    /// Clone the visited style.  Used for inheriting parent styles in
    /// StyleBuilder::for_inheritance.
    pub fn clone_visited_style(&self) -> Option<Arc<ComputedValues>> {
        self.visited_style.clone()
    }

    /// Returns whether this style's display value is equal to contents.
    ///
    /// Since this isn't supported in Servo, this is always false for Servo.
    pub fn is_display_contents(&self) -> bool { false }

    #[inline]
    /// Returns whether the "content" property for the given style is completely
    /// ineffective, and would yield an empty `::before` or `::after`
    /// pseudo-element.
    pub fn ineffective_content_property(&self) -> bool {
        use properties::longhands::content::computed_value::T;
        match self.get_counters().content {
            T::Normal | T::None => true,
            T::Items(ref items) => items.is_empty(),
        }
    }

    /// Whether the current style is multicolumn.
    #[inline]
    pub fn is_multicol(&self) -> bool {
        let style = self.get_column();
        match style.column_width {
            Either::First(_width) => true,
            Either::Second(_auto) => match style.column_count {
                Either::First(_n) => true,
                Either::Second(_auto) => false,
            }
        }
    }

    /// Resolves the currentColor keyword.
    ///
    /// Any color value from computed values (except for the 'color' property
    /// itself) should go through this method.
    ///
    /// Usage example:
    /// let top_color = style.resolve_color(style.Border.border_top_color);
    #[inline]
    pub fn resolve_color(&self, color: computed::Color) -> RGBA {
        color.to_rgba(self.get_color().color)
    }

    /// Get the logical computed inline size.
    #[inline]
    pub fn content_inline_size(&self) -> computed::LengthOrPercentageOrAuto {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() {
            position_style.height
        } else {
            position_style.width
        }
    }

    /// Get the logical computed block size.
    #[inline]
    pub fn content_block_size(&self) -> computed::LengthOrPercentageOrAuto {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.width } else { position_style.height }
    }

    /// Get the logical computed min inline size.
    #[inline]
    pub fn min_inline_size(&self) -> computed::LengthOrPercentage {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.min_height } else { position_style.min_width }
    }

    /// Get the logical computed min block size.
    #[inline]
    pub fn min_block_size(&self) -> computed::LengthOrPercentage {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.min_width } else { position_style.min_height }
    }

    /// Get the logical computed max inline size.
    #[inline]
    pub fn max_inline_size(&self) -> computed::LengthOrPercentageOrNone {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.max_height } else { position_style.max_width }
    }

    /// Get the logical computed max block size.
    #[inline]
    pub fn max_block_size(&self) -> computed::LengthOrPercentageOrNone {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.max_width } else { position_style.max_height }
    }

    /// Get the logical computed padding for this writing mode.
    #[inline]
    pub fn logical_padding(&self) -> LogicalMargin<computed::LengthOrPercentage> {
        let padding_style = self.get_padding();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            padding_style.padding_top.0,
            padding_style.padding_right.0,
            padding_style.padding_bottom.0,
            padding_style.padding_left.0,
        ))
    }

    /// Get the logical border width
    #[inline]
    pub fn border_width_for_writing_mode(&self, writing_mode: WritingMode) -> LogicalMargin<Au> {
        let border_style = self.get_border();
        LogicalMargin::from_physical(writing_mode, SideOffsets2D::new(
            Au::from(border_style.border_top_width),
            Au::from(border_style.border_right_width),
            Au::from(border_style.border_bottom_width),
            Au::from(border_style.border_left_width),
        ))
    }

    /// Gets the logical computed border widths for this style.
    #[inline]
    pub fn logical_border_width(&self) -> LogicalMargin<Au> {
        self.border_width_for_writing_mode(self.writing_mode)
    }

    /// Gets the logical computed margin from this style.
    #[inline]
    pub fn logical_margin(&self) -> LogicalMargin<computed::LengthOrPercentageOrAuto> {
        let margin_style = self.get_margin();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            margin_style.margin_top,
            margin_style.margin_right,
            margin_style.margin_bottom,
            margin_style.margin_left,
        ))
    }

    /// Gets the logical position from this style.
    #[inline]
    pub fn logical_position(&self) -> LogicalMargin<computed::LengthOrPercentageOrAuto> {
        // FIXME(SimonSapin): should be the writing mode of the containing block, maybe?
        let position_style = self.get_position();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            position_style.top,
            position_style.right,
            position_style.bottom,
            position_style.left,
        ))
    }

    /// Return true if the effects force the transform style to be Flat
    pub fn overrides_transform_style(&self) -> bool {
        use computed_values::mix_blend_mode;

        let effects = self.get_effects();
        // TODO(gw): Add clip-path, isolation, mask-image, mask-border-source when supported.
        effects.opacity < 1.0 ||
           !effects.filter.0.is_empty() ||
           !effects.clip.is_auto() ||
           effects.mix_blend_mode != mix_blend_mode::T::normal
    }

    /// https://drafts.csswg.org/css-transforms/#grouping-property-values
    pub fn get_used_transform_style(&self) -> computed_values::transform_style::T {
        use computed_values::transform_style;

        let box_ = self.get_box();

        if self.overrides_transform_style() {
            transform_style::T::flat
        } else {
            // Return the computed value if not overridden by the above exceptions
            box_.transform_style
        }
    }

    /// Whether given this transform value, the compositor would require a
    /// layer.
    pub fn transform_requires_layer(&self) -> bool {
        // Check if the transform matrix is 2D or 3D
        if let Some(ref transform_list) = self.get_box().transform.0 {
            for transform in transform_list {
                match *transform {
                    computed_values::transform::ComputedOperation::Perspective(..) => {
                        return true;
                    }
                    computed_values::transform::ComputedOperation::Matrix(m) => {
                        // See http://dev.w3.org/csswg/css-transforms/#2d-matrix
                        if m.m31 != 0.0 || m.m32 != 0.0 ||
                           m.m13 != 0.0 || m.m23 != 0.0 ||
                           m.m43 != 0.0 || m.m14 != 0.0 ||
                           m.m24 != 0.0 || m.m34 != 0.0 ||
                           m.m33 != 1.0 || m.m44 != 1.0 {
                            return true;
                        }
                    }
                    computed_values::transform::ComputedOperation::Translate(_, _, z) => {
                        if z.px() != 0. {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Neither perspective nor transform present
        false
    }

    /// Serializes the computed value of this property as a string.
    pub fn computed_value_to_string(&self, property: PropertyDeclarationId) -> String {
        match property {
            % for style_struct in data.active_style_structs():
                % for longhand in style_struct.longhands:
                    PropertyDeclarationId::Longhand(LonghandId::${longhand.camel_case}) => {
                        self.${style_struct.ident}.${longhand.ident}.to_css_string()
                    }
                % endfor
            % endfor
            PropertyDeclarationId::Custom(name) => {
                self.custom_properties
                    .as_ref()
                    .and_then(|map| map.get(name))
                    .map(|value| value.to_css_string())
                    .unwrap_or(String::new())
            }
        }
    }
}

/// Return a WritingMode bitflags from the relevant CSS properties.
pub fn get_writing_mode(inheritedbox_style: &style_structs::InheritedBox) -> WritingMode {
    use logical_geometry;
    let mut flags = WritingMode::empty();
    match inheritedbox_style.clone_direction() {
        computed_values::direction::T::ltr => {},
        computed_values::direction::T::rtl => {
            flags.insert(logical_geometry::FLAG_RTL);
        },
    }
    match inheritedbox_style.clone_writing_mode() {
        computed_values::writing_mode::T::horizontal_tb => {},
        computed_values::writing_mode::T::vertical_rl => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
        },
        computed_values::writing_mode::T::vertical_lr => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
        },
        % if product == "gecko":
        computed_values::writing_mode::T::sideways_rl => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
            flags.insert(logical_geometry::FLAG_SIDEWAYS);
        },
        computed_values::writing_mode::T::sideways_lr => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
            flags.insert(logical_geometry::FLAG_LINE_INVERTED);
            flags.insert(logical_geometry::FLAG_SIDEWAYS);
        },
        % endif
    }
    % if product == "gecko":
    // If FLAG_SIDEWAYS is already set, this means writing-mode is either
    // sideways-rl or sideways-lr, and for both of these values,
    // text-orientation has no effect.
    if !flags.intersects(logical_geometry::FLAG_SIDEWAYS) {
        match inheritedbox_style.clone_text_orientation() {
            computed_values::text_orientation::T::mixed => {},
            computed_values::text_orientation::T::upright => {
                flags.insert(logical_geometry::FLAG_UPRIGHT);
            },
            computed_values::text_orientation::T::sideways => {
                flags.insert(logical_geometry::FLAG_SIDEWAYS);
            },
        }
    }
    % endif
    flags
}

% if product == "gecko":
    pub use ::servo_arc::RawOffsetArc as BuilderArc;
    /// Clone an arc, returning a regular arc
    fn clone_arc<T: 'static>(x: &BuilderArc<T>) -> Arc<T> {
        Arc::from_raw_offset(x.clone())
    }
% else:
    pub use ::servo_arc::Arc as BuilderArc;
    /// Clone an arc, returning a regular arc
    fn clone_arc<T: 'static>(x: &BuilderArc<T>) -> Arc<T> {
        x.clone()
    }
% endif

/// A reference to a style struct of the parent, or our own style struct.
pub enum StyleStructRef<'a, T: 'static> {
    /// A borrowed struct from the parent, for example, for inheriting style.
    Borrowed(&'a BuilderArc<T>),
    /// An owned struct, that we've already mutated.
    Owned(UniqueArc<T>),
    /// Temporarily vacated, will panic if accessed
    Vacated,
}

impl<'a, T: 'a> StyleStructRef<'a, T>
    where T: Clone,
{
    /// Ensure a mutable reference of this value exists, either cloning the
    /// borrowed value, or returning the owned one.
    pub fn mutate(&mut self) -> &mut T {
        if let StyleStructRef::Borrowed(v) = *self {
            *self = StyleStructRef::Owned(UniqueArc::new((**v).clone()));
        }

        match *self {
            StyleStructRef::Owned(ref mut v) => v,
            StyleStructRef::Borrowed(..) => unreachable!(),
            StyleStructRef::Vacated => panic!("Accessed vacated style struct")
        }
    }

    /// Extract a unique Arc from this struct, vacating it.
    ///
    /// The vacated state is a transient one, please put the Arc back
    /// when done via `put()`. This function is to be used to separate
    /// the struct being mutated from the computed context
    pub fn take(&mut self) -> UniqueArc<T> {
        use std::mem::replace;
        let inner = replace(self, StyleStructRef::Vacated);

        match inner {
            StyleStructRef::Owned(arc) => arc,
            StyleStructRef::Borrowed(arc) => UniqueArc::new((**arc).clone()),
            StyleStructRef::Vacated => panic!("Accessed vacated style struct"),
        }
    }

    /// Replace vacated ref with an arc
    pub fn put(&mut self, arc: UniqueArc<T>) {
        debug_assert!(matches!(*self, StyleStructRef::Vacated));
        *self = StyleStructRef::Owned(arc);
    }

    /// Get a mutable reference to the owned struct, or `None` if the struct
    /// hasn't been mutated.
    pub fn get_if_mutated(&mut self) -> Option<<&mut T> {
        match *self {
            StyleStructRef::Owned(ref mut v) => Some(v),
            StyleStructRef::Borrowed(..) => None,
            StyleStructRef::Vacated => panic!("Accessed vacated style struct")
        }
    }

    /// Returns an `Arc` to the internal struct, constructing one if
    /// appropriate.
    pub fn build(self) -> Arc<T> {
        match self {
            StyleStructRef::Owned(v) => v.shareable(),
            StyleStructRef::Borrowed(v) => clone_arc(v),
            StyleStructRef::Vacated => panic!("Accessed vacated style struct")
        }
    }
}

impl<'a, T: 'a> ops::Deref for StyleStructRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            StyleStructRef::Owned(ref v) => &**v,
            StyleStructRef::Borrowed(v) => &**v,
            StyleStructRef::Vacated => panic!("Accessed vacated style struct")
        }
    }
}

/// A type used to compute a struct with minimal overhead.
///
/// This allows holding references to the parent/default computed values without
/// actually cloning them, until we either build the style, or mutate the
/// inherited value.
pub struct StyleBuilder<'a> {
    /// The device we're using to compute style.
    ///
    /// This provides access to viewport unit ratios, etc.
    pub device: &'a Device,

    /// The style we're inheriting from.
    ///
    /// This is effectively
    /// `parent_style.unwrap_or(device.default_computed_values())`.
    inherited_style: &'a ComputedValues,

    /// The style we're inheriting from for properties that don't inherit from
    /// ::first-line.  This is the same as inherited_style, unless
    /// inherited_style is a ::first-line style.
    inherited_style_ignoring_first_line: &'a ComputedValues,

    /// The style we're getting reset structs from.
    reset_style: &'a ComputedValues,

    /// The style we're inheriting from explicitly, or none if we're the root of
    /// a subtree.
    parent_style: Option<<&'a ComputedValues>,

    /// The rule node representing the ordered list of rules matched for this
    /// node.
    pub rules: Option<StrongRuleNode>,

    custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,

    /// The pseudo-element this style will represent.
    pub pseudo: Option<<&'a PseudoElement>,

    /// Whether we have mutated any reset structs since the the last time
    /// `clear_modified_reset` was called.  This is used to tell whether the
    /// `StyleAdjuster` did any work.
    modified_reset: bool,

    /// The writing mode flags.
    ///
    /// TODO(emilio): Make private.
    pub writing_mode: WritingMode,
    /// Flags for the computed value.
    pub flags: ComputedValueFlags,
    /// The element's style if visited, only computed if there's a relevant link
    /// for this element.  A element's "relevant link" is the element being
    /// matched if it is a link or the nearest ancestor link.
    visited_style: Option<Arc<ComputedValues>>,
    % for style_struct in data.active_style_structs():
        ${style_struct.ident}: StyleStructRef<'a, style_structs::${style_struct.name}>,
    % endfor
}

impl<'a> StyleBuilder<'a> {
    /// Trivially construct a `StyleBuilder`.
    fn new(
        device: &'a Device,
        parent_style: Option<<&'a ComputedValues>,
        parent_style_ignoring_first_line: Option<<&'a ComputedValues>,
        pseudo: Option<<&'a PseudoElement>,
        cascade_flags: CascadeFlags,
        rules: Option<StrongRuleNode>,
        custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
        writing_mode: WritingMode,
        mut flags: ComputedValueFlags,
        visited_style: Option<Arc<ComputedValues>>,
    ) -> Self {
        debug_assert_eq!(parent_style.is_some(), parent_style_ignoring_first_line.is_some());
        #[cfg(feature = "gecko")]
        debug_assert!(parent_style.is_none() ||
                      ::std::ptr::eq(parent_style.unwrap(),
                                     parent_style_ignoring_first_line.unwrap()) ||
                      parent_style.unwrap().pseudo() == Some(PseudoElement::FirstLine));
        let reset_style = device.default_computed_values();
        let inherited_style = parent_style.unwrap_or(reset_style);
        let inherited_style_ignoring_first_line = parent_style_ignoring_first_line.unwrap_or(reset_style);
        // FIXME(bz): INHERIT_ALL seems like a fundamentally broken idea.  I'm
        // 99% sure it should give incorrect behavior for table anonymous box
        // backgrounds, for example.  This code doesn't attempt to make it play
        // nice with inherited_style_ignoring_first_line.
        let reset_style = if cascade_flags.contains(INHERIT_ALL) {
            inherited_style
        } else {
            reset_style
        };

        if cascade_flags.contains(VISITED_DEPENDENT_ONLY) {
            flags.insert(IS_STYLE_IF_VISITED);
        }

        StyleBuilder {
            device,
            parent_style,
            inherited_style,
            inherited_style_ignoring_first_line,
            reset_style,
            pseudo,
            rules,
            modified_reset: false,
            custom_properties,
            writing_mode,
            flags,
            visited_style,
            % for style_struct in data.active_style_structs():
            % if style_struct.inherited:
            ${style_struct.ident}: StyleStructRef::Borrowed(inherited_style.${style_struct.name_lower}_arc()),
            % else:
            ${style_struct.ident}: StyleStructRef::Borrowed(reset_style.${style_struct.name_lower}_arc()),
            % endif
            % endfor
        }
    }

    /// Whether we're a visited style.
    pub fn is_style_if_visited(&self) -> bool {
        self.flags.contains(IS_STYLE_IF_VISITED)
    }

    /// Creates a StyleBuilder holding only references to the structs of `s`, in
    /// order to create a derived style.
    pub fn for_derived_style(
        device: &'a Device,
        style_to_derive_from: &'a ComputedValues,
        parent_style: Option<<&'a ComputedValues>,
        pseudo: Option<<&'a PseudoElement>,
    ) -> Self {
        let reset_style = device.default_computed_values();
        let inherited_style = parent_style.unwrap_or(reset_style);
        #[cfg(feature = "gecko")]
        debug_assert!(parent_style.is_none() ||
                      parent_style.unwrap().pseudo() != Some(PseudoElement::FirstLine));
        StyleBuilder {
            device,
            parent_style,
            inherited_style,
            // None of our callers pass in ::first-line parent styles.
            inherited_style_ignoring_first_line: inherited_style,
            reset_style,
            pseudo,
            modified_reset: false,
            rules: None, // FIXME(emilio): Dubious...
            custom_properties: style_to_derive_from.custom_properties().cloned(),
            writing_mode: style_to_derive_from.writing_mode,
            flags: style_to_derive_from.flags,
            visited_style: style_to_derive_from.clone_visited_style(),
            % for style_struct in data.active_style_structs():
            ${style_struct.ident}: StyleStructRef::Borrowed(
                style_to_derive_from.${style_struct.name_lower}_arc()
            ),
            % endfor
        }
    }

    /// Copy the reset properties from `style`.
    pub fn copy_reset_from(&mut self, style: &'a ComputedValues) {
        % for style_struct in data.active_style_structs():
        % if not style_struct.inherited:
        self.${style_struct.ident} =
            StyleStructRef::Borrowed(style.${style_struct.name_lower}_arc());
        % endif
        % endfor
    }

    % for property in data.longhands:
    % if property.ident != "font_size":
    /// Inherit `${property.ident}` from our parent style.
    #[allow(non_snake_case)]
    pub fn inherit_${property.ident}(&mut self) {
        let inherited_struct =
        % if property.style_struct.inherited:
            self.inherited_style.get_${property.style_struct.name_lower}();
        % else:
            self.inherited_style_ignoring_first_line
                .get_${property.style_struct.name_lower}();
        % endif

        % if not property.style_struct.inherited:
        self.flags.insert(::properties::computed_value_flags::INHERITS_RESET_STYLE);
        self.modified_reset = true;
        % endif

        % if property.ident == "content":
        self.flags.insert(::properties::computed_value_flags::INHERITS_CONTENT);
        % endif

        % if property.ident == "display":
        self.flags.insert(::properties::computed_value_flags::INHERITS_DISPLAY);
        % endif

        self.${property.style_struct.ident}.mutate()
            .copy_${property.ident}_from(
                inherited_struct,
                % if property.logical:
                self.writing_mode,
                % endif
            );
    }

    /// Reset `${property.ident}` to the initial value.
    #[allow(non_snake_case)]
    pub fn reset_${property.ident}(&mut self) {
        let reset_struct =
            self.reset_style.get_${property.style_struct.name_lower}();

        % if not property.style_struct.inherited:
        self.modified_reset = true;
        % endif

        self.${property.style_struct.ident}.mutate()
            .reset_${property.ident}(
                reset_struct,
                % if property.logical:
                self.writing_mode,
                % endif
            );
    }

    % if not property.is_vector:
    /// Set the `${property.ident}` to the computed value `value`.
    #[allow(non_snake_case)]
    pub fn set_${property.ident}(
        &mut self,
        value: longhands::${property.ident}::computed_value::T
    ) {
        % if not property.style_struct.inherited:
        self.modified_reset = true;
        % endif

        <% props_need_device = ["content", "list_style_type", "font_variant_alternates"] %>
        self.${property.style_struct.ident}.mutate()
            .set_${property.ident}(
                value,
                % if property.logical:
                self.writing_mode,
                % elif product == "gecko" and property.ident in props_need_device:
                self.device,
                % endif
            );
    }
    % endif
    % endif
    % endfor

    /// Inherits style from the parent element, accounting for the default
    /// computed values that need to be provided as well.
    pub fn for_inheritance(
        device: &'a Device,
        parent: &'a ComputedValues,
        pseudo: Option<<&'a PseudoElement>,
    ) -> Self {
        // FIXME(emilio): This Some(parent) here is inconsistent with what we
        // usually do if `parent` is the default computed values, but that's
        // fine, and we want to eventually get rid of it.
        Self::new(
            device,
            Some(parent),
            Some(parent),
            pseudo,
            CascadeFlags::empty(),
            /* rules = */ None,
            parent.custom_properties().cloned(),
            parent.writing_mode,
            parent.flags,
            parent.clone_visited_style()
        )
    }

    /// Returns whether we have a visited style.
    pub fn has_visited_style(&self) -> bool {
        self.visited_style.is_some()
    }

    /// Returns whether we're a pseudo-elements style.
    pub fn is_pseudo_element(&self) -> bool {
        self.pseudo.map_or(false, |p| !p.is_anon_box())
    }

    /// Returns the style we're getting reset properties from.
    pub fn default_style(&self) -> &'a ComputedValues {
        self.reset_style
    }

    % for style_struct in data.active_style_structs():
        /// Gets an immutable view of the current `${style_struct.name}` style.
        pub fn get_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
            &self.${style_struct.ident}
        }

        /// Gets a mutable view of the current `${style_struct.name}` style.
        pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
            % if not property.style_struct.inherited:
            self.modified_reset = true;
            % endif
            self.${style_struct.ident}.mutate()
        }

        /// Gets a mutable view of the current `${style_struct.name}` style.
        pub fn take_${style_struct.name_lower}(&mut self) -> UniqueArc<style_structs::${style_struct.name}> {
            % if not property.style_struct.inherited:
            self.modified_reset = true;
            % endif
            self.${style_struct.ident}.take()
        }

        /// Gets a mutable view of the current `${style_struct.name}` style.
        pub fn put_${style_struct.name_lower}(&mut self, s: UniqueArc<style_structs::${style_struct.name}>) {
            self.${style_struct.ident}.put(s)
        }

        /// Gets a mutable view of the current `${style_struct.name}` style,
        /// only if it's been mutated before.
        pub fn get_${style_struct.name_lower}_if_mutated(&mut self)
                                                         -> Option<<&mut style_structs::${style_struct.name}> {
            self.${style_struct.ident}.get_if_mutated()
        }

        /// Reset the current `${style_struct.name}` style to its default value.
        pub fn reset_${style_struct.name_lower}_struct(&mut self) {
            self.${style_struct.ident} =
                StyleStructRef::Borrowed(self.reset_style.${style_struct.name_lower}_arc());
        }
    % endfor

    /// Returns whether this computed style represents a floated object.
    pub fn floated(&self) -> bool {
        self.get_box().clone_float() != longhands::float::computed_value::T::none
    }

    /// Returns whether this computed style represents an out of flow-positioned
    /// object.
    pub fn out_of_flow_positioned(&self) -> bool {
        use properties::longhands::position::computed_value::T as position;
        matches!(self.get_box().clone_position(),
                 position::absolute | position::fixed)
    }

    /// Whether this style has a top-layer style. That's implemented in Gecko
    /// via the -moz-top-layer property, but servo doesn't have any concept of a
    /// top layer (yet, it's needed for fullscreen).
    #[cfg(feature = "servo")]
    pub fn in_top_layer(&self) -> bool { false }

    /// Whether this style has a top-layer style.
    #[cfg(feature = "gecko")]
    pub fn in_top_layer(&self) -> bool {
        matches!(self.get_box().clone__moz_top_layer(),
                 longhands::_moz_top_layer::computed_value::T::top)
    }

    /// Clears the "have any reset structs been modified" flag.
    fn clear_modified_reset(&mut self) {
        self.modified_reset = false;
    }

    /// Returns whether we have mutated any reset structs since the the last
    /// time `clear_modified_reset` was called.
    fn modified_reset(&self) -> bool {
        self.modified_reset
    }

    /// Turns this `StyleBuilder` into a proper `ComputedValues` instance.
    pub fn build(self) -> Arc<ComputedValues> {
        ComputedValues::new(
            self.device,
            self.parent_style,
            self.pseudo,
            self.custom_properties,
            self.writing_mode,
            self.flags,
            self.rules,
            self.visited_style,
            % for style_struct in data.active_style_structs():
            self.${style_struct.ident}.build(),
            % endfor
        )
    }

    /// Get the custom properties map if necessary.
    ///
    /// Cloning the Arc here is fine because it only happens in the case where
    /// we have custom properties, and those are both rare and expensive.
    fn custom_properties(&self) -> Option<<&Arc<::custom_properties::CustomPropertiesMap>> {
        self.custom_properties.as_ref()
    }

    /// Access to various information about our inherited styles.  We don't
    /// expose an inherited ComputedValues directly, because in the
    /// ::first-line case some of the inherited information needs to come from
    /// one ComputedValues instance and some from a different one.

    /// Inherited writing-mode.
    pub fn inherited_writing_mode(&self) -> &WritingMode {
        &self.inherited_style.writing_mode
    }

    /// Inherited style flags.
    pub fn inherited_flags(&self) -> &ComputedValueFlags {
        &self.inherited_style.flags
    }

    /// And access to inherited style structs.
    % for style_struct in data.active_style_structs():
        /// Gets our inherited `${style_struct.name}`.  We don't name these
        /// accessors `inherited_${style_struct.name_lower}` because we already
        /// have things like "box" vs "inherited_box" as struct names.  Do the
        /// next-best thing and call them `parent_${style_struct.name_lower}`
        /// instead.
        pub fn get_parent_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
            % if style_struct.inherited:
            self.inherited_style.get_${style_struct.name_lower}()
            % else:
            self.inherited_style_ignoring_first_line.get_${style_struct.name_lower}()
            % endif
        }
    % endfor
}

#[cfg(feature = "servo")]
pub use self::lazy_static_module::INITIAL_SERVO_VALUES;

// Use a module to work around #[cfg] on lazy_static! not being applied to every generated item.
#[cfg(feature = "servo")]
#[allow(missing_docs)]
mod lazy_static_module {
    use logical_geometry::WritingMode;
    use servo_arc::Arc;
    use super::{ComputedValues, ComputedValuesInner, longhands, style_structs};
    use super::computed_value_flags::ComputedValueFlags;

    /// The initial values for all style structs as defined by the specification.
    lazy_static! {
        pub static ref INITIAL_SERVO_VALUES: ComputedValues = ComputedValues {
            inner: ComputedValuesInner {
                % for style_struct in data.active_style_structs():
                    ${style_struct.ident}: Arc::new(style_structs::${style_struct.name} {
                        % for longhand in style_struct.longhands:
                            ${longhand.ident}: longhands::${longhand.ident}::get_initial_value(),
                        % endfor
                        % if style_struct.name == "Font":
                            hash: 0,
                        % endif
                    }),
                % endfor
                custom_properties: None,
                writing_mode: WritingMode::empty(),
                rules: None,
                visited_style: None,
                flags: ComputedValueFlags::empty(),
            }
        };
    }
}

/// A per-longhand function that performs the CSS cascade for that longhand.
pub type CascadePropertyFn =
    extern "Rust" fn(
        declaration: &PropertyDeclaration,
        context: &mut computed::Context,
    );

/// A per-longhand array of functions to perform the CSS cascade on each of
/// them, effectively doing virtual dispatch.
static CASCADE_PROPERTY: [CascadePropertyFn; ${len(data.longhands)}] = [
    % for property in data.longhands:
        longhands::${property.ident}::cascade_property,
    % endfor
];

bitflags! {
    /// A set of flags to tweak the behavior of the `cascade` function.
    pub flags CascadeFlags: u8 {
        /// Whether to inherit all styles from the parent. If this flag is not
        /// present, non-inherited styles are reset to their initial values.
        const INHERIT_ALL = 1,

        /// Whether to skip any display style fixup for root element, flex/grid
        /// item, and ruby descendants.
        const SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP = 1 << 1,

        /// Whether to only cascade properties that are visited dependent.
        const VISITED_DEPENDENT_ONLY = 1 << 2,

        /// Whether the given element we're styling is the document element,
        /// that is, matches :root.
        ///
        /// Not set for native anonymous content since some NAC form their own
        /// root, but share the device.
        ///
        /// This affects some style adjustments, like blockification, and means
        /// that it may affect global state, like the Device's root font-size.
        const IS_ROOT_ELEMENT = 1 << 3,

        /// Whether to convert display:contents into display:inline.  This
        /// is used by Gecko to prevent display:contents on generated
        /// content.
        const PROHIBIT_DISPLAY_CONTENTS = 1 << 4,

        /// Whether we're styling the ::-moz-fieldset-content anonymous box.
        const IS_FIELDSET_CONTENT = 1 << 5,

        /// Whether we're computing the style of a link, either visited or
        /// unvisited.
        const IS_LINK = 1 << 6,

        /// Whether we're computing the style of a link element that happens to
        /// be visited.
        const IS_VISITED_LINK = 1 << 7,
    }
}

/// Performs the CSS cascade, computing new styles for an element from its parent style.
///
/// The arguments are:
///
///   * `device`: Used to get the initial viewport and other external state.
///
///   * `rule_node`: The rule node in the tree that represent the CSS rules that
///   matched.
///
///   * `parent_style`: The parent style, if applicable; if `None`, this is the root node.
///
/// Returns the computed values.
///   * `flags`: Various flags.
///
pub fn cascade(
    device: &Device,
    pseudo: Option<<&PseudoElement>,
    rule_node: &StrongRuleNode,
    guards: &StylesheetGuards,
    parent_style: Option<<&ComputedValues>,
    parent_style_ignoring_first_line: Option<<&ComputedValues>,
    layout_parent_style: Option<<&ComputedValues>,
    visited_style: Option<Arc<ComputedValues>>,
    font_metrics_provider: &FontMetricsProvider,
    flags: CascadeFlags,
    quirks_mode: QuirksMode,
    rule_cache: Option<<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
) -> Arc<ComputedValues> {
    debug_assert_eq!(parent_style.is_some(), parent_style_ignoring_first_line.is_some());
    let empty = SmallBitVec::new();
    let iter_declarations = || {
        rule_node.self_and_ancestors().flat_map(|node| {
            let cascade_level = node.cascade_level();
            let source = node.style_source();

            let declarations = if source.is_some() {
                source.read(cascade_level.guard(guards)).declaration_importance_iter()
            } else {
                // The root node has no style source.
                DeclarationImportanceIterator::new(&[], &empty)
            };
            let node_importance = node.importance();

            let property_restriction = pseudo.and_then(|p| p.property_restriction());

            declarations
                // Yield declarations later in source order (with more precedence) first.
                .rev()
                .filter_map(move |(declaration, declaration_importance)| {
                    if let Some(property_restriction) = property_restriction {
                        // declaration.id() is either a longhand or a custom
                        // property.  Custom properties are always allowed, but
                        // longhands are only allowed if they have our
                        // property_restriction flag set.
                        if let PropertyDeclarationId::Longhand(id) = declaration.id() {
                            if !id.flags().contains(property_restriction) {
                                return None
                            }
                        }
                    }

                    if declaration_importance == node_importance {
                        Some((declaration, cascade_level))
                    } else {
                        None
                    }
                })
        })
    };
    apply_declarations(
        device,
        pseudo,
        rule_node,
        iter_declarations,
        parent_style,
        parent_style_ignoring_first_line,
        layout_parent_style,
        visited_style,
        font_metrics_provider,
        flags,
        quirks_mode,
        rule_cache,
        rule_cache_conditions,
    )
}

/// NOTE: This function expects the declaration with more priority to appear
/// first.
#[allow(unused_mut)] // conditionally compiled code for "position"
pub fn apply_declarations<'a, F, I>(
    device: &Device,
    pseudo: Option<<&PseudoElement>,
    rules: &StrongRuleNode,
    iter_declarations: F,
    parent_style: Option<<&ComputedValues>,
    parent_style_ignoring_first_line: Option<<&ComputedValues>,
    layout_parent_style: Option<<&ComputedValues>,
    visited_style: Option<Arc<ComputedValues>>,
    font_metrics_provider: &FontMetricsProvider,
    flags: CascadeFlags,
    quirks_mode: QuirksMode,
    rule_cache: Option<<&RuleCache>,
    rule_cache_conditions: &mut RuleCacheConditions,
) -> Arc<ComputedValues>
where
    F: Fn() -> I,
    I: Iterator<Item = (&'a PropertyDeclaration, CascadeLevel)>,
{
    debug_assert!(layout_parent_style.is_none() || parent_style.is_some());
    debug_assert_eq!(parent_style.is_some(), parent_style_ignoring_first_line.is_some());
    #[cfg(feature = "gecko")]
    debug_assert!(parent_style.is_none() ||
                  ::std::ptr::eq(parent_style.unwrap(),
                                 parent_style_ignoring_first_line.unwrap()) ||
                  parent_style.unwrap().pseudo() == Some(PseudoElement::FirstLine));
    let (inherited_style, layout_parent_style) = match parent_style {
        Some(parent_style) => {
            (parent_style,
             layout_parent_style.unwrap_or(parent_style))
        },
        None => {
            (device.default_computed_values(),
             device.default_computed_values())
        }
    };

    let custom_properties = {
        let mut builder =
            CustomPropertiesBuilder::new(inherited_style.custom_properties());

        for (declaration, _cascade_level) in iter_declarations() {
            if let PropertyDeclaration::Custom(ref name, ref value) = *declaration {
                builder.cascade(name, value.borrow());
            }
        }

        builder.build()
    };

    let mut context = computed::Context {
        is_root_element: flags.contains(IS_ROOT_ELEMENT),
        // We'd really like to own the rules here to avoid refcount traffic, but
        // animation's usage of `apply_declarations` make this tricky. See bug
        // 1375525.
        builder: StyleBuilder::new(
            device,
            parent_style,
            parent_style_ignoring_first_line,
            pseudo,
            flags,
            Some(rules.clone()),
            custom_properties,
            WritingMode::empty(),
            ComputedValueFlags::empty(),
            visited_style,
        ),
        cached_system_font: None,
        in_media_query: false,
        for_smil_animation: false,
        for_non_inherited_property: None,
        font_metrics_provider,
        quirks_mode,
        rule_cache_conditions: RefCell::new(rule_cache_conditions),
    };

    let ignore_colors = !device.use_document_colors();
    let default_background_color_decl = if ignore_colors {
        let color = device.default_background_color();
        Some(PropertyDeclaration::BackgroundColor(color.into()))
    } else {
        None
    };

    // Set computed values, overwriting earlier declarations for the same
    // property.
    let mut seen = LonghandIdSet::new();

    // Declaration blocks are stored in increasing precedence order, we want
    // them in decreasing order here.
    //
    // We could (and used to) use a pattern match here, but that bloats this
    // function to over 100K of compiled code!
    //
    // To improve i-cache behavior, we outline the individual functions and use
    // virtual dispatch instead.
    let mut apply_reset = true;
    % for category_to_cascade_now in ["early", "other"]:
        % if category_to_cascade_now == "early":
            // Pull these out so that we can compute them in a specific order
            // without introducing more iterations.
            let mut font_size = None;
            let mut font_family = None;
        % endif
        for (declaration, cascade_level) in iter_declarations() {
            let mut declaration = match *declaration {
                PropertyDeclaration::WithVariables(id, ref unparsed) => {
                    if !id.inherited() {
                        context.rule_cache_conditions.borrow_mut()
                            .set_uncacheable();
                    }
                    Cow::Owned(unparsed.substitute_variables(
                        id,
                        context.builder.custom_properties.as_ref(),
                        context.quirks_mode
                    ))
                }
                ref d => Cow::Borrowed(d)
            };

            let longhand_id = match declaration.id() {
                PropertyDeclarationId::Longhand(id) => id,
                PropertyDeclarationId::Custom(..) => continue,
            };

            // Only a few properties are allowed to depend on the visited state
            // of links.  When cascading visited styles, we can save time by
            // only processing these properties.
            if flags.contains(VISITED_DEPENDENT_ONLY) &&
               !longhand_id.is_visited_dependent() {
                continue
            }

            if !apply_reset && !longhand_id.inherited() {
                continue;
            }

            // When document colors are disabled, skip properties that are
            // marked as ignored in that mode, if they come from a UA or
            // user style sheet.
            if ignore_colors &&
               longhand_id.is_ignored_when_document_colors_disabled() &&
               !matches!(cascade_level,
                         CascadeLevel::UANormal |
                         CascadeLevel::UserNormal |
                         CascadeLevel::UserImportant |
                         CascadeLevel::UAImportant) {
                let non_transparent_background = match *declaration {
                    PropertyDeclaration::BackgroundColor(ref color) => {
                        // Treat background-color a bit differently.  If the specified
                        // color is anything other than a fully transparent color, convert
                        // it into the Device's default background color.
                        color.is_non_transparent()
                    }
                    _ => continue
                };
                // FIXME: moving this out of `match` is a work around for borrows being lexical.
                if non_transparent_background {
                    declaration = Cow::Borrowed(default_background_color_decl.as_ref().unwrap());
                }
            }

            if
                % if category_to_cascade_now == "early":
                    !
                % endif
                longhand_id.is_early_property()
            {
                continue
            }

            <% maybe_to_physical = ".to_physical(writing_mode)" if category_to_cascade_now != "early" else "" %>
            let physical_longhand_id = longhand_id ${maybe_to_physical};
            if seen.contains(physical_longhand_id) {
                continue
            }
            seen.insert(physical_longhand_id);

            % if category_to_cascade_now == "early":
                if LonghandId::FontSize == longhand_id {
                    font_size = Some(declaration.clone());
                    continue;
                }
                if LonghandId::FontFamily == longhand_id {
                    font_family = Some(declaration.clone());
                    continue;
                }
            % endif

            let discriminant = longhand_id as usize;
            (CASCADE_PROPERTY[discriminant])(&*declaration, &mut context);
        }
        % if category_to_cascade_now == "early":
            let writing_mode = get_writing_mode(context.builder.get_inheritedbox());
            context.builder.writing_mode = writing_mode;

            let mut _skip_font_family = false;

            % if product == "gecko":

                // <svg:text> is not affected by text zoom, and it uses a preshint to
                // disable it. We fix up the struct when this happens by unzooming
                // its contained font values, which will have been zoomed in the parent
                if seen.contains(LonghandId::XTextZoom) {
                    let zoom = context.builder.get_font().gecko().mAllowZoom;
                    let parent_zoom = context.style().get_parent_font().gecko().mAllowZoom;
                    if  zoom != parent_zoom {
                        debug_assert!(!zoom,
                                      "We only ever disable text zoom (in svg:text), never enable it");
                        // can't borrow both device and font, use the take/put machinery
                        let mut font = context.builder.take_font();
                        font.unzoom_fonts(context.device());
                        context.builder.put_font(font);
                    }
                }

                // Whenever a single generic value is specified, gecko will do a bunch of
                // recalculation walking up the rule tree, including handling the font-size stuff.
                // It basically repopulates the font struct with the default font for a given
                // generic and language. We handle the font-size stuff separately, so this boils
                // down to just copying over the font-family lists (no other aspect of the default
                // font can be configured).

                if seen.contains(LonghandId::XLang) || font_family.is_some() {
                    // if just the language changed, the inherited generic is all we need
                    let mut generic = inherited_style.get_font().gecko().mGenericID;
                    if let Some(ref declaration) = font_family {
                        if let PropertyDeclaration::FontFamily(ref fam) = **declaration {
                            if let Some(id) = fam.single_generic() {
                                generic = id;
                                // In case of a specified font family with a single generic, we will
                                // end up setting font family below, but its value would get
                                // overwritten later in the pipeline when cascading.
                                //
                                // We instead skip cascading font-family in that case.
                                //
                                // In case of the language changing, we wish for a specified font-
                                // family to override this, so we do not skip cascading then.
                                _skip_font_family = true;
                            }
                        }
                    }

                    let pres_context = context.builder.device.pres_context();
                    let gecko_font = context.builder.mutate_font().gecko_mut();
                    gecko_font.mGenericID = generic;
                    unsafe {
                        bindings::Gecko_nsStyleFont_PrefillDefaultForGeneric(
                            gecko_font,
                            pres_context,
                            generic,
                        );
                    }
                }
            % endif

            // It is important that font_size is computed before
            // the late properties (for em units), but after font-family
            // (for the base-font-size dependence for default and keyword font-sizes)
            // Additionally, when we support system fonts they will have to be
            // computed early, and *before* font_family, so I'm including
            // font_family here preemptively instead of keeping it within
            // the early properties.
            //
            // To avoid an extra iteration, we just pull out the property
            // during the early iteration and cascade them in order
            // after it.
            if !_skip_font_family {
                if let Some(ref declaration) = font_family {

                    let discriminant = LonghandId::FontFamily as usize;
                    (CASCADE_PROPERTY[discriminant])(declaration, &mut context);
                    % if product == "gecko":
                        let device = context.builder.device;
                        if let PropertyDeclaration::FontFamily(ref val) = **declaration {
                            if val.get_system().is_some() {
                                let default = context.cached_system_font
                                                     .as_ref().unwrap().default_font_type;
                                context.builder.mutate_font().fixup_system(default);
                            } else {
                                context.builder.mutate_font().fixup_none_generic(device);
                            }
                        }
                    % endif
                }
            }

            if let Some(ref declaration) = font_size {
                let discriminant = LonghandId::FontSize as usize;
                (CASCADE_PROPERTY[discriminant])(declaration, &mut context);
            % if product == "gecko":
            // Font size must be explicitly inherited to handle lang changes and
            // scriptlevel changes.
            } else if seen.contains(LonghandId::XLang) ||
                      seen.contains(LonghandId::MozScriptLevel) ||
                      seen.contains(LonghandId::MozMinFontSizeRatio) ||
                      font_family.is_some() {
                let discriminant = LonghandId::FontSize as usize;
                let size = PropertyDeclaration::CSSWideKeyword(
                    LonghandId::FontSize, CSSWideKeyword::Inherit);

                (CASCADE_PROPERTY[discriminant])(&size, &mut context);
            % endif
            }

            if let Some(style) = rule_cache.and_then(|c| c.find(&context.builder)) {
                context.builder.copy_reset_from(style);
                apply_reset = false;
            }
        % endif // category == "early"
    % endfor

    let mut builder = context.builder;

    % if product == "gecko":
        if let Some(ref mut bg) = builder.get_background_if_mutated() {
            bg.fill_arrays();
        }

        if let Some(ref mut svg) = builder.get_svg_if_mutated() {
            svg.fill_arrays();
        }
    % endif

    % if product == "servo":
        if seen.contains(LonghandId::FontStyle) ||
           seen.contains(LonghandId::FontWeight) ||
           seen.contains(LonghandId::FontStretch) ||
           seen.contains(LonghandId::FontFamily) {
            builder.mutate_font().compute_font_hash();
        }
    % endif

    builder.clear_modified_reset();

    StyleAdjuster::new(&mut builder)
        .adjust(layout_parent_style, flags);

    if builder.modified_reset() || !apply_reset {
        // If we adjusted any reset structs, we can't cache this ComputedValues.
        //
        // Also, if we re-used existing reset structs, don't bother caching it
        // back again. (Aside from being wasted effort, it will be wrong, since
        // context.rule_cache_conditions won't be set appropriately if we
        // didn't compute those reset properties.)
        context.rule_cache_conditions.borrow_mut()
            .set_uncacheable();
    }

    builder.build()
}

/// See StyleAdjuster::adjust_for_border_width.
pub fn adjust_border_width(style: &mut StyleBuilder) {
    % for side in ["top", "right", "bottom", "left"]:
        // Like calling to_computed_value, which wouldn't type check.
        if style.get_border().clone_border_${side}_style().none_or_hidden() &&
           style.get_border().border_${side}_has_nonzero_width() {
            style.set_border_${side}_width(NonNegativeLength::zero());
        }
    % endfor
}

/// An identifier for a given alias property.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum AliasId {
    % for i, property in enumerate(data.all_aliases()):
        /// ${property.name}
        ${property.camel_case} = ${i},
    % endfor
}

impl fmt::Debug for AliasId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            % for property in data.all_aliases():
                AliasId::${property.camel_case} => "${property.camel_case}",
            % endfor
        };
        formatter.write_str(name)
    }
}

impl AliasId {
    /// Returns an nsCSSPropertyID.
    #[cfg(feature = "gecko")]
    #[allow(non_upper_case_globals)]
    pub fn to_nscsspropertyid(&self) -> Result<nsCSSPropertyID, ()> {
        use gecko_bindings::structs::*;

        match *self {
            % for property in data.all_aliases():
                AliasId::${property.camel_case} => {
                    Ok(${helpers.alias_to_nscsspropertyid(property.ident)})
                },
            % endfor
        }
    }
}

#[macro_export]
macro_rules! css_properties_accessors {
    ($macro_name: ident) => {
        $macro_name! {
            % for kind, props in [("Longhand", data.longhands), ("Shorthand", data.shorthands)]:
                % for property in props:
                    % if not property.derived_from and not property.internal:
                        % for name in [property.name] + property.alias:
                            % if '-' in name:
                                [${to_rust_ident(name).capitalize()}, Set${to_rust_ident(name).capitalize()},
                                 PropertyId::${kind}(${kind}Id::${property.camel_case})],
                            % endif
                            [${to_camel_case(name)}, Set${to_camel_case(name)},
                             PropertyId::${kind}(${kind}Id::${property.camel_case})],
                        % endfor
                    % endif
                % endfor
            % endfor
        }
    }
}

#[macro_export]
macro_rules! longhand_properties_idents {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in data.longhands:
                { ${property.ident}, ${"true" if property.boxed else "false"} }
            % endfor
        }
    }
}
