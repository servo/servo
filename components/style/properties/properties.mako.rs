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
use std::sync::Arc;

use app_units::Au;
#[cfg(feature = "servo")] use cssparser::{Color as CSSParserColor, RGBA};
use cssparser::{Parser, TokenSerializationType};
use error_reporting::ParseErrorReporter;
#[cfg(feature = "servo")] use euclid::side_offsets::SideOffsets2D;
use computed_values;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")] use gecko_bindings::bindings;
#[cfg(feature = "gecko")] use gecko_bindings::structs::{self, nsCSSPropertyID};
#[cfg(feature = "servo")] use logical_geometry::{LogicalMargin, PhysicalSide};
use logical_geometry::WritingMode;
use media_queries::Device;
use parser::{LengthParsingMode, Parse, ParserContext};
use properties::animated_properties::TransitionProperty;
#[cfg(feature = "servo")] use servo_config::prefs::PREFS;
use shared_lock::StylesheetGuards;
use style_traits::ToCss;
use stylesheets::{CssRuleType, Origin, UrlExtraData};
#[cfg(feature = "servo")] use values::Either;
use values::{HasViewportPercentage, computed};
use cascade_info::CascadeInfo;
use rule_tree::StrongRuleNode;
#[cfg(feature = "servo")] use values::specified::BorderStyle;

pub use self::declaration_block::*;

#[cfg(feature = "gecko")]
#[macro_export]
macro_rules! property_name {
    ($s: tt) => { atom!($s) }
}

<%!
    from data import Method, Keyword, to_rust_ident, to_camel_case
    import os.path
%>

#[path="${repr(os.path.join(os.path.dirname(__file__), 'declaration_block.rs'))[1:-1]}"]
pub mod declaration_block;

/// A module with all the code for longhand properties.
#[allow(missing_docs)]
pub mod longhands {
    use cssparser::Parser;
    use parser::{Parse, ParserContext};
    use values::specified;

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
    use values::specified;

    /// Parses a property for four different sides per CSS syntax.
    ///
    ///  * Zero or more than four values is invalid.
    ///  * One value sets them all
    ///  * Two values set (top, bottom) and (left, right)
    ///  * Three values set top, (left, right) and bottom
    ///  * Four values set them in order
    ///
    /// returns the values in (top, right, bottom, left) order.
    pub fn parse_four_sides<F, T>(input: &mut Parser, parse_one: F) -> Result<(T, T, T, T), ()>
        where F: Fn(&mut Parser) -> Result<T, ()>,
              T: Clone,
    {
        let top = try!(parse_one(input));
        let right;
        let bottom;
        let left;
        match input.try(|i| parse_one(i)) {
            Err(()) => {
                right = top.clone();
                bottom = top.clone();
                left = top.clone();
            }
            Ok(value) => {
                right = value;
                match input.try(|i| parse_one(i)) {
                    Err(()) => {
                        bottom = top.clone();
                        left = right.clone();
                    }
                    Ok(value) => {
                        bottom = value;
                        match input.try(|i| parse_one(i)) {
                            Err(()) => {
                                left = right.clone();
                            }
                            Ok(value) => {
                                left = value;
                            }
                        }

                    }
                }

            }
        }
        Ok((top, right, bottom, left))
    }

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
        use properties::{ParsedDeclaration, ShorthandId, UnparsedValue};
        use std::sync::Arc;

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<ParsedDeclaration, ()> {
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
                let (first_token_type, css) = try!(
                    ::custom_properties::parse_non_custom_with_var(input));
                Ok(ParsedDeclaration::AllWithVariables(Arc::new(UnparsedValue {
                    css: css.into_owned(),
                    first_token_type: first_token_type,
                    url_data: context.url_data.clone(),
                    from_shorthand: Some(ShorthandId::All),
                })))
            } else {
                Err(())
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
#[derive(Clone)]
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

    /// Set the corresponding bit of TransitionProperty.
    /// This function will panic if TransitionProperty::All is given.
    pub fn set_transition_property_bit(&mut self, property: &TransitionProperty) {
        match *property {
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => self.insert(LonghandId::${prop.camel_case}),
                % endif
            % endfor
            other => unreachable!("Tried to set TransitionProperty::{:?} in a PropertyBitfield", other),
        }
    }

    /// Return true if the corresponding bit of TransitionProperty is set.
    /// This function will panic if TransitionProperty::All is given.
    pub fn has_transition_property_bit(&self, property: &TransitionProperty) -> bool {
        match *property {
            % for prop in data.longhands:
                % if prop.animatable:
                    TransitionProperty::${prop.camel_case} => self.contains(LonghandId::${prop.camel_case}),
                % endif
            % endfor
            other => unreachable!("Tried to get TransitionProperty::{:?} in a PropertyBitfield", other),
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
            custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
            f: F,
            error_reporter: &ParseErrorReporter)
            % if property.boxed:
                where F: FnOnce(&DeclaredValue<Box<longhands::${property.ident}::SpecifiedValue>>)
            % else:
                where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
            % endif
        {
            if let DeclaredValue::WithVariables(ref with_variables) = *value {
                substitute_variables_${property.ident}_slow(&with_variables.css,
                                                            with_variables.first_token_type,
                                                            &with_variables.url_data,
                                                            with_variables.from_shorthand,
                                                            custom_properties,
                                                            f,
                                                            error_reporter);
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
                custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
                f: F,
                error_reporter: &ParseErrorReporter)
                % if property.boxed:
                    where F: FnOnce(&DeclaredValue<Box<longhands::${property.ident}::SpecifiedValue>>)
                % else:
                    where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
                % endif
        {
            f(&
                ::custom_properties::substitute(css, first_token_type, custom_properties)
                .and_then(|css| {
                    // As of this writing, only the base URL is used for property values:
                    //
                    // FIXME(pcwalton): Cloning the error reporter is slow! But so are custom
                    // properties, so whatever...
                    let context = ParserContext::new(Origin::Author,
                                                     url_data,
                                                     error_reporter,
                                                     None,
                                                     LengthParsingMode::Default);
                    Parser::new(&css).parse_entirely(|input| {
                        match from_shorthand {
                            None => {
                                longhands::${property.ident}
                                         ::parse_specified(&context, input).map(DeclaredValueOwned::Value)
                            }
                            Some(ShorthandId::All) => {
                                // No need to parse the 'all' shorthand as anything other than a CSS-wide
                                // keyword, after variable substitution.
                                Err(())
                            }
                            % for shorthand in data.shorthands_except_all():
                                % if property in shorthand.sub_properties:
                                    Some(ShorthandId::${shorthand.camel_case}) => {
                                        shorthands::${shorthand.ident}::parse_value(&context, input)
                                        .map(|result| {
                                            % if property.boxed:
                                                DeclaredValueOwned::Value(Box::new(result.${property.ident}))
                                            % else:
                                                DeclaredValueOwned::Value(result.${property.ident})
                                            % endif
                                        })
                                    }
                                % endif
                            % endfor
                            _ => unreachable!()
                        }
                    })
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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
    pub fn from_ident<'i>(ident: &Cow<'i, str>) -> Option<Self> {
        match_ignore_ascii_case! { ident,
            "initial" => Some(CSSWideKeyword::Initial),
            "inherit" => Some(CSSWideKeyword::Inherit),
            "unset" => Some(CSSWideKeyword::Unset),
            _ => None
        }
    }
}

impl ToCss for CSSWideKeyword {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(self.to_str())
    }
}

impl Parse for CSSWideKeyword {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let ident = input.expect_ident()?;
        input.expect_exhausted()?;
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
        /// This property(shorthand) is an alias of another property.
        const ALIAS_PROPERTY = 1 << 3,
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
}

/// An identifier for a given shorthand property.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ShorthandId {
    % for property in data.shorthands:
        /// ${property.name}
        ${property.camel_case},
    % endfor
}

impl ToCss for ShorthandId {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        dest.write_str(self.name())
    }
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
            PropertyDeclarationId::Custom(name) => write!(dest, "--{}", name),
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
                    PropertyId::Shorthand(shorthand) => shorthand.longhands().contains(&id),
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
            PropertyDeclarationId::Longhand(ref id) => shorthand.longhands().contains(id),
            _ => false,
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
            PropertyId::Custom(ref name) => write!(dest, "--{}", name),
        }
    }
}

impl PropertyId {
    /// Returns a given property from the string `s`.
    ///
    /// Returns Err(()) for unknown non-custom properties
    pub fn parse(property_name: Cow<str>) -> Result<Self, ()> {
        if let Ok(name) = ::custom_properties::parse_name(&property_name) {
            return Ok(PropertyId::Custom(::custom_properties::Name::from(name)))
        }

        // FIXME(https://github.com/rust-lang/rust/issues/33156): remove this enum and use PropertyId
        // when stable Rust allows destructors in statics.
        enum StaticId {
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
            None => Err(()),
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

    /// Returns a property id from Gecko's nsCSSPropertyID.
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
}

/// Includes shorthands before expansion
pub enum ParsedDeclaration {
    % for shorthand in data.shorthands:
        % if shorthand.name == "all":
        // No need for an All(shorthands::all::Longhands) case, since we can
        // never have any values for 'all' other than the CSS-wide keywords
        // and values with variable references.
        % else:
        /// ${shorthand.name}
        ${shorthand.camel_case}(shorthands::${shorthand.ident}::Longhands),
        % endif

        /// ${shorthand.name} with a CSS-wide keyword
        ${shorthand.camel_case}CSSWideKeyword(CSSWideKeyword),

        /// ${shorthand.name} with var() functions
        ${shorthand.camel_case}WithVariables(Arc<UnparsedValue>),
    % endfor

    /// Not a shorthand
    LonghandOrCustom(PropertyDeclaration),
}

impl ParsedDeclaration {
    /// Transform this ParsedDeclaration into a sequence of PropertyDeclaration
    /// by expanding shorthand declarations into their corresponding longhands
    ///
    /// Adds or overrides exsting declarations in the given block,
    /// except if existing declarations are more important.
    #[inline]
    pub fn expand_push_into(self, block: &mut PropertyDeclarationBlock,
                            importance: Importance) {
        self.expand_into(block, importance, false);
    }

    /// Transform this ParsedDeclaration into a sequence of PropertyDeclaration
    /// by expanding shorthand declarations into their corresponding longhands
    ///
    /// Add or override existing declarations in the given block.
    /// Return whether anything changed.
    #[inline]
    pub fn expand_set_into(self, block: &mut PropertyDeclarationBlock,
                           importance: Importance) -> bool {
        self.expand_into(block, importance, true)
    }

    fn expand_into(self, block: &mut PropertyDeclarationBlock,
                   importance: Importance,
                   overwrite_more_important: bool) -> bool {
        match self {
            % for shorthand in data.shorthands:
                % if shorthand.name != "all":
                ParsedDeclaration::${shorthand.camel_case}(
                    shorthands::${shorthand.ident}::Longhands {
                        % for sub_property in shorthand.sub_properties:
                            ${sub_property.ident},
                        % endfor
                    }
                ) => {
                    let mut changed = false;
                    % for sub_property in shorthand.sub_properties:
                        changed |= block.push_common(
                            PropertyDeclaration::${sub_property.camel_case}(
                                % if sub_property.boxed:
                                    Box::new(${sub_property.ident})
                                % else:
                                    ${sub_property.ident}
                                % endif
                            ),
                            importance,
                            overwrite_more_important,
                        );
                    % endfor
                    changed
                },
                % endif
                ParsedDeclaration::${shorthand.camel_case}CSSWideKeyword(keyword) => {
                    let mut changed = false;
                    % for sub_property in shorthand.sub_properties:
                        changed |= block.push_common(
                            PropertyDeclaration::CSSWideKeyword(
                                LonghandId::${sub_property.camel_case},
                                keyword,
                            ),
                            importance,
                            overwrite_more_important,
                        );
                    % endfor
                    changed
                },
                ParsedDeclaration::${shorthand.camel_case}WithVariables(value) => {
                    debug_assert_eq!(
                        value.from_shorthand,
                        Some(ShorthandId::${shorthand.camel_case})
                    );
                    let mut changed = false;
                    % for sub_property in shorthand.sub_properties:
                        changed |= block.push_common(
                            PropertyDeclaration::WithVariables(
                                LonghandId::${sub_property.camel_case},
                                value.clone()
                            ),
                            importance,
                            overwrite_more_important,
                        );
                    % endfor
                    changed
                }
            % endfor
            ParsedDeclaration::LonghandOrCustom(declaration) => {
                block.push_common(declaration, importance, overwrite_more_important)
            }
        }
    }

    /// The `in_keyframe_block` parameter controls this:
    ///
    /// https://drafts.csswg.org/css-animations/#keyframes
    /// > The <declaration-list> inside of <keyframe-block> accepts any CSS property
    /// > except those defined in this specification,
    /// > but does accept the `animation-play-state` property and interprets it specially.
    ///
    /// This will not actually parse Importance values, and will always set things
    /// to Importance::Normal. Parsing Importance values is the job of PropertyDeclarationParser,
    /// we only set them here so that we don't have to reallocate
    pub fn parse(id: PropertyId, context: &ParserContext, input: &mut Parser)
                 -> Result<ParsedDeclaration, PropertyDeclarationParseError> {
        let rule_type = context.rule_type();
        debug_assert!(rule_type == CssRuleType::Keyframe ||
                      rule_type == CssRuleType::Page ||
                      rule_type == CssRuleType::Style,
                      "Declarations are only expected inside a keyframe, page, or style rule.");
        match id {
            PropertyId::Custom(name) => {
                let value = match input.try(|i| CSSWideKeyword::parse(context, i)) {
                    Ok(keyword) => DeclaredValueOwned::CSSWideKeyword(keyword),
                    Err(()) => match ::custom_properties::SpecifiedValue::parse(context, input) {
                        Ok(value) => DeclaredValueOwned::Value(value),
                        Err(()) => return Err(PropertyDeclarationParseError::InvalidValue),
                    }
                };
                Ok(ParsedDeclaration::LonghandOrCustom(PropertyDeclaration::Custom(name, value)))
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
                        % if property.internal:
                            if context.stylesheet_origin != Origin::UserAgent {
                                return Err(PropertyDeclarationParseError::UnknownProperty)
                            }
                        % endif
                        % if not property.allowed_in_page_rule:
                            if rule_type == CssRuleType::Page {
                                return Err(PropertyDeclarationParseError::NotAllowedInPageRule)
                            }
                        % endif

                        ${property_pref_check(property)}

                        match longhands::${property.ident}::parse_declared(context, input) {
                            Ok(value) => {
                                Ok(ParsedDeclaration::LonghandOrCustom(value))
                            },
                            Err(()) => Err(PropertyDeclarationParseError::InvalidValue),
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
                    % if shorthand.internal:
                        if context.stylesheet_origin != Origin::UserAgent {
                            return Err(PropertyDeclarationParseError::UnknownProperty)
                        }
                    % endif
                    % if not shorthand.allowed_in_page_rule:
                        if rule_type == CssRuleType::Page {
                            return Err(PropertyDeclarationParseError::NotAllowedInPageRule)
                        }
                    % endif

                    ${property_pref_check(shorthand)}

                    match input.try(|i| CSSWideKeyword::parse(context, i)) {
                        Ok(keyword) => {
                            Ok(ParsedDeclaration::${shorthand.camel_case}CSSWideKeyword(keyword))
                        },
                        Err(()) => {
                            shorthands::${shorthand.ident}::parse(context, input)
                                .map_err(|()| PropertyDeclarationParseError::InvalidValue)
                        }
                    }
                }
            % endfor
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

/// The result of parsing a property declaration.
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PropertyDeclarationParseError {
    /// The property declaration was for an unknown property.
    UnknownProperty,
    /// The property declaration was for a disabled experimental property.
    ExperimentalProperty,
    /// The property declaration contained an invalid value.
    InvalidValue,
    /// The declaration contained an animation property, and we were parsing
    /// this as a keyframe block (so that property should be ignored).
    ///
    /// See: https://drafts.csswg.org/css-animations/#keyframes
    AnimationPropertyInKeyframeBlock,
    /// The property is not allowed within a page rule.
    NotAllowedInPageRule,
}

impl fmt::Debug for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.id().to_css(f));
        try!(f.write_str(": "));
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
                if with_variables.from_shorthand.is_none() {
                    dest.write_str(&*with_variables.css)?
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

<%def name="property_pref_check(property)">
    % if property.experimental and product == "servo":
        if !PREFS.get("${property.experimental}")
            .as_boolean().unwrap_or(false) {
            return Err(PropertyDeclarationParseError::ExperimentalProperty)
        }
    % endif
    % if product == "gecko":
        <%
            # gecko can't use the identifier `float`
            # and instead uses `float_`
            # XXXManishearth make this an attr on the property
            # itself?
            pref_ident = property.ident
            if pref_ident == "float":
                pref_ident = "float_"
        %>
        if structs::root::mozilla::SERVO_PREF_ENABLED_${pref_ident} {
            let id = structs::${helpers.to_nscsspropertyid(property.ident)};
            let enabled = unsafe { bindings::Gecko_PropertyId_IsPrefEnabled(id) };
            if !enabled {
                return Err(PropertyDeclarationParseError::ExperimentalProperty)
            }
        }
    % endif
</%def>

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
                } else { None }
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
        // first generate longhand to shorthands lookup map
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
                PropertyDeclaration::${property.camel_case}(_) => ${property.ident.upper()},
            % endfor
            PropertyDeclaration::CSSWideKeyword(id, _) |
            PropertyDeclaration::WithVariables(id, _) => match id {
                % for property in data.longhands:
                    LonghandId::${property.camel_case} => ${property.ident.upper()},
                % endfor
            },
            PropertyDeclaration::Custom(_, _) => &[]
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
                    /// Set ${longhand.name}.
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T) {
                        self.${longhand.ident} = v;
                    }
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
                                              _: Option<Au>) {
                    self.copy_font_size_from(parent);
                }
                /// (Servo does not handle MathML, so this just calls set_font_size)
                pub fn apply_font_size(&mut self,
                                       v: longhands::font_size::computed_value::T,
                                       _: &Self) -> Option<Au> {
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
                self.animation_name_iter().any(|name| name.0 != atom!(""))
            }

            /// Returns whether there are any transitions specified.
            #[cfg(feature = "servo")]
            pub fn specifies_transitions(&self) -> bool {
                self.transition_property_count() > 0
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
#[cfg_attr(feature = "servo", derive(Clone, Debug))]
pub struct ComputedValues {
    % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor
    custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
    /// The writing mode of this computed values struct.
    pub writing_mode: WritingMode,
    /// The root element's computed font size.
    pub root_font_size: Au,
    /// The keyword behind the current font-size property, if any
    pub font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>,
}

#[cfg(feature = "servo")]
impl ComputedValues {
    /// Construct a `ComputedValues` instance.
    pub fn new(custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
               writing_mode: WritingMode,
               root_font_size: Au,
               font_size_keyword: Option<(longhands::font_size::KeywordSize, f32)>,
            % for style_struct in data.active_style_structs():
               ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
            % endfor
    ) -> Self {
        ComputedValues {
            custom_properties: custom_properties,
            writing_mode: writing_mode,
            root_font_size: root_font_size,
            font_size_keyword: font_size_keyword,
        % for style_struct in data.active_style_structs():
            ${style_struct.ident}: ${style_struct.ident},
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

        /// Get a mutable reference to the ${style_struct.name} struct.
        #[inline]
        pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
            Arc::make_mut(&mut self.${style_struct.ident})
        }
    % endfor

    /// Get the custom properties map if necessary.
    ///
    /// Cloning the Arc here is fine because it only happens in the case where
    /// we have custom properties, and those are both rare and expensive.
    fn custom_properties(&self) -> Option<Arc<::custom_properties::ComputedValuesMap>> {
        self.custom_properties.as_ref().map(|x| x.clone())
    }

    /// Whether this style has a -moz-binding value. This is always false for
    /// Servo for obvious reasons.
    pub fn has_moz_binding(&self) -> bool { false }

    /// Returns whether this style's display value is equal to contents.
    ///
    /// Since this isn't supported in Servo, this is always false for Servo.
    pub fn is_display_contents(&self) -> bool { false }

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
    pub fn resolve_color(&self, color: CSSParserColor) -> RGBA {
        match color {
            CSSParserColor::RGBA(rgba) => rgba,
            CSSParserColor::CurrentColor => self.get_color().color,
        }
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

    /// https://drafts.csswg.org/css-transforms/#grouping-property-values
    pub fn get_used_transform_style(&self) -> computed_values::transform_style::T {
        use computed_values::mix_blend_mode;
        use computed_values::transform_style;

        let effects = self.get_effects();
        let box_ = self.get_box();

        // TODO(gw): Add clip-path, isolation, mask-image, mask-border-source when supported.
        if effects.opacity < 1.0 ||
           !effects.filter.is_empty() ||
           !effects.clip.is_auto() {
           effects.mix_blend_mode != mix_blend_mode::T::normal ||
            return transform_style::T::flat;
        }

        if box_.transform_style == transform_style::T::auto {
            if box_.transform.0.is_some() {
                return transform_style::T::flat;
            }
            if let Either::First(ref _length) = box_.perspective {
                return transform_style::T::flat;
            }
        }

        // Return the computed value if not overridden by the above exceptions
        box_.transform_style
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


#[cfg(feature = "servo")]
pub use self::lazy_static_module::INITIAL_SERVO_VALUES;

// Use a module to work around #[cfg] on lazy_static! not being applied to every generated item.
#[cfg(feature = "servo")]
#[allow(missing_docs)]
mod lazy_static_module {
    use logical_geometry::WritingMode;
    use std::sync::Arc;
    use super::{ComputedValues, longhands, style_structs};

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
            root_font_size: longhands::font_size::get_initial_value(),
            font_size_keyword: Some((Default::default(), 1.)),
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
               cascade_info: Option<<&mut CascadeInfo>,
               error_reporter: &ParseErrorReporter,
               font_metrics_provider: &FontMetricsProvider,
               flags: CascadeFlags)
               -> ComputedValues {
    debug_assert_eq!(parent_style.is_some(), layout_parent_style.is_some());
    let (is_root_element, inherited_style, layout_parent_style) = match parent_style {
        Some(parent_style) => {
            (false,
             parent_style,
             layout_parent_style.unwrap())
        },
        None => {
            (true,
             device.default_computed_values(),
             device.default_computed_values())
        }
    };

    let iter_declarations = || {
        rule_node.self_and_ancestors().flat_map(|node| {
            let declarations = match node.style_source() {
                Some(source) => source.read(node.cascade_level().guard(guards)).declarations(),
                // The root node has no style source.
                None => &[]
            };
            let node_importance = node.importance();
            declarations
                .iter()
                // Yield declarations later in source order (with more precedence) first.
                .rev()
                .filter_map(move |&(ref declaration, declaration_importance)| {
                    if declaration_importance == node_importance {
                        Some(declaration)
                    } else {
                        None
                    }
                })
        })
    };
    apply_declarations(device,
                       is_root_element,
                       iter_declarations,
                       inherited_style,
                       layout_parent_style,
                       cascade_info,
                       error_reporter,
                       font_metrics_provider,
                       flags)
}

/// NOTE: This function expects the declaration with more priority to appear
/// first.
#[allow(unused_mut)] // conditionally compiled code for "position"
pub fn apply_declarations<'a, F, I>(device: &Device,
                                    is_root_element: bool,
                                    iter_declarations: F,
                                    inherited_style: &ComputedValues,
                                    layout_parent_style: &ComputedValues,
                                    mut cascade_info: Option<<&mut CascadeInfo>,
                                    error_reporter: &ParseErrorReporter,
                                    font_metrics_provider: &FontMetricsProvider,
                                    flags: CascadeFlags)
                                    -> ComputedValues
    where F: Fn() -> I,
          I: Iterator<Item = &'a PropertyDeclaration>,
{
    let default_style = device.default_computed_values();
    let inherited_custom_properties = inherited_style.custom_properties();
    let mut custom_properties = None;
    let mut seen_custom = HashSet::new();
    for declaration in iter_declarations() {
        if let PropertyDeclaration::Custom(ref name, ref value) = *declaration {
            ::custom_properties::cascade(
                &mut custom_properties, &inherited_custom_properties,
                &mut seen_custom, name, value.borrow());
        }
    }

    let custom_properties =
        ::custom_properties::finish_cascade(
            custom_properties, &inherited_custom_properties);

    let starting_style = if !flags.contains(INHERIT_ALL) {
        ComputedValues::new(custom_properties,
                            WritingMode::empty(),
                            inherited_style.root_font_size,
                            inherited_style.font_size_keyword,
                            % for style_struct in data.active_style_structs():
                                % if style_struct.inherited:
                                    inherited_style.clone_${style_struct.name_lower}(),
                                % else:
                                    default_style.clone_${style_struct.name_lower}(),
                                % endif
                            % endfor
                            )
    } else {
        ComputedValues::new(custom_properties,
                            WritingMode::empty(),
                            inherited_style.root_font_size,
                            inherited_style.font_size_keyword,
                            % for style_struct in data.active_style_structs():
                                inherited_style.clone_${style_struct.name_lower}(),
                            % endfor
                            )
    };

    let mut context = computed::Context {
        is_root_element: is_root_element,
        device: device,
        inherited_style: inherited_style,
        layout_parent_style: layout_parent_style,
        style: starting_style,
        font_metrics_provider: font_metrics_provider,
        in_media_query: false,
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
            // Pull these out so that we can
            // compute them in a specific order without
            // introducing more iterations
            let mut font_size = None;
            let mut font_family = None;
        % endif
        for declaration in iter_declarations() {
            let longhand_id = match declaration.id() {
                PropertyDeclarationId::Longhand(id) => id,
                PropertyDeclarationId::Custom(..) => continue,
            };

            // The computed value of some properties depends on the
            // (sometimes computed) value of *other* properties.
            //
            // So we classify properties into "early" and "other", such that
            // the only dependencies can be from "other" to "early".
            //
            // We iterate applicable_declarations twice, first cascading
            // "early" properties then "other".
            //
            // Unfortunately, it’s not easy to check that this
            // classification is correct.
            let is_early_property = matches!(longhand_id,
                LonghandId::FontSize |
                LonghandId::FontFamily |
                LonghandId::Color |
                LonghandId::TextDecorationLine |
                LonghandId::WritingMode |
                LonghandId::Direction
                % if product == 'gecko':
                    | LonghandId::TextOrientation
                    | LonghandId::AnimationName
                    | LonghandId::TransitionProperty
                    | LonghandId::XLang
                    | LonghandId::MozScriptLevel
                % endif
            );
            if
                % if category_to_cascade_now == "early":
                    !
                % endif
                is_early_property
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
            if let Some(declaration) = font_family {
                let discriminant = LonghandId::FontFamily as usize;
                (CASCADE_PROPERTY[discriminant])(declaration,
                                                 inherited_style,
                                                 default_style,
                                                 &mut context,
                                                 &mut cacheable,
                                                 &mut cascade_info,
                                                 error_reporter);
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
            } else {
                // Font size must be explicitly inherited to handle keyword
                // sizes and scriptlevel
                let discriminant = LonghandId::FontSize as usize;
                let size = PropertyDeclaration::CSSWideKeyword(
                    LonghandId::FontSize, CSSWideKeyword::Inherit
                );
                (CASCADE_PROPERTY[discriminant])(&size,
                                                 inherited_style,
                                                 default_style,
                                                 &mut context,
                                                 &mut cacheable,
                                                 &mut cascade_info,
                                                 error_reporter);
            }
        % endif
    % endfor

    let mut style = context.style;

    let mut positioned = matches!(style.get_box().clone_position(),
        longhands::position::SpecifiedValue::absolute |
        longhands::position::SpecifiedValue::fixed);

    // https://fullscreen.spec.whatwg.org/#new-stacking-layer
    // Any position value other than 'absolute' and 'fixed' are
    // computed to 'absolute' if the element is in a top layer.
    % if product == "gecko":
        if !positioned &&
            matches!(style.get_box().clone__moz_top_layer(),
                     longhands::_moz_top_layer::SpecifiedValue::top) {
            positioned = true;
            style.mutate_box().set_position(longhands::position::computed_value::T::absolute);
        }
    % endif

    let positioned = positioned; // To ensure it's not mutated further.

    let floated = style.get_box().clone_float() != longhands::float::computed_value::T::none;
    let is_item = matches!(context.layout_parent_style.get_box().clone_display(),
        % if product == "gecko":
        computed_values::display::T::grid |
        computed_values::display::T::inline_grid |
        % endif
        computed_values::display::T::flex |
        computed_values::display::T::inline_flex);

    let (blockify_root, blockify_item) =
        if flags.contains(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP) {
            (false, false)
        } else {
            (is_root_element, is_item)
        };

    if positioned || floated || blockify_root || blockify_item {
        use computed_values::display::T;

        let specified_display = style.get_box().clone_display();
        let computed_display = match specified_display {
            // Values that have a corresponding block-outside version.
            T::inline_table => Some(T::table),
            % if product == "gecko":
            T::inline_flex => Some(T::flex),
            T::inline_grid => Some(T::grid),
            T::_webkit_inline_box => Some(T::_webkit_box),
            % endif

            // Special handling for contents and list-item on the root element for Gecko.
            % if product == "gecko":
            T::contents | T::list_item if blockify_root => Some(T::block),
            % endif

            // Values that are not changed by blockification.
            T::none | T::block | T::flex | T::list_item | T::table => None,
            % if product == "gecko":
            T::contents | T::flow_root | T::grid | T::_webkit_box => None,
            % endif

            // Everything becomes block.
            _ => Some(T::block),
        };
        if let Some(computed_display) = computed_display {
            let box_ = style.mutate_box();
            % if product == "servo":
                box_.set_display(computed_display);
                box_.set__servo_display_for_hypothetical_box(if blockify_root || blockify_item {
                    computed_display
                } else {
                    specified_display
                });
            % else:
                box_.set_adjusted_display(computed_display);
            % endif
        }
    }

    {
        use computed_values::display::T as display;
        // CSS writing modes spec (https://drafts.csswg.org/css-writing-modes-3/#block-flow):
        //
        //  If a box has a different writing-mode value than its containing block:
        //  - If the box has a specified display of inline, its display computes to inline-block. [CSS21]
        //
        // www-style mail regarding above spec: https://lists.w3.org/Archives/Public/www-style/2017Mar/0045.html
        // See https://github.com/servo/servo/issues/15754
        let our_writing_mode = style.get_inheritedbox().clone_writing_mode();
        let parent_writing_mode = context.layout_parent_style.get_inheritedbox().clone_writing_mode();
        if our_writing_mode != parent_writing_mode &&
           style.get_box().clone_display() == display::inline {
            style.mutate_box().set_display(display::inline_block);
        }
    }

    {
        use computed_values::overflow_x::T as overflow;
        use computed_values::overflow_y;
        match (style.get_box().clone_overflow_x() == longhands::overflow_x::computed_value::T::visible,
               style.get_box().clone_overflow_y().0 == longhands::overflow_x::computed_value::T::visible) {
            (true, true) => {}
            (true, _) => {
                style.mutate_box().set_overflow_x(overflow::auto);
            }
            (_, true) => {
                style.mutate_box().set_overflow_y(overflow_y::T(overflow::auto));
            }
            _ => {}
        }
    }

    // CSS 2.1 section 9.7:
    //
    //    If 'position' has the value 'absolute' or 'fixed', [...] the computed
    //    value of 'float' is 'none'.
    //
    if positioned && floated {
        style.mutate_box().set_float(longhands::float::computed_value::T::none);
    }

    // This implements an out-of-date spec. The new spec moves the handling of
    // this to layout, which Gecko implements but Servo doesn't.
    //
    // See https://github.com/servo/servo/issues/15229
    % if product == "servo" and "align-items" in data.longhands_by_name:
    {
        use computed_values::align_self::T as align_self;
        use computed_values::align_items::T as align_items;
        if style.get_position().clone_align_self() == computed_values::align_self::T::auto && !positioned {
            let self_align =
                match context.layout_parent_style.get_position().clone_align_items() {
                    align_items::stretch => align_self::stretch,
                    align_items::baseline => align_self::baseline,
                    align_items::flex_start => align_self::flex_start,
                    align_items::flex_end => align_self::flex_end,
                    align_items::center => align_self::center,
                };
            style.mutate_position().set_align_self(self_align);
        }
    }
    % endif

    // The initial value of border-*-width may be changed at computed value time.
    % for side in ["top", "right", "bottom", "left"]:
        // Like calling to_computed_value, which wouldn't type check.
        if style.get_border().clone_border_${side}_style().none_or_hidden() &&
           style.get_border().border_${side}_has_nonzero_width() {
            style.mutate_border().set_border_${side}_width(Au(0));
        }
    % endfor


    % if product == "gecko":
        // FIXME(emilio): This is effectively creating a new nsStyleBackground
        // and nsStyleSVG per element. We should only do this when necessary
        // using the `seen` bitfield!
        style.mutate_background().fill_arrays();
        style.mutate_svg().fill_arrays();
    % endif

    // The initial value of outline width may be changed at computed value time.
    if style.get_outline().clone_outline_style().none_or_hidden() &&
       style.get_outline().outline_has_nonzero_width() {
        style.mutate_outline().set_outline_width(Au(0));
    }

    if is_root_element {
        let s = style.get_font().clone_font_size();
        style.root_font_size = s;
    }

    % if product == "servo":
        if seen.contains(LonghandId::FontStyle) ||
           seen.contains(LonghandId::FontWeight) ||
           seen.contains(LonghandId::FontStretch) ||
           seen.contains(LonghandId::FontFamily) {
            style.mutate_font().compute_font_hash();
        }
    % endif

    style
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

/// Testing function to check the size of a PropertyDeclaration. We implement
/// this here so that the code can be used by both servo and stylo unit tests.
/// This is important because structs can have different sizes in stylo and
/// servo.
#[cfg(feature = "testing")]
pub fn test_size_of_property_declaration() {
    use std::mem::size_of;

    let old = 32;
    let new = size_of::<PropertyDeclaration>();
    if new < old {
        panic!("Your changes have decreased the stack size of PropertyDeclaration enum from {} to {}. \
                Good work! Please update the size in components/style/properties/properties.mako.rs.",
                old, new)
    } else if new > old {
        panic!("Your changes have increased the stack size of PropertyDeclaration enum from {} to {}. \
                These enum is present in large quantities in the style, and increasing the size \
                may negatively affect style system performance. Please consider using `boxed=\"True\"` in \
                the longhand If you feel that the increase is necessary, update to the new size in \
                components/style/properties/properties.mako.rs.",
                old, new)
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
