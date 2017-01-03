/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

// Please note that valid Rust syntax may be mangled by the Mako parser.
// For example, Vec<&Foo> will be mangled as Vec&Foo>. To work around these issues, the code
// can be escaped. In the above example, Vec<<&Foo> or Vec< &Foo> achieves the desired result of Vec<&Foo>.

<%namespace name="helpers" file="/helpers.mako.rs" />

use std::borrow::Cow;
use std::boxed::Box as StdBox;
use std::collections::HashSet;
use std::fmt::{self, Write};
use std::sync::Arc;

use app_units::Au;
#[cfg(feature = "servo")] use cssparser::{Color as CSSParserColor, RGBA};
use cssparser::{Parser, TokenSerializationType};
use error_reporting::ParseErrorReporter;
#[cfg(feature = "servo")] use euclid::side_offsets::SideOffsets2D;
use euclid::size::Size2D;
use computed_values;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "gecko")] use gecko_bindings::structs::nsCSSPropertyID;
#[cfg(feature = "servo")] use logical_geometry::{LogicalMargin, PhysicalSide};
use logical_geometry::WritingMode;
use parser::{Parse, ParserContext, ParserContextExtraData};
#[cfg(feature = "servo")] use servo_config::prefs::PREFS;
use servo_url::ServoUrl;
use style_traits::ToCss;
use stylesheets::Origin;
#[cfg(feature = "servo")] use values::Either;
use values::{HasViewportPercentage, computed};
use cascade_info::CascadeInfo;
use rule_tree::StrongRuleNode;
#[cfg(feature = "servo")] use values::specified::BorderStyle;

use self::property_bit_field::PropertyBitField;
pub use self::declaration_block::*;

#[cfg(feature = "gecko")]
#[macro_export]
macro_rules! property_name {
    ($s: tt) => { atom!($s) }
}

<%!
    from data import Method, Keyword, to_rust_ident
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

/// A module with code for all the shorthand css properties, and a few
/// serialization helpers.
#[allow(missing_docs)]
pub mod shorthands {
    use cssparser::Parser;
    use parser::{Parse, ParserContext};
    use values::specified;

    bitflags! {
        flags SerializeFlags: u8 {
            const ALL_INHERIT = 0b001,
            const ALL_INITIAL = 0b010,
            const ALL_UNSET   = 0b100,
        }
    }

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
    <%include file="/shorthand/text.mako.rs" />
}

/// A module with all the code related to animated properties.
///
/// This needs to be "included" by mako at least after all longhand modules,
/// given they populate the global data.
pub mod animated_properties {
    <%include file="/helpers/animated_properties.mako.rs" />
}


// TODO(SimonSapin): Convert this to a syntax extension rather than a Mako template.
// Maybe submit for inclusion in libstd?
mod property_bit_field {
    use logical_geometry::WritingMode;

    /// A bitfield for all longhand properties, in order to quickly test whether
    /// we've seen one of them.
    pub struct PropertyBitField {
        storage: [u32; (${len(data.longhands)} - 1 + 32) / 32]
    }

    impl PropertyBitField {
        /// Create a new `PropertyBitField`, with all the bits set to zero.
        #[inline]
        pub fn new() -> PropertyBitField {
            PropertyBitField { storage: [0; (${len(data.longhands)} - 1 + 32) / 32] }
        }

        #[inline]
        fn get(&self, bit: usize) -> bool {
            (self.storage[bit / 32] & (1 << (bit % 32))) != 0
        }

        #[inline]
        fn set(&mut self, bit: usize) {
            self.storage[bit / 32] |= 1 << (bit % 32)
        }
        % for i, property in enumerate(data.longhands):
            % if not property.derived_from:
                #[allow(non_snake_case, missing_docs)]
                #[inline]
                pub fn get_${property.ident}(&self) -> bool {
                    self.get(${i})
                }
                #[allow(non_snake_case)]
                #[inline]
                pub fn set_${property.ident}(&mut self) {
                    self.set(${i})
                }
            % endif
            % if property.logical:
                #[allow(non_snake_case, missing_docs)]
                pub fn get_physical_${property.ident}(&self, wm: WritingMode) -> bool {
                    <%helpers:logical_setter_helper name="${property.name}">
                        <%def name="inner(physical_ident)">
                            self.get_${physical_ident}()
                        </%def>
                    </%helpers:logical_setter_helper>
                }
                #[allow(non_snake_case, missing_docs)]
                pub fn set_physical_${property.ident}(&mut self, wm: WritingMode) {
                    <%helpers:logical_setter_helper name="${property.name}">
                        <%def name="inner(physical_ident)">
                            self.set_${physical_ident}()
                        </%def>
                    </%helpers:logical_setter_helper>
                }
            % endif
        % endfor
    }
}

% for property in data.longhands:
    % if not property.derived_from:
        /// Perform CSS variable substitution if needed, and execute `f` with
        /// the resulting declared value.
        #[allow(non_snake_case)]
        fn substitute_variables_${property.ident}<F>(
            value: &DeclaredValue<longhands::${property.ident}::SpecifiedValue>,
            custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
            f: F,
            error_reporter: &mut StdBox<ParseErrorReporter + Send>)
            where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
        {
            if let DeclaredValue::WithVariables {
                ref css, first_token_type, ref base_url, from_shorthand
            } = *value {
                // FIXME(heycam): A ParserContextExtraData should be built from data
                // stored in the WithVariables, in case variable expansion results in
                // a url() value.
                let extra_data = ParserContextExtraData::default();
                substitute_variables_${property.ident}_slow(css,
                                                            first_token_type,
                                                            base_url,
                                                            from_shorthand,
                                                            custom_properties,
                                                            f,
                                                            error_reporter,
                                                            extra_data);
            } else {
                f(value);
            }
        }

        #[allow(non_snake_case)]
        #[inline(never)]
        fn substitute_variables_${property.ident}_slow<F>(
                css: &String,
                first_token_type: TokenSerializationType,
                base_url: &ServoUrl,
                from_shorthand: Option<ShorthandId>,
                custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
                f: F,
                error_reporter: &mut StdBox<ParseErrorReporter + Send>,
                extra_data: ParserContextExtraData)
                where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>)
        {
            f(&
                ::custom_properties::substitute(css, first_token_type, custom_properties)
                .and_then(|css| {
                    // As of this writing, only the base URL is used for property values:
                    //
                    // FIXME(pcwalton): Cloning the error reporter is slow! But so are custom
                    // properties, so whatever...
                    let context = ParserContext::new_with_extra_data(
                        ::stylesheets::Origin::Author, base_url, (*error_reporter).clone(),
                        extra_data);
                    Parser::new(&css).parse_entirely(|input| {
                        match from_shorthand {
                            None => {
                                longhands::${property.ident}::parse_specified(&context, input)
                            }
                            % for shorthand in data.shorthands:
                                % if property in shorthand.sub_properties:
                                    Some(ShorthandId::${shorthand.camel_case}) => {
                                        shorthands::${shorthand.ident}::parse_value(&context, input)
                                        .map(|result| match result.${property.ident} {
                                            Some(value) => DeclaredValue::Value(value),
                                            None => DeclaredValue::Initial,
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
                    DeclaredValue::${"Inherit" if property.style_struct.inherited else "Initial"}
                )
            );
        }
    % endif
% endfor

/// Given a property declaration block, only keep the "winning" declaration for
/// any given property, by importance then source order.
///
/// The input and output are in source order
fn deduplicate_property_declarations(block: &mut PropertyDeclarationBlock) {
    let mut deduplicated = Vec::new();
    let mut seen_normal = PropertyBitField::new();
    let mut seen_important = PropertyBitField::new();
    let mut seen_custom_normal = Vec::new();
    let mut seen_custom_important = Vec::new();

    for (declaration, importance) in block.declarations.drain(..).rev() {
        match declaration {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(..) => {
                    % if not property.derived_from:
                        if importance.important() {
                            if seen_important.get_${property.ident}() {
                                block.important_count -= 1;
                                continue
                            }
                            if seen_normal.get_${property.ident}() {
                                remove_one(&mut deduplicated, |d| {
                                    matches!(d, &(PropertyDeclaration::${property.camel_case}(..), _))
                                });
                            }
                            seen_important.set_${property.ident}()
                        } else {
                            if seen_normal.get_${property.ident}() ||
                               seen_important.get_${property.ident}() {
                                continue
                            }
                            seen_normal.set_${property.ident}()
                        }
                    % else:
                        unreachable!();
                    % endif
                },
            % endfor
            PropertyDeclaration::Custom(ref name, _) => {
                if importance.important() {
                    if seen_custom_important.contains(name) {
                        block.important_count -= 1;
                        continue
                    }
                    if seen_custom_normal.contains(name) {
                        remove_one(&mut deduplicated, |d| {
                            matches!(d, &(PropertyDeclaration::Custom(ref n, _), _) if n == name)
                        });
                    }
                    seen_custom_important.push(name.clone())
                } else {
                    if seen_custom_normal.contains(name) ||
                       seen_custom_important.contains(name) {
                        continue
                    }
                    seen_custom_normal.push(name.clone())
                }
            }
        }
        deduplicated.push((declaration, importance))
    }
    deduplicated.reverse();
    block.declarations = deduplicated;
}

#[inline]
fn remove_one<T, F: FnMut(&T) -> bool>(v: &mut Vec<T>, mut remove_this: F) {
    let previous_len = v.len();
    v.retain(|x| !remove_this(x));
    debug_assert_eq!(v.len(), previous_len - 1);
}

/// An enum to represent a CSS Wide keyword.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CSSWideKeyword {
    /// The `initial` keyword.
    InitialKeyword,
    /// The `inherit` keyword.
    InheritKeyword,
    /// The `unset` keyword.
    UnsetKeyword,
}

impl Parse for CSSWideKeyword {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "initial" => Ok(CSSWideKeyword::InitialKeyword),
            "inherit" => Ok(CSSWideKeyword::InheritKeyword),
            "unset" => Ok(CSSWideKeyword::UnsetKeyword),
            _ => Err(())
        }
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
            % for property in data.shorthands:
                ShorthandId::${property.camel_case} => {
                    match shorthands::${property.ident}::LonghandsToSerialize::from_iter(declarations) {
                        Ok(longhands) => longhands.to_css(dest),
                        Err(_) => Err(fmt::Error)
                    }
                },
            % endfor
        }
    }

    /// Overflow does not behave like a normal shorthand. When overflow-x and
    /// overflow-y are not of equal values, they no longer use the shared
    /// property name "overflow".
    ///
    /// We use this function as a special-case for that.
    pub fn overflow_longhands_to_css<'a, W, I>(&self,
                                               declarations: I,
                                               dest: &mut W)
                                               -> fmt::Result
        where W: fmt::Write,
              I: Iterator<Item=&'a PropertyDeclaration>,
    {
        match *self {
            ShorthandId::Overflow => {
                match shorthands::overflow::LonghandsToSerialize::from_iter(declarations) {
                    Ok(longhands) => longhands.to_css_declared_with_name(dest),
                    Err(_) => Err(fmt::Error)
                }
            },
            _ => Err(fmt::Error)
        }
    }

    /// Serializes the possible shorthand name with value to input buffer given
    /// a list of longhand declarations.
    ///
    /// On success, returns true if the shorthand value is written, or false if
    /// no shorthand value is present.
    pub fn serialize_shorthand_to_buffer<'a, W, I>(self,
                                                   dest: &mut W,
                                                   declarations: I,
                                                   is_first_serialization: &mut bool,
                                                   importance: Importance)
                                                   -> Result<bool, fmt::Error>
        where W: Write,
              I: IntoIterator<Item=&'a PropertyDeclaration>,
              I::IntoIter: Clone,
    {
        match self.get_shorthand_appendable_value(declarations) {
            None => Ok(false),
            Some(appendable_value) => {
                append_serialization(
                    dest,
                    &self,
                    appendable_value,
                    importance,
                    is_first_serialization
                ).and_then(|_| Ok(true))
            }
        }
    }

    fn get_shorthand_appendable_value<'a, I>(self,
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
               return Some(AppendableValue::Css(css));
           }
           return None;
        }

        if !declarations3.any(|d| d.with_variables()) {
            return Some(AppendableValue::DeclarationsForShorthand(self, declarations));
        }

        None
    }
}

/// Servo's representation of a declared value for a given `T`, which is the
/// declared value for that property.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum DeclaredValue<T> {
    /// A known specified value from the stylesheet.
    Value(T),
    /// A value that contained any css variables.
    WithVariables {
        /// The css serialization for this value.
        css: String,
        /// The first token type for this serialization.
        first_token_type: TokenSerializationType,
        /// The base url.
        base_url: ServoUrl,
        /// The shorthand this came from.
        from_shorthand: Option<ShorthandId>,
    },
    /// The `initial` keyword.
    Initial,
    /// The `inherit` keyword.
    Inherit,
    /// The `unset` keyword.
    Unset,
}

impl<T: HasViewportPercentage> HasViewportPercentage for DeclaredValue<T> {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            DeclaredValue::Value(ref v) => v.has_viewport_percentage(),
            DeclaredValue::WithVariables { .. } => {
                panic!("DeclaredValue::has_viewport_percentage without \
                        resolving variables!")
            },
            DeclaredValue::Initial |
            DeclaredValue::Inherit |
            DeclaredValue::Unset => false,
        }
    }
}

impl<T: ToCss> ToCss for DeclaredValue<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            DeclaredValue::Value(ref inner) => inner.to_css(dest),
            DeclaredValue::WithVariables { ref css, from_shorthand: None, .. } => {
                dest.write_str(css)
            }
            // https://drafts.csswg.org/css-variables/#variables-in-shorthands
            DeclaredValue::WithVariables { .. } => Ok(()),
            DeclaredValue::Initial => dest.write_str("initial"),
            DeclaredValue::Inherit => dest.write_str("inherit"),
            DeclaredValue::Unset => dest.write_str("unset"),
        }
    }
}

/// An identifier for a given property declaration, which can be either a
/// longhand or a custom property.
#[derive(PartialEq, Clone)]
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

// FIXME(https://github.com/rust-lang/rust/issues/33156): remove this enum and use PropertyId
// when stable Rust allows destructors in statics.
enum StaticId {
    Longhand(LonghandId),
    Shorthand(ShorthandId),
}
include!(concat!(env!("OUT_DIR"), "/static_ids.rs"));

impl PropertyId {
    /// Returns a given property from the string `s`.
    ///
    /// Returns Err(()) for unknown non-custom properties
    pub fn parse(s: Cow<str>) -> Result<Self, ()> {
        if let Ok(name) = ::custom_properties::parse_name(&s) {
            return Ok(PropertyId::Custom(::custom_properties::Name::from(name)))
        }

        let lower_case = ::str::cow_into_ascii_lowercase(s);
        match STATIC_IDS.get(&*lower_case) {
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
        <%
            def to_nscsspropertyid(ident):
                if ident == "word_wrap":
                    return "nsCSSPropertyID_eCSSPropertyAlias_WordWrap"

                if ident == "float":
                    ident = "float_"
                return "nsCSSPropertyID::eCSSProperty_" + ident
        %>
        match id {
            % for property in data.longhands:
                ${to_nscsspropertyid(property.ident)} => {
                    Ok(PropertyId::Longhand(LonghandId::${property.camel_case}))
                }
            % endfor
            % for property in data.shorthands:
                ${to_nscsspropertyid(property.ident)} => {
                    Ok(PropertyId::Shorthand(ShorthandId::${property.camel_case}))
                }
            % endfor
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

/// Servo's representation for a property declaration.
#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum PropertyDeclaration {
    % for property in data.longhands:
        /// ${property.name}
        ${property.camel_case}(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
    /// A custom property declaration, with the property name and the declared
    /// value.
    Custom(::custom_properties::Name, DeclaredValue<::custom_properties::SpecifiedValue>),
}

impl HasViewportPercentage for PropertyDeclaration {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(ref val) => {
                    val.has_viewport_percentage()
                },
            % endfor
            PropertyDeclaration::Custom(_, ref val) => {
                val.has_viewport_percentage()
            }
        }
    }
}

/// The result of parsing a property declaration.
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PropertyDeclarationParseResult {
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
    /// The declaration was either valid or ignored.
    ValidOrIgnoredDeclaration,
}

impl fmt::Debug for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(self.id().to_css(f));
        try!(f.write_str(": "));
        match *self {
            % for property in data.longhands:
                % if not property.derived_from:
                    PropertyDeclaration::${property.camel_case}(ref value) => value.to_css(f),
                % endif
            % endfor
            PropertyDeclaration::Custom(_, ref value) => value.to_css(f),
            % if any(property.derived_from for property in data.longhands):
                _ => Err(fmt::Error),
            % endif
        }
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
            PropertyDeclaration::Custom(_, ref value) => value.to_css(dest),
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
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(..) => {
                    PropertyDeclarationId::Longhand(LonghandId::${property.camel_case})
                }
            % endfor
            PropertyDeclaration::Custom(ref name, _) => {
                PropertyDeclarationId::Custom(name)
            }
        }
    }

    fn with_variables_from_shorthand(&self, shorthand: ShorthandId) -> Option< &str> {
        match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(ref value) => match *value {
                    DeclaredValue::WithVariables { ref css, from_shorthand: Some(s), .. }
                    if s == shorthand => {
                        Some(&**css)
                    }
                    _ => None
                },
            % endfor
            PropertyDeclaration::Custom(..) => None,
        }
    }

    /// Return whether this is a pending-substitution value.
    /// https://drafts.csswg.org/css-variables/#variables-in-shorthands
    pub fn with_variables(&self) -> bool {
        match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(ref value) => match *value {
                    DeclaredValue::WithVariables { .. } => true,
                    _ => false,
                },
            % endfor
            PropertyDeclaration::Custom(_, ref value) => match *value {
                DeclaredValue::WithVariables { .. } => true,
                _ => false,
            }
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
          % for property in data.longhands:
              PropertyDeclaration::${property.camel_case}(ref value) => {
                  matches!(*value, DeclaredValue::WithVariables { .. })
              },
          % endfor
          PropertyDeclaration::Custom(..) => true
      }
    }

    /// The `in_keyframe_block` parameter controls this:
    ///
    /// https://drafts.csswg.org/css-animations/#keyframes
    /// > The <declaration-list> inside of <keyframe-block> accepts any CSS property
    /// > except those defined in this specification,
    /// > but does accept the `animation-play-state` property and interprets it specially.
    pub fn parse(id: PropertyId, context: &ParserContext, input: &mut Parser,
                 result_list: &mut Vec<PropertyDeclaration>,
                 in_keyframe_block: bool)
                 -> PropertyDeclarationParseResult {
        match id {
            PropertyId::Custom(name) => {
                let value = match input.try(|i| CSSWideKeyword::parse(context, i)) {
                    Ok(CSSWideKeyword::UnsetKeyword) => DeclaredValue::Unset,
                    Ok(CSSWideKeyword::InheritKeyword) => DeclaredValue::Inherit,
                    Ok(CSSWideKeyword::InitialKeyword) => DeclaredValue::Initial,
                    Err(()) => match ::custom_properties::SpecifiedValue::parse(context, input) {
                        Ok(value) => DeclaredValue::Value(value),
                        Err(()) => return PropertyDeclarationParseResult::InvalidValue,
                    }
                };
                result_list.push(PropertyDeclaration::Custom(name, value));
                return PropertyDeclarationParseResult::ValidOrIgnoredDeclaration;
            }
            PropertyId::Longhand(id) => match id {
            % for property in data.longhands:
                LonghandId::${property.camel_case} => {
                    % if not property.derived_from:
                        % if not property.allowed_in_keyframe_block:
                            if in_keyframe_block {
                                return PropertyDeclarationParseResult::AnimationPropertyInKeyframeBlock
                            }
                        % endif
                        % if property.internal:
                            if context.stylesheet_origin != Origin::UserAgent {
                                return PropertyDeclarationParseResult::UnknownProperty
                            }
                        % endif
                        % if property.experimental and product == "servo":
                            if !PREFS.get("${property.experimental}")
                                .as_boolean().unwrap_or(false) {
                                return PropertyDeclarationParseResult::ExperimentalProperty
                            }
                        % endif
                        match longhands::${property.ident}::parse_declared(context, input) {
                            Ok(value) => {
                                result_list.push(PropertyDeclaration::${property.camel_case}(value));
                                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                            },
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    % else:
                        PropertyDeclarationParseResult::UnknownProperty
                    % endif
                }
            % endfor
            },
            PropertyId::Shorthand(id) => match id {
            % for shorthand in data.shorthands:
                ShorthandId::${shorthand.camel_case} => {
                    % if not shorthand.allowed_in_keyframe_block:
                        if in_keyframe_block {
                            return PropertyDeclarationParseResult::AnimationPropertyInKeyframeBlock
                        }
                    % endif
                    % if shorthand.internal:
                        if context.stylesheet_origin != Origin::UserAgent {
                            return PropertyDeclarationParseResult::UnknownProperty
                        }
                    % endif
                    % if shorthand.experimental and product == "servo":
                        if !PREFS.get("${shorthand.experimental}")
                            .as_boolean().unwrap_or(false) {
                            return PropertyDeclarationParseResult::ExperimentalProperty
                        }
                    % endif
                    match input.try(|i| CSSWideKeyword::parse(context, i)) {
                        Ok(CSSWideKeyword::InheritKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(
                                    PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Inherit));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::InitialKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(
                                    PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Initial));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::UnsetKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Unset));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Err(()) => match shorthands::${shorthand.ident}::parse(context, input, result_list) {
                            Ok(()) => PropertyDeclarationParseResult::ValidOrIgnoredDeclaration,
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    }
                }
            % endfor
            }
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
            PropertyDeclaration::Custom(..) => false,
        }
    }
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
            % elif style_struct.name == "Outline":
                /// Whether the outline-width property is non-zero.
                #[inline]
                pub fn outline_has_nonzero_width(&self) -> bool {
                    self.outline_width != ::app_units::Au(0)
                }
            % elif style_struct.name == "Text":
                <% text_decoration_field = 'text_decoration' if product == 'servo' else 'text_decoration_line' %>
                /// Whether the text decoration has an underline.
                #[inline]
                pub fn has_underline(&self) -> bool {
                    self.${text_decoration_field}.underline
                }

                /// Whether the text decoration has an overline.
                #[inline]
                pub fn has_overline(&self) -> bool {
                    self.${text_decoration_field}.overline
                }

                /// Whether the text decoration has a line through.
                #[inline]
                pub fn has_line_through(&self) -> bool {
                    self.${text_decoration_field}.line_through
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
    shareable: bool,
    /// The writing mode of this computed values struct.
    pub writing_mode: WritingMode,
    /// The root element's computed font size.
    pub root_font_size: Au,
}

#[cfg(feature = "servo")]
impl ComputedValues {
    /// Construct a `ComputedValues` instance.
    pub fn new(custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
               shareable: bool,
               writing_mode: WritingMode,
               root_font_size: Au,
            % for style_struct in data.active_style_structs():
               ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
            % endfor
    ) -> Self {
        ComputedValues {
            custom_properties: custom_properties,
            shareable: shareable,
            writing_mode: writing_mode,
            root_font_size: root_font_size,
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

    /// Get the root font size.
    fn root_font_size(&self) -> Au { self.root_font_size }

    /// Set the root font size.
    fn set_root_font_size(&mut self, size: Au) { self.root_font_size = size }
    /// Set the writing mode for this style.
    pub fn set_writing_mode(&mut self, mode: WritingMode) { self.writing_mode = mode; }

    /// Whether the current style is multicolumn.
    #[inline]
    pub fn is_multicol(&self) -> bool {
        let style = self.get_column();
        match style.column_width {
            Either::First(_width) => true,
            Either::Second(_auto) => style.column_count.0.is_some(),
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
           effects.clip.0.is_some() {
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
    }
    match inheritedbox_style.clone_text_orientation() {
    % if product == "servo":
        computed_values::text_orientation::T::sideways_right => {},
        computed_values::text_orientation::T::sideways_left => {
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
        },
    % elif product == "gecko":
        // FIXME(bholley): Need to make sure these are correct when we add
        // full writing-mode support.
        computed_values::text_orientation::T::mixed => {},
        computed_values::text_orientation::T::upright => {},
    % endif
        computed_values::text_orientation::T::sideways => {
            if flags.intersects(logical_geometry::FLAG_VERTICAL_LR) {
                flags.insert(logical_geometry::FLAG_SIDEWAYS_LEFT);
            }
        },
    }
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
            shareable: true,
            writing_mode: WritingMode::empty(),
            root_font_size: longhands::font_size::get_initial_value(),
        };
    }
}

/// A per-longhand function that performs the CSS cascade for that longhand.
pub type CascadePropertyFn =
    extern "Rust" fn(declaration: &PropertyDeclaration,
                     inherited_style: &ComputedValues,
                     context: &mut computed::Context,
                     seen: &mut PropertyBitField,
                     cacheable: &mut bool,
                     cascade_info: &mut Option<<&mut CascadeInfo>,
                     error_reporter: &mut StdBox<ParseErrorReporter + Send>);

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
        /// Whether the `ComputedValues` structure to be constructed should be
        /// considered shareable.
        const SHAREABLE = 0x01,
        /// Whether to inherit all styles from the parent. If this flag is not
        /// present, non-inherited styles are reset to their initial values.
        const INHERIT_ALL = 0x02,
        /// Whether to skip any root element and flex/grid item display style
        /// fixup.
        const SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP = 0x04,
    }
}

/// Performs the CSS cascade, computing new styles for an element from its parent style.
///
/// The arguments are:
///
///   * `viewport_size`: The size of the initial viewport.
///
///   * `rule_node`: The rule node in the tree that represent the CSS rules that
///   matched.
///
///   * `parent_style`: The parent style, if applicable; if `None`, this is the root node.
///
/// Returns the computed values.
///   * `flags`: Various flags.
///
pub fn cascade(viewport_size: Size2D<Au>,
               rule_node: &StrongRuleNode,
               parent_style: Option<<&ComputedValues>,
               cascade_info: Option<<&mut CascadeInfo>,
               error_reporter: StdBox<ParseErrorReporter + Send>,
               flags: CascadeFlags)
               -> ComputedValues {
    let (is_root_element, inherited_style) = match parent_style {
        Some(parent_style) => (false, parent_style),
        None => (true, ComputedValues::initial_values()),
    };
    // Hold locks until after the apply_declarations() call returns.
    // Use filter_map because the root node has no style source.
    let lock_guards = rule_node.self_and_ancestors().filter_map(|node| {
        node.style_source().map(|source| (source.read(), node.importance()))
    }).collect::<Vec<_>>();
    let iter_declarations = || {
        lock_guards.iter().flat_map(|&(ref source, source_importance)| {
            source.declarations.iter()
            // Yield declarations later in source order (with more precedence) first.
            .rev()
            .filter_map(move |&(ref declaration, declaration_importance)| {
                if declaration_importance == source_importance {
                    Some(declaration)
                } else {
                    None
                }
            })
        })
    };
    apply_declarations(viewport_size,
                       is_root_element,
                       iter_declarations,
                       inherited_style,
                       cascade_info,
                       error_reporter,
                       None,
                       flags)
}

/// NOTE: This function expects the declaration with more priority to appear
/// first.
pub fn apply_declarations<'a, F, I>(viewport_size: Size2D<Au>,
                                    is_root_element: bool,
                                    iter_declarations: F,
                                    inherited_style: &ComputedValues,
                                    mut cascade_info: Option<<&mut CascadeInfo>,
                                    mut error_reporter: StdBox<ParseErrorReporter + Send>,
                                    font_metrics_provider: Option<<&FontMetricsProvider>,
                                    flags: CascadeFlags)
                                    -> ComputedValues
    where F: Fn() -> I,
          I: Iterator<Item = &'a PropertyDeclaration>,
{
    let inherited_custom_properties = inherited_style.custom_properties();
    let mut custom_properties = None;
    let mut seen_custom = HashSet::new();
    for declaration in iter_declarations() {
        match *declaration {
            PropertyDeclaration::Custom(ref name, ref value) => {
                ::custom_properties::cascade(
                    &mut custom_properties, &inherited_custom_properties,
                    &mut seen_custom, name, value)
            }
            _ => {}
        }
    }

    let custom_properties =
        ::custom_properties::finish_cascade(
            custom_properties, &inherited_custom_properties);

    let initial_values = ComputedValues::initial_values();

    let starting_style = if !flags.contains(INHERIT_ALL) {
        ComputedValues::new(custom_properties,
                            flags.contains(SHAREABLE),
                            WritingMode::empty(),
                            inherited_style.root_font_size(),
                            % for style_struct in data.active_style_structs():
                                % if style_struct.inherited:
                                    inherited_style.clone_${style_struct.name_lower}(),
                                % else:
                                    initial_values.clone_${style_struct.name_lower}(),
                                % endif
                            % endfor
                            )
    } else {
        ComputedValues::new(custom_properties,
                            flags.contains(SHAREABLE),
                            WritingMode::empty(),
                            inherited_style.root_font_size(),
                            % for style_struct in data.active_style_structs():
                                inherited_style.clone_${style_struct.name_lower}(),
                            % endfor
                            )
    };

    let mut context = computed::Context {
        is_root_element: is_root_element,
        viewport_size: viewport_size,
        inherited_style: inherited_style,
        style: starting_style,
        font_metrics_provider: font_metrics_provider,
    };

    // Set computed values, overwriting earlier declarations for the same
    // property.
    //
    // NB: The cacheable boolean is not used right now, but will be once we
    // start caching computed values in the rule nodes.
    let mut cacheable = true;
    let mut seen = PropertyBitField::new();

    // Declaration blocks are stored in increasing precedence order, we want
    // them in decreasing order here.
    //
    // We could (and used to) use a pattern match here, but that bloats this
    // function to over 100K of compiled code!
    //
    // To improve i-cache behavior, we outline the individual functions and use
    // virtual dispatch instead.
    % for category_to_cascade_now in ["early", "other"]:
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
            let is_early_property = matches!(*declaration,
                PropertyDeclaration::FontSize(_) |
                PropertyDeclaration::FontFamily(_) |
                PropertyDeclaration::Color(_) |
                PropertyDeclaration::Position(_) |
                PropertyDeclaration::Float(_) |
                PropertyDeclaration::TextDecoration${'' if product == 'servo' else 'Line'}(_) |
                PropertyDeclaration::WritingMode(_) |
                PropertyDeclaration::Direction(_) |
                PropertyDeclaration::TextOrientation(_)
            );
            if
                % if category_to_cascade_now == "early":
                    !
                % endif
                is_early_property
            {
                continue
            }

            let discriminant = longhand_id as usize;
            (CASCADE_PROPERTY[discriminant])(declaration,
                                             inherited_style,
                                             &mut context,
                                             &mut seen,
                                             &mut cacheable,
                                             &mut cascade_info,
                                             &mut error_reporter);
        }
        % if category_to_cascade_now == "early":
            let mode = get_writing_mode(context.style.get_inheritedbox());
            context.style.set_writing_mode(mode);
        % endif
    % endfor

    let mut style = context.style;

    let positioned = matches!(style.get_box().clone_position(),
        longhands::position::SpecifiedValue::absolute |
        longhands::position::SpecifiedValue::fixed);
    let floated = style.get_box().clone_float() != longhands::float::SpecifiedValue::none;
    // FIXME(heycam): We should look past any display:contents ancestors to
    // determine if we are a flex or grid item, but we don't have access to
    // grandparent or higher style here.
    let is_item = matches!(context.inherited_style.get_box().clone_display(),
        % if product == "gecko":
        computed_values::display::T::grid |
        % endif
        computed_values::display::T::flex);
    let (blockify_root, blockify_item) = match flags.contains(SKIP_ROOT_AND_ITEM_BASED_DISPLAY_FIXUP) {
        false => (is_root_element, is_item),
        true => (false, false),
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
            T::block | T::flex | T::list_item | T::table => None,
            % if product == "gecko":
            T::contents | T::grid | T::_webkit_box => None,
            % endif

            // Everything becomes block.
            _ => Some(T::block),
        };
        if let Some(computed_display) = computed_display {
            let box_ = style.mutate_box();
            box_.set_display(computed_display);
            % if product == "servo":
                box_.set__servo_display_for_hypothetical_box(if blockify_root || blockify_item {
                    computed_display
                } else {
                    specified_display
                });
            % endif
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

    % if "align-items" in data.longhands_by_name:
    {
        use computed_values::align_self::T as align_self;
        use computed_values::align_items::T as align_items;
        if style.get_position().clone_align_self() == computed_values::align_self::T::auto && !positioned {
            let self_align =
                match context.inherited_style.get_position().clone_align_items() {
                    align_items::stretch => align_self::stretch,
                    align_items::baseline => align_self::baseline,
                    align_items::flex_start => align_self::flex_start,
                    align_items::flex_end => align_self::flex_end,
                    align_items::center => align_self::center,
                    % if product == "gecko":
                        align_items::normal => align_self::normal,
                    % endif
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
        style.set_root_font_size(s);
    }

    if seen.get_font_style() || seen.get_font_weight() || seen.get_font_stretch() ||
       seen.get_font_family() {
        style.mutate_font().compute_font_hash();
    }

    style
}

/// Modifies the style for an anonymous flow so it resets all its non-inherited
/// style structs, and set their borders and outlines to zero.
///
/// Also, it gets a new display value, which is honored except when it's
/// `inline`.
#[cfg(feature = "servo")]
pub fn modify_style_for_anonymous_flow(style: &mut Arc<ComputedValues>,
                                       new_display_value: longhands::display::computed_value::T) {
    // The 'align-self' property needs some special treatment since
    // its value depends on the 'align-items' value of its parent.
    % if "align-items" in data.longhands_by_name:
        use computed_values::align_self::T as align_self;
        use computed_values::align_items::T as align_items;
        let self_align =
            match style.position.align_items {
                align_items::stretch => align_self::stretch,
                align_items::baseline => align_self::baseline,
                align_items::flex_start => align_self::flex_start,
                align_items::flex_end => align_self::flex_end,
                align_items::center => align_self::center,
            };
    % endif
    let inital_values = &*INITIAL_SERVO_VALUES;
    let mut style = Arc::make_mut(style);
    % for style_struct in data.active_style_structs():
    % if not style_struct.inherited:
        style.${style_struct.ident} = inital_values.clone_${style_struct.name_lower}();
    % endif
    % endfor
    % if "align-items" in data.longhands_by_name:
       let position = Arc::make_mut(&mut style.position);
       position.align_self = self_align;
    % endif
    if new_display_value != longhands::display::computed_value::T::inline {
        let new_box = Arc::make_mut(&mut style.box_);
        new_box.display = new_display_value;
    }
    let border = Arc::make_mut(&mut style.border);
    % for side in ["top", "right", "bottom", "left"]:
        // Like calling to_computed_value, which wouldn't type check.
        border.border_${side}_width = Au(0);
    % endfor
    // Initial value of outline-style is always none for anonymous box.
    let outline = Arc::make_mut(&mut style.outline);
    outline.outline_width = Au(0);
}

/// Alters the given style to accommodate replaced content. This is called in
/// flow construction. It handles cases like:
///
///     <div style="position: absolute">foo bar baz</div>
///
/// (in which `foo`, `bar`, and `baz` must not be absolutely-positioned) and
/// cases like `<sup>Foo</sup>` (in which the `vertical-align: top` style of
/// `sup` must not propagate down into `Foo`).
///
/// FIXME(#5625, pcwalton): It would probably be cleaner and faster to do this
/// in the cascade.
#[cfg(feature = "servo")]
#[inline]
pub fn modify_style_for_replaced_content(style: &mut Arc<ComputedValues>) {
    // Reset `position` to handle cases like `<div style="position: absolute">foo bar baz</div>`.
    if style.box_.display != longhands::display::computed_value::T::inline {
        let mut style = Arc::make_mut(style);
        Arc::make_mut(&mut style.box_).display = longhands::display::computed_value::T::inline;
        Arc::make_mut(&mut style.box_).position =
            longhands::position::computed_value::T::static_;
    }

    // Reset `vertical-align` to handle cases like `<sup>foo</sup>`.
    if style.box_.vertical_align != longhands::vertical_align::computed_value::T::baseline {
        let mut style = Arc::make_mut(style);
        Arc::make_mut(&mut style.box_).vertical_align =
            longhands::vertical_align::computed_value::T::baseline
    }

    // Reset margins.
    if style.margin.margin_top != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_left != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_bottom != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_right != computed::LengthOrPercentageOrAuto::Length(Au(0)) {
        let mut style = Arc::make_mut(style);
        let margin = Arc::make_mut(&mut style.margin);
        margin.margin_top = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_left = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_bottom = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_right = computed::LengthOrPercentageOrAuto::Length(Au(0));
    }
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

/// Adjusts the `position` property as necessary for the outer fragment wrapper
/// of an inline-block.
#[cfg(feature = "servo")]
#[inline]
pub fn modify_style_for_outer_inline_block_fragment(style: &mut Arc<ComputedValues>) {
    let mut style = Arc::make_mut(style);
    let box_style = Arc::make_mut(&mut style.box_);
    box_style.position = longhands::position::computed_value::T::static_
}

/// Adjusts the `position` and `padding` properties as necessary to account for
/// text.
///
/// Text is never directly relatively positioned; it's always contained within
/// an element that is itself relatively positioned.
#[cfg(feature = "servo")]
#[inline]
pub fn modify_style_for_text(style: &mut Arc<ComputedValues>) {
    if style.box_.position == longhands::position::computed_value::T::relative {
        // We leave the `position` property set to `relative` so that we'll still establish a
        // containing block if needed. But we reset all position offsets to `auto`.
        let mut style = Arc::make_mut(style);
        let mut position = Arc::make_mut(&mut style.position);
        position.top = computed::LengthOrPercentageOrAuto::Auto;
        position.right = computed::LengthOrPercentageOrAuto::Auto;
        position.bottom = computed::LengthOrPercentageOrAuto::Auto;
        position.left = computed::LengthOrPercentageOrAuto::Auto;
    }

    if style.padding.padding_top != computed::LengthOrPercentage::Length(Au(0)) ||
            style.padding.padding_right != computed::LengthOrPercentage::Length(Au(0)) ||
            style.padding.padding_bottom != computed::LengthOrPercentage::Length(Au(0)) ||
            style.padding.padding_left != computed::LengthOrPercentage::Length(Au(0)) {
        let mut style = Arc::make_mut(style);
        let mut padding = Arc::make_mut(&mut style.padding);
        padding.padding_top = computed::LengthOrPercentage::Length(Au(0));
        padding.padding_right = computed::LengthOrPercentage::Length(Au(0));
        padding.padding_bottom = computed::LengthOrPercentage::Length(Au(0));
        padding.padding_left = computed::LengthOrPercentage::Length(Au(0));
    }

    if style.effects.opacity != 1.0 {
        let mut style = Arc::make_mut(style);
        let mut effects = Arc::make_mut(&mut style.effects);
        effects.opacity = 1.0;
    }
}

/// Adjusts the `clip` property so that an inline absolute hypothetical fragment
/// doesn't clip its children.
#[cfg(feature = "servo")]
pub fn modify_style_for_inline_absolute_hypothetical_fragment(style: &mut Arc<ComputedValues>) {
    if style.get_effects().clip.0.is_some() {
        let mut style = Arc::make_mut(style);
        let effects_style = Arc::make_mut(&mut style.effects);
        effects_style.clip.0 = None
    }
}

#[macro_export]
macro_rules! css_properties_accessors {
    ($macro_name: ident) => {
        $macro_name! {
            % for kind, props in [("Longhand", data.longhands), ("Shorthand", data.shorthands)]:
                % for property in props:
                    % if not property.derived_from and not property.internal:
                        % if '-' in property.name:
                            [${property.ident.capitalize()}, Set${property.ident.capitalize()},
                             PropertyId::${kind}(${kind}Id::${property.camel_case})],
                        % endif
                        [${property.camel_case}, Set${property.camel_case},
                         PropertyId::${kind}(${kind}Id::${property.camel_case})],
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
