/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

// Please note that valid Rust syntax may be mangled by the Mako parser.
// For example, Vec<&Foo> will be mangled as Vec&Foo>. To work around these issues, the code
// can be escaped. In the above example, Vec<<&Foo> or Vec< &Foo> achieves the desired result of Vec<&Foo>.

<%namespace name="helpers" file="/helpers.mako.rs" />

use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt;
use std::mem;
use std::ops::Deref;
use stylearc::{Arc, UniqueArc};

use app_units::Au;
#[cfg(feature = "servo")] use cssparser::RGBA;
use cssparser::{Parser, TokenSerializationType, serialize_identifier};
use cssparser::{ParserInput, CompactCowStr};
use error_reporting::ParseErrorReporter;
#[cfg(feature = "servo")] use euclid::SideOffsets2D;
use computed_values;
use context::QuirksMode;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")] use gecko_bindings::bindings;
#[cfg(feature = "gecko")] use gecko_bindings::structs::{self, nsCSSPropertyID};
#[cfg(feature = "servo")] use logical_geometry::{LogicalMargin, PhysicalSide};
use logical_geometry::WritingMode;
use media_queries::Device;
use parser::{Parse, ParserContext};
use properties::animated_properties::AnimatableLonghand;
#[cfg(feature = "gecko")] use properties::longhands::system_font::SystemFont;
use selectors::parser::SelectorParseError;
#[cfg(feature = "servo")] use servo_config::prefs::PREFS;
use shared_lock::StylesheetGuards;
use style_traits::{PARSING_MODE_DEFAULT, HasViewportPercentage, ToCss, ParseError};
use style_traits::{PropertyDeclarationParseError, StyleParseError};
use stylesheets::{CssRuleType, MallocSizeOf, MallocSizeOfFn, Origin, UrlExtraData};
#[cfg(feature = "servo")] use values::Either;
use values::generics::text::LineHeight;
use values::computed;
use cascade_info::CascadeInfo;
use rule_tree::{CascadeLevel, StrongRuleNode};
use self::computed_value_flags::ComputedValueFlags;
use style_adjuster::StyleAdjuster;
#[cfg(feature = "servo")] use values::specified::BorderStyle;

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


/// This is where we store extra font data while
/// while computing font sizes.
#[derive(Clone, Debug)]
pub struct FontComputationData {
    /// font-size keyword values (and font-size-relative values applied
    /// to keyword values) need to preserve their identity as originating
    /// from keywords and relative font sizes. We store this information
    /// out of band in the ComputedValues. When None, the font size on the
    /// current struct was computed from a value that was not a keyword
    /// or a chain of font-size-relative values applying to successive parents
    /// terminated by a keyword. When Some, this means the font-size was derived
    /// from a keyword value or a keyword value on some ancestor with only
    /// font-size-relative keywords and regular inheritance in between. The
    /// integer stores the final ratio of the chain of font size relative values.
    /// and is 1 when there was just a keyword and no relative values.
    ///
    /// When this is Some, we compute font sizes by computing the keyword against
    /// the generic font, and then multiplying it by the ratio.
   pub font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>
}


impl FontComputationData{
        /// Assigns values for variables in struct FontComputationData
    pub fn new(font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>) -> Self {
        FontComputationData {
            font_size_keyword: font_size_keyword
        }
    }
        /// Assigns default values for variables in struct FontComputationData
   pub fn default_values() -> Self {
        FontComputationData{
            font_size_keyword: Some((Default::default(), 1.))
        }
    }
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
                              [p.name for p in data.longhands if p.name not in ['direction', 'unicode-bidi']],
                              spec="https://drafts.csswg.org/css-cascade-3/#all-shorthand") %>
    pub mod all {
        use cssparser::Parser;
        use parser::ParserContext;
        use properties::{SourcePropertyDeclaration, AllShorthand, ShorthandId, UnparsedValue};
        use stylearc::Arc;
        use style_traits::{ParseError, StyleParseError};

        pub fn parse_into<'i, 't>(declarations: &mut SourcePropertyDeclaration,
                                  context: &ParserContext, input: &mut Parser<'i, 't>)
                                  -> Result<(), ParseError<'i>> {
            // This function is like the parse() that is generated by
            // helpers:shorthand, but since the only values for the 'all'
            // shorthand when not just a single CSS-wide keyword is one
            // with variable references, we can make this function a
            // little simpler.
            //
            // FIXME(heycam) Try to share code with the helpers:shorthand
            // definition.
            input.look_for_var_functions();
            let start = input.position();
            while let Ok(_) = input.next() {}  // Look for var()
            if input.seen_var_functions() {
                input.reset(start);
                let (first_token_type, css) =
                    ::custom_properties::parse_non_custom_with_var(input)?;
                declarations.all_shorthand = AllShorthand::WithVariables(Arc::new(UnparsedValue {
                    css: css.into_owned(),
                    first_token_type: first_token_type,
                    url_data: context.url_data.clone(),
                    from_shorthand: Some(ShorthandId::All),
                }));
                Ok(())
            } else {
                Err(StyleParseError::UnspecifiedError.into())
            }
        }
    }
}

/// A module with all the code related to animated properties.
///
/// This needs to be "included" by mako at least after all longhand modules,
/// given they populate the global data.
pub mod animated_properties {
    <%include file="/helpers/animated_properties.mako.rs" />
}

/// A set of longhand properties
#[derive(Clone, PartialEq)]
pub struct LonghandIdSet {
    storage: [u32; (${len(data.longhands)} - 1 + 32) / 32]
}

impl LonghandIdSet {
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

    /// Set the corresponding bit of AnimatableLonghand.
    pub fn set_animatable_longhand_bit(&mut self, property: &AnimatableLonghand) {
        match *property {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatableLonghand::${prop.camel_case} => self.insert(LonghandId::${prop.camel_case}),
                % endif
            % endfor
        }
    }

    /// Return true if the corresponding bit of AnimatableLonghand is set.
    pub fn has_animatable_longhand_bit(&self, property: &AnimatableLonghand) -> bool {
        match *property {
            % for prop in data.longhands:
                % if prop.animatable:
                    AnimatableLonghand::${prop.camel_case} => self.contains(LonghandId::${prop.camel_case}),
                % endif
            % endfor
        }
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

% for property in data.longhands:
    % if not property.derived_from:
        /// Perform CSS variable substitution if needed, and execute `f` with
        /// the resulting declared value.
        #[allow(non_snake_case)]
        fn substitute_variables_${property.ident}<F>(
            % if property.boxed:
                value: &DeclaredValue<Box<longhands::${property.ident}::SpecifiedValue>>,
            % else:
                value: &DeclaredValue<longhands::${property.ident}::SpecifiedValue>,
            % endif
            custom_properties: &Option<Arc<::custom_properties::CustomPropertiesMap>>,
            f: &mut F,
            error_reporter: &ParseErrorReporter,
            quirks_mode: QuirksMode)
            % if property.boxed:
                where F: FnMut(&DeclaredValue<Box<longhands::${property.ident}::SpecifiedValue>>)
            % else:
                where F: FnMut(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
            % endif
        {
            if let DeclaredValue::WithVariables(ref with_variables) = *value {
                substitute_variables_${property.ident}_slow(&with_variables.css,
                                                            with_variables.first_token_type,
                                                            &with_variables.url_data,
                                                            with_variables.from_shorthand,
                                                            custom_properties,
                                                            f,
                                                            error_reporter,
                                                            quirks_mode);
            } else {
                f(value);
            }
        }

        #[allow(non_snake_case)]
        #[inline(never)]
        fn substitute_variables_${property.ident}_slow<F>(
                css: &String,
                first_token_type: TokenSerializationType,
                url_data: &UrlExtraData,
                from_shorthand: Option<ShorthandId>,
                custom_properties: &Option<Arc<::custom_properties::CustomPropertiesMap>>,
                f: &mut F,
                error_reporter: &ParseErrorReporter,
                quirks_mode: QuirksMode)
                % if property.boxed:
                    where F: FnMut(&DeclaredValue<Box<longhands::${property.ident}::SpecifiedValue>>)
                % else:
                    where F: FnMut(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
                % endif
        {
            f(&
                ::custom_properties::substitute(css, first_token_type, custom_properties)
                .ok()
                .and_then(|css| {
                    // As of this writing, only the base URL is used for property values:
                    //
                    // FIXME(pcwalton): Cloning the error reporter is slow! But so are custom
                    // properties, so whatever...
                    let context = ParserContext::new(Origin::Author,
                                                     url_data,
                                                     error_reporter,
                                                     None,
                                                     PARSING_MODE_DEFAULT,
                                                     quirks_mode);
                    let mut input = ParserInput::new(&css);
                    Parser::new(&mut input).parse_entirely(|input| {
                        match from_shorthand {
                            None => {
                                longhands::${property.ident}
                                         ::parse_specified(&context, input).map(DeclaredValueOwned::Value)
                            }
                            Some(ShorthandId::All) => {
                                // No need to parse the 'all' shorthand as anything other than a CSS-wide
                                // keyword, after variable substitution.
                                Err(SelectorParseError::UnexpectedIdent("all".into()).into())
                            }
                            % for shorthand in data.shorthands_except_all():
                                % if property in shorthand.sub_properties:
                                    Some(ShorthandId::${shorthand.camel_case}) => {
                                        shorthands::${shorthand.ident}::parse_value(&context, input)
                                        .map(|result| {
                                            DeclaredValueOwned::Value(result.${property.ident})
                                        })
                                    }
                                % endif
                            % endfor
                            _ => unreachable!()
                        }
                    }).ok()
                })
                .unwrap_or(
                    // Invalid at computed-value time.
                    DeclaredValueOwned::CSSWideKeyword(
                        % if property.style_struct.inherited:
                            CSSWideKeyword::Inherit
                        % else:
                            CSSWideKeyword::Initial
                        % endif
                    )
                ).borrow()
            );
        }
    % endif
% endfor

/// An enum to represent a CSS Wide keyword.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, ToCss)]
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

impl Parse for CSSWideKeyword {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let ident = input.expect_ident()?;
        input.expect_exhausted()?;
        CSSWideKeyword::from_ident(&ident)
            .ok_or(SelectorParseError::UnexpectedIdent(ident).into())
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
    }
}

/// An identifier for a given longhand property.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LonghandId {
    % for i, property in enumerate(data.longhands):
        /// ${property.name}
        ${property.camel_case} = ${i},
    % endfor
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

    fn shorthands(&self) -> &'static [ShorthandId] {
        // first generate longhand to shorthands lookup map
        //
        // NOTE(emilio): This currently doesn't exclude the "all" shorthand. It
        // could potentially do so, which would speed up serialization
        // algorithms and what not, I guess.
        <%
            longhand_to_shorthand_map = {}
            for shorthand in data.shorthands:
                for sub_property in shorthand.sub_properties:
                    if sub_property.ident not in longhand_to_shorthand_map:
                        longhand_to_shorthand_map[sub_property.ident] = []

                    longhand_to_shorthand_map[sub_property.ident].append(shorthand.camel_case)

            for shorthand_list in longhand_to_shorthand_map.itervalues():
                shorthand_list.sort()
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
}

/// An identifier for a given shorthand property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToCss)]
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
}

/// Servo's representation of a declared value for a given `T`, which is the
/// declared value for that property.
#[derive(Clone, PartialEq, Eq, Debug)]
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DeclaredValueOwned<T> {
    /// A known specified value from the stylesheet.
    Value(T),
    /// An unparsed value that contains `var()` functions.
    WithVariables(Arc<UnparsedValue>),
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
#[derive(PartialEq, Eq, Debug)]
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

impl<'a, T: HasViewportPercentage> HasViewportPercentage for DeclaredValue<'a, T> {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            DeclaredValue::Value(ref v) => v.has_viewport_percentage(),
            DeclaredValue::WithVariables(_) => {
                panic!("DeclaredValue::has_viewport_percentage without \
                        resolving variables!")
            },
            DeclaredValue::CSSWideKeyword(_) => false,
        }
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
#[derive(PartialEq, Clone, Copy)]
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
#[derive(Eq, PartialEq, Clone)]
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
    pub fn parse<'i>(property_name: CompactCowStr<'i>) -> Result<Self, ParseError<'i>> {
        if let Ok(name) = ::custom_properties::parse_name(&property_name) {
            return Ok(PropertyId::Custom(::custom_properties::Name::from(name)))
        }

        // FIXME(https://github.com/rust-lang/rust/issues/33156): remove this enum and use PropertyId
        // when stable Rust allows destructors in statics.
        pub enum StaticId {
            Longhand(LonghandId),
            Shorthand(ShorthandId),
        }
        ascii_case_insensitive_phf_map! {
            static_id -> StaticId = {
                % for (kind, properties) in [("Longhand", data.longhands), ("Shorthand", data.shorthands)]:
                    % for property in properties:
                        % for name in [property.name] + property.alias:
                            "${name}" => StaticId::${kind}(${kind}Id::${property.camel_case}),
                        % endfor
                    % endfor
                % endfor
            }
        }
        match static_id(&property_name) {
            Some(&StaticId::Longhand(id)) => Ok(PropertyId::Longhand(id)),
            Some(&StaticId::Shorthand(id)) => Ok(PropertyId::Shorthand(id)),
            None => Err(StyleParseError::UnknownProperty(property_name).into()),
        }
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
        use gecko_bindings::structs::*;

        match *self {
            PropertyId::Longhand(id) => match id {
                % for property in data.longhands:
                    LonghandId::${property.camel_case} => {
                        Ok(${helpers.to_nscsspropertyid(property.ident)})
                    }
                % endfor
            },
            PropertyId::Shorthand(id) => match id {
                % for property in data.shorthands:
                    ShorthandId::${property.camel_case} => {
                        Ok(${helpers.to_nscsspropertyid(property.ident)})
                    }
                % endfor
            },
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
}

/// Servo's representation for a property declaration.
#[derive(PartialEq, Clone)]
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
    WithVariables(LonghandId, Arc<UnparsedValue>),
    /// A custom property declaration, with the property name and the declared
    /// value.
    Custom(::custom_properties::Name, DeclaredValueOwned<Box<::custom_properties::SpecifiedValue>>),
}

impl HasViewportPercentage for PropertyDeclaration {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(ref val) => {
                    val.has_viewport_percentage()
                },
            % endfor
            PropertyDeclaration::WithVariables(..) => {
                panic!("DeclaredValue::has_viewport_percentage without \
                        resolving variables!")
            },
            PropertyDeclaration::CSSWideKeyword(..) => false,
            PropertyDeclaration::Custom(_, ref val) => {
                val.borrow().has_viewport_percentage()
            }
        }
    }
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

<%def name="property_exposure_check(property)">
    // For properties that are experimental but not internal, the pref will
    // control its availability in all sheets.   For properties that are
    // both experimental and internal, the pref only controls its
    // availability in non-UA sheets (and in UA sheets it is always available).
    let is_experimental =
        % if property.experimental and product == "servo":
            true;
        % elif product == "gecko":
            structs::root::mozilla::SERVO_PREF_ENABLED_${property.gecko_pref_ident};
        % else:
            false;
        % endif

    let passes_pref_check =
        % if property.experimental and product == "servo":
            PREFS.get("${property.experimental}").as_boolean().unwrap_or(false);
        % elif product == "gecko":
            {
                let id = structs::${helpers.to_nscsspropertyid(property.ident)};
                unsafe { bindings::Gecko_PropertyId_IsPrefEnabled(id) }
            };
        % else:
            true;
        % endif

    % if property.internal:
        if context.stylesheet_origin != Origin::UserAgent {
            if is_experimental {
                if !passes_pref_check {
                    return Err(PropertyDeclarationParseError::ExperimentalProperty);
                }
            } else {
                return Err(PropertyDeclarationParseError::UnknownProperty);
            }
        }
    % else:
        if is_experimental && !passes_pref_check {
            return Err(PropertyDeclarationParseError::ExperimentalProperty);
        }
    % endif
</%def>

impl MallocSizeOf for PropertyDeclaration {
    fn malloc_size_of_children(&self, _malloc_size_of: MallocSizeOfFn) -> usize {
        // The variants of PropertyDeclaration mostly (entirely?) contain
        // scalars, so this is reasonable.
        0
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

    /// Returns true if this property is one of the animable properties, false
    /// otherwise.
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
    pub fn parse_into(declarations: &mut SourcePropertyDeclaration,
                      id: PropertyId, context: &ParserContext, input: &mut Parser)
                      -> Result<(), PropertyDeclarationParseError> {
        assert!(declarations.is_empty());
        let rule_type = context.rule_type();
        debug_assert!(rule_type == CssRuleType::Keyframe ||
                      rule_type == CssRuleType::Page ||
                      rule_type == CssRuleType::Style,
                      "Declarations are only expected inside a keyframe, page, or style rule.");
        match id {
            PropertyId::Custom(name) => {
                let value = match input.try(|i| CSSWideKeyword::parse(context, i)) {
                    Ok(keyword) => DeclaredValueOwned::CSSWideKeyword(keyword),
                    Err(_) => match ::custom_properties::SpecifiedValue::parse(context, input) {
                        Ok(value) => DeclaredValueOwned::Value(value),
                        Err(_) => return Err(PropertyDeclarationParseError::InvalidValue(name.to_string())),
                    }
                };
                declarations.push(PropertyDeclaration::Custom(name, value));
                Ok(())
            }
            PropertyId::Longhand(id) => match id {
            % for property in data.longhands:
                LonghandId::${property.camel_case} => {
                    % if not property.derived_from:
                        % if not property.allowed_in_keyframe_block:
                            if rule_type == CssRuleType::Keyframe {
                                return Err(PropertyDeclarationParseError::AnimationPropertyInKeyframeBlock)
                            }
                        % endif
                        % if not property.allowed_in_page_rule:
                            if rule_type == CssRuleType::Page {
                                return Err(PropertyDeclarationParseError::NotAllowedInPageRule)
                            }
                        % endif

                        ${property_exposure_check(property)}

                        match longhands::${property.ident}::parse_declared(context, input) {
                            Ok(value) => {
                                declarations.push(value);
                                Ok(())
                            },
                            Err(_) => Err(PropertyDeclarationParseError::InvalidValue("${property.ident}".into())),
                        }
                    % else:
                        Err(PropertyDeclarationParseError::UnknownProperty)
                    % endif
                }
            % endfor
            },
            PropertyId::Shorthand(id) => match id {
            % for shorthand in data.shorthands:
                ShorthandId::${shorthand.camel_case} => {
                    % if not shorthand.allowed_in_keyframe_block:
                        if rule_type == CssRuleType::Keyframe {
                            return Err(PropertyDeclarationParseError::AnimationPropertyInKeyframeBlock)
                        }
                    % endif
                    % if not shorthand.allowed_in_page_rule:
                        if rule_type == CssRuleType::Page {
                            return Err(PropertyDeclarationParseError::NotAllowedInPageRule)
                        }
                    % endif

                    ${property_exposure_check(shorthand)}

                    match input.try(|i| CSSWideKeyword::parse(context, i)) {
                        Ok(keyword) => {
                            % if shorthand.name == "all":
                                declarations.all_shorthand = AllShorthand::CSSWideKeyword(keyword);
                            % else:
                                % for sub_property in shorthand.sub_properties:
                                    declarations.push(PropertyDeclaration::CSSWideKeyword(
                                        LonghandId::${sub_property.camel_case},
                                        keyword,
                                    ));
                                % endfor
                            % endif
                            Ok(())
                        },
                        Err(_) => {
                            shorthands::${shorthand.ident}::parse_into(declarations, context, input)
                                .map_err(|_| PropertyDeclarationParseError::InvalidValue("${shorthand.ident}".into()))
                        }
                    }
                }
            % endfor
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
    use app_units::Au;
    use fnv::FnvHasher;
    use super::longhands;
    use std::hash::{Hash, Hasher};
    use logical_geometry::WritingMode;
    use media_queries::Device;

    % for style_struct in data.active_style_structs():
        % if style_struct.name == "Font":
        #[derive(Clone, Debug)]
        % else:
        #[derive(PartialEq, Clone, Debug)]
        % endif
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
                    % if longhand.need_clone:
                        /// Get the computed value for ${longhand.name}.
                        #[allow(non_snake_case)]
                        #[inline]
                        pub fn clone_${longhand.ident}(&self) -> longhands::${longhand.ident}::computed_value::T {
                            self.${longhand.ident}.clone()
                        }
                    % endif
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
                        self.border_${side}_width != ::app_units::Au(0)
                    }
                % endfor
            % elif style_struct.name == "Font":
                /// Computes a font hash in order to be able to cache fonts
                /// effectively in GFX and layout.
                pub fn compute_font_hash(&mut self) {
                    // Corresponds to the fields in
                    // `gfx::font_template::FontTemplateDescriptor`.
                    let mut hasher: FnvHasher = Default::default();
                    hasher.write_u16(self.font_weight as u16);
                    self.font_stretch.hash(&mut hasher);
                    self.font_family.hash(&mut hasher);
                    self.hash = hasher.finish()
                }

                /// (Servo does not handle MathML, so this just calls copy_font_size_from)
                pub fn inherit_font_size_from(&mut self, parent: &Self,
                                              _: Option<Au>, _: &Device) -> bool {
                    self.copy_font_size_from(parent);
                    false
                }
                /// (Servo does not handle MathML, so this just calls set_font_size)
                pub fn apply_font_size(&mut self,
                                       v: longhands::font_size::computed_value::T,
                                       _: &Self,
                                       _: &Device) -> Option<Au> {
                    self.set_font_size(v);
                    None
                }
                /// (Servo does not handle MathML, so this does nothing)
                pub fn apply_unconstrained_font_size(&mut self, _: Au) {
                }

            % elif style_struct.name == "Outline":
                /// Whether the outline-width property is non-zero.
                #[inline]
                pub fn outline_has_nonzero_width(&self) -> bool {
                    self.outline_width != ::app_units::Au(0)
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
pub use gecko_properties::ComputedValues;

/// A legacy alias for a servo-version of ComputedValues. Should go away soon.
#[cfg(feature = "servo")]
pub type ServoComputedValues = ComputedValues;

/// The struct that Servo uses to represent computed values.
///
/// This struct contains an immutable atomically-reference-counted pointer to
/// every kind of style struct.
///
/// When needed, the structs may be copied in order to get mutated.
#[cfg(feature = "servo")]
#[cfg_attr(feature = "servo", derive(Clone))]
pub struct ComputedValues {
    % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor
    custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
    /// The writing mode of this computed values struct.
    pub writing_mode: WritingMode,
    /// The keyword behind the current font-size property, if any
    pub font_computation_data: FontComputationData,

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

#[cfg(feature = "servo")]
impl ComputedValues {
    /// Construct a `ComputedValues` instance.
    pub fn new(
        custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
        writing_mode: WritingMode,
        font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>,
        flags: ComputedValueFlags,
        rules: Option<StrongRuleNode>,
        visited_style: Option<Arc<ComputedValues>>,
        % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
        % endfor
    ) -> Self {
        let font_computation_data = FontComputationData::new(font_size_keyword);
        ComputedValues {
            custom_properties,
            writing_mode,
            font_computation_data,
            flags,
            rules,
            visited_style,
        % for style_struct in data.active_style_structs():
            ${style_struct.ident},
        % endfor
        }
    }

    /// Get the initial computed values.
    pub fn initial_values() -> &'static Self { &*INITIAL_SERVO_VALUES }

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

    /// Gets a reference to the visited style, if any.
    pub fn get_visited_style(&self) -> Option<<&Arc<ComputedValues>> {
        self.visited_style.as_ref()
    }

    /// Gets a reference to the visited style. Panic if no visited style exists.
    pub fn visited_style(&self) -> &Arc<ComputedValues> {
        self.get_visited_style().unwrap()
    }

    /// Clone the visited style.  Used for inheriting parent styles in
    /// StyleBuilder::for_inheritance.
    pub fn clone_visited_style(&self) -> Option<Arc<ComputedValues>> {
        self.visited_style.clone()
    }

    // Aah! The << in the return type below is not valid syntax, but we must
    // escape < that way for Mako.
    /// Gets a reference to the custom properties map (if one exists).
    pub fn get_custom_properties(&self) -> Option<<&::custom_properties::CustomPropertiesMap> {
        self.custom_properties.as_ref().map(|x| &**x)
    }

    /// Get the custom properties map if necessary.
    ///
    /// Cloning the Arc here is fine because it only happens in the case where
    /// we have custom properties, and those are both rare and expensive.
    pub fn custom_properties(&self) -> Option<Arc<::custom_properties::CustomPropertiesMap>> {
        self.custom_properties.clone()
    }

    /// Whether this style has a -moz-binding value. This is always false for
    /// Servo for obvious reasons.
    pub fn has_moz_binding(&self) -> bool { false }

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
            padding_style.padding_top,
            padding_style.padding_right,
            padding_style.padding_bottom,
            padding_style.padding_left,
        ))
    }

    /// Get the logical border width
    #[inline]
    pub fn border_width_for_writing_mode(&self, writing_mode: WritingMode) -> LogicalMargin<Au> {
        let border_style = self.get_border();
        LogicalMargin::from_physical(writing_mode, SideOffsets2D::new(
            border_style.border_top_width,
            border_style.border_right_width,
            border_style.border_bottom_width,
            border_style.border_left_width,
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
                        if z != Au(0) {
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
                    .and_then(|map| map.get_computed_value(name))
                    .map(|value| value.to_css_string())
                    .unwrap_or(String::new())
            }
        }
    }
}

// We manually implement Debug for ComputedValues so that we can avoid the
// verbose stringification of every property and instead focus on a few values.
impl fmt::Debug for ComputedValues {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComputedValues {{ rules: {:?}, .. }}", self.rules)
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

/// A reference to a style struct of the parent, or our own style struct.
pub enum StyleStructRef<'a, T: 'static> {
    /// A borrowed struct from the parent, for example, for inheriting style.
    Borrowed(&'a Arc<T>),
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
            StyleStructRef::Borrowed(v) => v.clone(),
            StyleStructRef::Vacated => panic!("Accessed vacated style struct")
        }
    }
}

impl<'a, T: 'a> Deref for StyleStructRef<'a, T> {
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
    /// The style we're inheriting from.
    inherited_style: &'a ComputedValues,
    /// The rule node representing the ordered list of rules matched for this
    /// node.
    rules: Option<StrongRuleNode>,
    custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
    /// The writing mode flags.
    ///
    /// TODO(emilio): Make private.
    pub writing_mode: WritingMode,
    /// The keyword behind the current font-size property, if any.
    pub font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>,
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
        inherited_style: &'a ComputedValues,
        reset_style: &'a ComputedValues,
        rules: Option<StrongRuleNode>,
        custom_properties: Option<Arc<::custom_properties::CustomPropertiesMap>>,
        writing_mode: WritingMode,
        font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>,
        visited_style: Option<Arc<ComputedValues>>,
    ) -> Self {
        StyleBuilder {
            inherited_style,
            rules,
            custom_properties,
            writing_mode,
            font_size_keyword,
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

    /// Creates a StyleBuilder holding only references to the structs of `s`, in
    /// order to create a derived style.
    pub fn for_derived_style(s: &'a ComputedValues) -> Self {
        Self::for_inheritance(s, s)
    }

    /// Inherits style from the parent element, accounting for the default
    /// computed values that need to be provided as well.
    pub fn for_inheritance(
        parent: &'a ComputedValues,
        default: &'a ComputedValues
    ) -> Self {
        Self::new(
            parent,
            default,
            /* rules = */ None,
            parent.custom_properties(),
            parent.writing_mode,
            parent.font_computation_data.font_size_keyword,
            parent.clone_visited_style()
        )
    }


    % for style_struct in data.active_style_structs():
        /// Gets an immutable view of the current `${style_struct.name}` style.
        pub fn get_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
            &self.${style_struct.ident}
        }

        /// Gets a mutable view of the current `${style_struct.name}` style.
        pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
            self.${style_struct.ident}.mutate()
        }

        /// Gets a mutable view of the current `${style_struct.name}` style.
        pub fn take_${style_struct.name_lower}(&mut self) -> UniqueArc<style_structs::${style_struct.name}> {
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


    /// Turns this `StyleBuilder` into a proper `ComputedValues` instance.
    pub fn build(self) -> ComputedValues {
        let flags = ComputedValueFlags::compute(&self, &self.inherited_style);
        ComputedValues::new(self.custom_properties,
                            self.writing_mode,
                            self.font_size_keyword,
                            flags,
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
    fn custom_properties(&self) -> Option<Arc<::custom_properties::CustomPropertiesMap>> {
        self.custom_properties.clone()
    }
}

#[cfg(feature = "servo")]
pub use self::lazy_static_module::INITIAL_SERVO_VALUES;

// Use a module to work around #[cfg] on lazy_static! not being applied to every generated item.
#[cfg(feature = "servo")]
#[allow(missing_docs)]
mod lazy_static_module {
    use logical_geometry::WritingMode;
    use stylearc::Arc;
    use super::{ComputedValues, longhands, style_structs, FontComputationData};
    use super::computed_value_flags::ComputedValueFlags;

    /// The initial values for all style structs as defined by the specification.
    lazy_static! {
        pub static ref INITIAL_SERVO_VALUES: ComputedValues = ComputedValues {
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
            flags: ComputedValueFlags::initial(),
            font_computation_data: FontComputationData::default_values(),
            rules: None,
            visited_style: None,
        };
    }
}

/// A per-longhand function that performs the CSS cascade for that longhand.
pub type CascadePropertyFn =
    extern "Rust" fn(declaration: &PropertyDeclaration,
                     inherited_style: &ComputedValues,
                     default_style: &ComputedValues,
                     context: &mut computed::Context,
                     cacheable: &mut bool,
                     cascade_info: &mut Option<<&mut CascadeInfo>,
                     error_reporter: &ParseErrorReporter);

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
        const INHERIT_ALL = 0x01,

        /// Whether to skip any root element and flex/grid item display style
        /// fixup.
        const SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP = 0x02,

        /// Whether to only cascade properties that are visited dependent.
        const VISITED_DEPENDENT_ONLY = 0x04,

        /// Whether the given element we're styling is the document element,
        /// that is, matches :root.
        ///
        /// Not set for native anonymous content since some NAC form their own
        /// root, but share the device.
        ///
        /// This affects some style adjustments, like blockification, and means
        /// that it may affect global state, like the Device's root font-size.
        const IS_ROOT_ELEMENT = 0x08,

        /// Whether to convert display:contents into display:inline.  This
        /// is used by Gecko to prevent display:contents on generated
        /// content.
        const PROHIBIT_DISPLAY_CONTENTS = 0x10,
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
pub fn cascade(device: &Device,
               rule_node: &StrongRuleNode,
               guards: &StylesheetGuards,
               parent_style: Option<<&ComputedValues>,
               layout_parent_style: Option<<&ComputedValues>,
               visited_style: Option<Arc<ComputedValues>>,
               cascade_info: Option<<&mut CascadeInfo>,
               error_reporter: &ParseErrorReporter,
               font_metrics_provider: &FontMetricsProvider,
               flags: CascadeFlags,
               quirks_mode: QuirksMode)
               -> ComputedValues {
    debug_assert_eq!(parent_style.is_some(), layout_parent_style.is_some());
    let (inherited_style, layout_parent_style) = match parent_style {
        Some(parent_style) => {
            (parent_style,
             layout_parent_style.unwrap())
        },
        None => {
            (device.default_computed_values(),
             device.default_computed_values())
        }
    };

    let iter_declarations = || {
        rule_node.self_and_ancestors().flat_map(|node| {
            let cascade_level = node.cascade_level();
            let source = node.style_source();
            let declarations = if source.is_some() {
                source.read(cascade_level.guard(guards)).declarations()
            } else {
                // The root node has no style source.
                &[]
            };
            let node_importance = node.importance();
            declarations
                .iter()
                // Yield declarations later in source order (with more precedence) first.
                .rev()
                .filter_map(move |&(ref declaration, declaration_importance)| {
                    if declaration_importance == node_importance {
                        Some((declaration, cascade_level))
                    } else {
                        None
                    }
                })
        })
    };
    apply_declarations(device,
                       rule_node,
                       iter_declarations,
                       inherited_style,
                       layout_parent_style,
                       visited_style,
                       cascade_info,
                       error_reporter,
                       font_metrics_provider,
                       flags,
                       quirks_mode)
}

/// NOTE: This function expects the declaration with more priority to appear
/// first.
#[allow(unused_mut)] // conditionally compiled code for "position"
pub fn apply_declarations<'a, F, I>(device: &Device,
                                    rules: &StrongRuleNode,
                                    iter_declarations: F,
                                    inherited_style: &ComputedValues,
                                    layout_parent_style: &ComputedValues,
                                    visited_style: Option<Arc<ComputedValues>>,
                                    mut cascade_info: Option<<&mut CascadeInfo>,
                                    error_reporter: &ParseErrorReporter,
                                    font_metrics_provider: &FontMetricsProvider,
                                    flags: CascadeFlags,
                                    quirks_mode: QuirksMode)
                                    -> ComputedValues
    where F: Fn() -> I,
          I: Iterator<Item = (&'a PropertyDeclaration, CascadeLevel)>,
{
    let default_style = device.default_computed_values();
    let inherited_custom_properties = inherited_style.custom_properties();
    let mut custom_properties = None;
    let mut seen_custom = HashSet::new();
    for (declaration, _cascade_level) in iter_declarations() {
        if let PropertyDeclaration::Custom(ref name, ref value) = *declaration {
            ::custom_properties::cascade(
                &mut custom_properties, &inherited_custom_properties,
                &mut seen_custom, name, value.borrow());
        }
    }

    let custom_properties =
        ::custom_properties::finish_cascade(
            custom_properties, &inherited_custom_properties);

    let reset_style = if flags.contains(INHERIT_ALL) {
        inherited_style
    } else {
        default_style
    };

    let mut context = computed::Context {
        is_root_element: flags.contains(IS_ROOT_ELEMENT),
        device: device,
        inherited_style: inherited_style,
        layout_parent_style: layout_parent_style,
        // We'd really like to own the rules here to avoid refcount traffic, but
        // animation's usage of `apply_declarations` make this tricky. See bug
        // 1375525.
        style: StyleBuilder::new(
            inherited_style,
            reset_style,
            Some(rules.clone()),
            custom_properties,
            WritingMode::empty(),
            inherited_style.font_computation_data.font_size_keyword,
            visited_style,
        ),
        font_metrics_provider: font_metrics_provider,
        cached_system_font: None,
        in_media_query: false,
        quirks_mode: quirks_mode,
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
    //
    // NB: The cacheable boolean is not used right now, but will be once we
    // start caching computed values in the rule nodes.
    let mut cacheable = true;
    let mut seen = LonghandIdSet::new();

    // Declaration blocks are stored in increasing precedence order, we want
    // them in decreasing order here.
    //
    // We could (and used to) use a pattern match here, but that bloats this
    // function to over 100K of compiled code!
    //
    // To improve i-cache behavior, we outline the individual functions and use
    // virtual dispatch instead.
    % for category_to_cascade_now in ["early", "other"]:
        % if category_to_cascade_now == "early":
            // Pull these out so that we can compute them in a specific order
            // without introducing more iterations.
            let mut font_size = None;
            let mut font_family = None;
        % endif
        for (declaration, cascade_level) in iter_declarations() {
            let mut declaration = declaration;
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
                if let PropertyDeclaration::BackgroundColor(ref color) = *declaration {
                    // Treat background-color a bit differently.  If the specified
                    // color is anything other than a fully transparent color, convert
                    // it into the Device's default background color.
                    if color.is_non_transparent() {
                        declaration = default_background_color_decl.as_ref().unwrap();
                    }
                } else {
                    continue
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
                    font_size = Some(declaration);
                    continue;
                }
                if LonghandId::FontFamily == longhand_id {
                    font_family = Some(declaration);
                    continue;
                }
            % endif

            let discriminant = longhand_id as usize;
            (CASCADE_PROPERTY[discriminant])(declaration,
                                             inherited_style,
                                             default_style,
                                             &mut context,
                                             &mut cacheable,
                                             &mut cascade_info,
                                             error_reporter);
        }
        % if category_to_cascade_now == "early":
            let writing_mode = get_writing_mode(context.style.get_inheritedbox());
            context.style.writing_mode = writing_mode;

            let mut _skip_font_family = false;

            % if product == "gecko":
                // Whenever a single generic value is specified, gecko will do a bunch of
                // recalculation walking up the rule tree, including handling the font-size stuff.
                // It basically repopulates the font struct with the default font for a given
                // generic and language. We handle the font-size stuff separately, so this boils
                // down to just copying over the font-family lists (no other aspect of the default
                // font can be configured).

                if seen.contains(LonghandId::XLang) || font_family.is_some() {
                    // if just the language changed, the inherited generic is all we need
                    let mut generic = inherited_style.get_font().gecko().mGenericID;
                    if let Some(declaration) = font_family {
                        if let PropertyDeclaration::FontFamily(ref fam) = *declaration {
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

                    // In case of just the language changing, the parent could have had no generic,
                    // which Gecko just does regular cascading with. Do the same.
                    // This can only happen in the case where the language changed but the family did not
                    if generic != structs::kGenericFont_NONE {
                        let gecko_font = context.style.mutate_font().gecko_mut();
                        gecko_font.mGenericID = generic;
                        unsafe {
                            bindings::Gecko_nsStyleFont_PrefillDefaultForGeneric(
                                gecko_font,
                                context.device.pres_context(),
                                generic
                            );
                        }
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
                if let Some(declaration) = font_family {

                    let discriminant = LonghandId::FontFamily as usize;
                    (CASCADE_PROPERTY[discriminant])(declaration,
                                                     inherited_style,
                                                     default_style,
                                                     &mut context,
                                                     &mut cacheable,
                                                     &mut cascade_info,
                                                     error_reporter);
                    % if product == "gecko":
                        context.style.mutate_font().fixup_none_generic(context.device);
                    % endif
                }
            }

            if let Some(declaration) = font_size {
                let discriminant = LonghandId::FontSize as usize;
                (CASCADE_PROPERTY[discriminant])(declaration,
                                                 inherited_style,
                                                 default_style,
                                                 &mut context,
                                                 &mut cacheable,
                                                 &mut cascade_info,
                                                 error_reporter);
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

                (CASCADE_PROPERTY[discriminant])(&size,
                                                 inherited_style,
                                                 default_style,
                                                 &mut context,
                                                 &mut cacheable,
                                                 &mut cascade_info,
                                                 error_reporter);
            % endif
            }

            if context.is_root_element {
                let s = context.style.get_font().clone_font_size();
                context.device.set_root_font_size(s);
            }
        % endif
    % endfor

    let mut style = context.style;

    {
        StyleAdjuster::new(&mut style)
            .adjust(context.layout_parent_style, flags);
    }

    % if product == "gecko":
        if let Some(ref mut bg) = style.get_background_if_mutated() {
            bg.fill_arrays();
        }

        if let Some(ref mut svg) = style.get_svg_if_mutated() {
            svg.fill_arrays();
        }
    % endif

    % if product == "servo":
        if seen.contains(LonghandId::FontStyle) ||
           seen.contains(LonghandId::FontWeight) ||
           seen.contains(LonghandId::FontStretch) ||
           seen.contains(LonghandId::FontFamily) {
            style.mutate_font().compute_font_hash();
        }
    % endif

    style.build()
}


/// See StyleAdjuster::adjust_for_border_width.
pub fn adjust_border_width(style: &mut StyleBuilder) {
    % for side in ["top", "right", "bottom", "left"]:
        // Like calling to_computed_value, which wouldn't type check.
        if style.get_border().clone_border_${side}_style().none_or_hidden() &&
           style.get_border().border_${side}_has_nonzero_width() {
            style.mutate_border().set_border_${side}_width(Au(0));
        }
    % endfor
}

/// Adjusts borders as appropriate to account for a fragment's status as the
/// first or last fragment within the range of an element.
///
/// Specifically, this function sets border widths to zero on the sides for
/// which the fragment is not outermost.
#[cfg(feature = "servo")]
#[inline]
pub fn modify_border_style_for_inline_sides(style: &mut Arc<ComputedValues>,
                                            is_first_fragment_of_element: bool,
                                            is_last_fragment_of_element: bool) {
    fn modify_side(style: &mut Arc<ComputedValues>, side: PhysicalSide) {
        {
            let border = &style.border;
            let current_style = match side {
                PhysicalSide::Left =>   (border.border_left_width,   border.border_left_style),
                PhysicalSide::Right =>  (border.border_right_width,  border.border_right_style),
                PhysicalSide::Top =>    (border.border_top_width,    border.border_top_style),
                PhysicalSide::Bottom => (border.border_bottom_width, border.border_bottom_style),
            };
            if current_style == (Au(0), BorderStyle::none) {
                return;
            }
        }
        let mut style = Arc::make_mut(style);
        let border = Arc::make_mut(&mut style.border);
        match side {
            PhysicalSide::Left => {
                border.border_left_width = Au(0);
                border.border_left_style = BorderStyle::none;
            }
            PhysicalSide::Right => {
                border.border_right_width = Au(0);
                border.border_right_style = BorderStyle::none;
            }
            PhysicalSide::Bottom => {
                border.border_bottom_width = Au(0);
                border.border_bottom_style = BorderStyle::none;
            }
            PhysicalSide::Top => {
                border.border_top_width = Au(0);
                border.border_top_style = BorderStyle::none;
            }
        }
    }

    if !is_first_fragment_of_element {
        let side = style.writing_mode.inline_start_physical_side();
        modify_side(style, side)
    }

    if !is_last_fragment_of_element {
        let side = style.writing_mode.inline_end_physical_side();
        modify_side(style, side)
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


macro_rules! longhand_properties_idents {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in data.longhands:
                ${property.ident}
            % endfor
        }
    }
}

/// Testing function to check the size of all SpecifiedValues.
#[cfg(feature = "testing")]
pub fn test_size_of_specified_values() {
    use std::mem::size_of;
    let threshold = 24;

    let mut longhands = vec![];
    % for property in data.longhands:
        longhands.push(("${property.name}",
                       size_of::<longhands::${property.ident}::SpecifiedValue>(),
                       ${"true" if property.boxed else "false"}));
    % endfor

    let mut failing_messages = vec![];

    for specified_value in longhands {
        if specified_value.1 > threshold && !specified_value.2 {
            failing_messages.push(
                format!("Your changes have increased the size of {} SpecifiedValue to {}. The threshold is \
                        currently {}. SpecifiedValues affect size of PropertyDeclaration enum and \
                        increasing the size may negative affect style system performance. Please consider \
                        using `boxed=\"True\"` in this longhand.",
                        specified_value.0, specified_value.1, threshold));
        } else if specified_value.1 <= threshold && specified_value.2 {
            failing_messages.push(
                format!("Your changes have decreased the size of {} SpecifiedValue to {}. Good work! \
                        The threshold is currently {}. Please consider removing `boxed=\"True\"` from this longhand.",
                        specified_value.0, specified_value.1, threshold));
        }
    }

    if !failing_messages.is_empty() {
        panic!("{}", failing_messages.join("\n\n"));
    }
}
