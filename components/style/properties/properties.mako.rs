/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

// Please note that valid Rust syntax may be mangled by the Mako parser.
// For example, Vec<&Foo> will be mangled as Vec&Foo>. To work around these issues, the code
// can be escaped. In the above example, Vec<<&Foo> or Vec< &Foo> achieves the desired result of Vec<&Foo>.

<%namespace name="helpers" file="/helpers.mako.rs" />

use std::ascii::AsciiExt;
use std::boxed::Box as StdBox;
use std::collections::HashSet;
use std::fmt::{self, Write};
use std::sync::Arc;

use Atom;
use app_units::Au;
#[cfg(feature = "servo")] use cssparser::{Color as CSSParserColor, RGBA};
use cssparser::{Parser, TokenSerializationType};
use error_reporting::ParseErrorReporter;
use url::Url;
#[cfg(feature = "servo")] use euclid::side_offsets::SideOffsets2D;
use euclid::size::Size2D;
use computed_values;
use font_metrics::FontMetricsProvider;
#[cfg(feature = "servo")] use logical_geometry::{LogicalMargin, PhysicalSide};
use logical_geometry::WritingMode;
use parser::{Parse, ParserContext, ParserContextExtraData};
use style_traits::ToCss;
use stylesheets::Origin;
#[cfg(feature = "servo")] use values::Either;
use values::{HasViewportPercentage, computed};
use cascade_info::CascadeInfo;
use rule_tree::StrongRuleNode;
#[cfg(feature = "servo")] use values::specified::BorderStyle;

use self::property_bit_field::PropertyBitField;
pub use self::declaration_block::*;

<%!
    from data import Method, Keyword, to_rust_ident
    import os.path
%>

#[path="${repr(os.path.join(os.path.dirname(__file__), 'declaration_block.rs'))[1:-1]}"]
pub mod declaration_block;

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

pub mod shorthands {
    use cssparser::Parser;
    use parser::{Parse, ParserContext};
    use values::specified;

    pub fn parse_four_sides<F, T>(input: &mut Parser, parse_one: F) -> Result<(T, T, T, T), ()>
    where F: Fn(&mut Parser) -> Result<T, ()>, F: Copy, T: Clone {
        // zero or more than four values is invalid.
        // one value sets them all
        // two values set (top, bottom) and (left, right)
        // three values set top, (left, right) and bottom
        // four values set them in order
        let top = try!(parse_one(input));
        let right;
        let bottom;
        let left;
        match input.try(parse_one) {
            Err(()) => {
                right = top.clone();
                bottom = top.clone();
                left = top.clone();
            }
            Ok(value) => {
                right = value;
                match input.try(parse_one) {
                    Err(()) => {
                        bottom = top.clone();
                        left = right.clone();
                    }
                    Ok(value) => {
                        bottom = value;
                        match input.try(parse_one) {
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

pub mod animated_properties {
    <%include file="/helpers/animated_properties.mako.rs" />
}


// TODO(SimonSapin): Convert this to a syntax extension rather than a Mako template.
// Maybe submit for inclusion in libstd?
mod property_bit_field {
    use logical_geometry::WritingMode;

    pub struct PropertyBitField {
        storage: [u32; (${len(data.longhands)} - 1 + 32) / 32]
    }

    impl PropertyBitField {
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
                #[allow(non_snake_case)]
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
                #[allow(non_snake_case)]
                pub fn get_physical_${property.ident}(&self, wm: WritingMode) -> bool {
                    <%helpers:logical_setter_helper name="${property.name}">
                        <%def name="inner(physical_ident)">
                            self.get_${physical_ident}()
                        </%def>
                    </%helpers:logical_setter_helper>
                }
                #[allow(non_snake_case)]
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
                base_url: &Url,
                from_shorthand: Option<Shorthand>,
                custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
                f: F,
                error_reporter: &mut StdBox<ParseErrorReporter + Send>,
                extra_data: ParserContextExtraData)
                where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>) {
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
                                    Some(Shorthand::${shorthand.camel_case}) => {
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

/// Only keep the "winning" declaration for any given property, by importance then source order.
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CSSWideKeyword {
    InitialKeyword,
    InheritKeyword,
    UnsetKeyword,
}

impl Parse for CSSWideKeyword {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "initial" => Ok(CSSWideKeyword::InitialKeyword),
            "inherit" => Ok(CSSWideKeyword::InheritKeyword),
            "unset" => Ok(CSSWideKeyword::UnsetKeyword),
            _ => Err(())
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Shorthand {
    % for property in data.shorthands:
        ${property.camel_case},
    % endfor
}

impl Shorthand {
    pub fn from_name(name: &str) -> Option<Shorthand> {
        match_ignore_ascii_case! { name,
            % for property in data.shorthands:
                "${property.name}" => Some(Shorthand::${property.camel_case}),
            % endfor
            _ => None
        }
    }

    pub fn name(&self) -> &'static str {
        match *self {
            % for property in data.shorthands:
                Shorthand::${property.camel_case} => "${property.name}",
            % endfor
        }
    }

    pub fn longhands(&self) -> &'static [&'static str] {
        % for property in data.shorthands:
            static ${property.ident.upper()}: &'static [&'static str] = &[
                % for sub in property.sub_properties:
                    "${sub.name}",
                % endfor
            ];
        % endfor
        match *self {
            % for property in data.shorthands:
                Shorthand::${property.camel_case} => ${property.ident.upper()},
            % endfor
        }
    }

    pub fn longhands_to_css<'a, W, I>(&self, declarations: I, dest: &mut W) -> fmt::Result
        where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
        match *self {
            % for property in data.shorthands:
                Shorthand::${property.camel_case} => {
                    match shorthands::${property.ident}::LonghandsToSerialize::from_iter(declarations) {
                        Ok(longhands) => longhands.to_css(dest),
                        Err(_) => Err(fmt::Error)
                    }
                },
            % endfor
        }
    }

    /// Serializes possible shorthand name with value to input buffer given a list of longhand declarations.
    /// On success, returns true if shorthand value is written and false if no shorthand value is present.
    pub fn serialize_shorthand_to_buffer<'a, W, I>(self,
                                                   dest: &mut W,
                                                   declarations: I,
                                                   is_first_serialization: &mut bool)
                                                   -> Result<bool, fmt::Error>
    where W: Write, I: IntoIterator<Item=&'a PropertyDeclaration>, I::IntoIter: Clone {
        match self.get_shorthand_appendable_value(declarations) {
            None => Ok(false),
            Some(appendable_value) => {
                let property_name = self.name();

                append_serialization(
                    dest,
                    property_name,
                    appendable_value,
                    Importance::Normal,
                    is_first_serialization
                ).and_then(|_| Ok(true))
            }
        }
    }

    fn get_shorthand_appendable_value<'a, I>(self, declarations: I)
                                             -> Option<AppendableValue<'a, I::IntoIter>>
        where I: IntoIterator<Item=&'a PropertyDeclaration>, I::IntoIter: Clone {
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
               else {
                   return None;
               }
            }

            if !declarations3.any(|d| d.with_variables()) {
                return Some(AppendableValue::DeclarationsForShorthand(self, declarations));
            }

            None
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum DeclaredValue<T> {
    Value(T),
    WithVariables {
        css: String,
        first_token_type: TokenSerializationType,
        base_url: Url,
        from_shorthand: Option<Shorthand>,
    },
    Initial,
    Inherit,
    // There is no Unset variant here.
    // The 'unset' keyword is represented as either Initial or Inherit,
    // depending on whether the property is inherited.
}

impl<T: HasViewportPercentage> HasViewportPercentage for DeclaredValue<T> {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            DeclaredValue::Value(ref v)
                => v.has_viewport_percentage(),
            DeclaredValue::WithVariables { .. }
                => panic!("DeclaredValue::has_viewport_percentage without resolving variables!"),
            DeclaredValue::Initial |
            DeclaredValue::Inherit => false,
        }
    }
}

impl<T: ToCss> ToCss for DeclaredValue<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            DeclaredValue::Value(ref inner) => inner.to_css(dest),
            DeclaredValue::WithVariables { ref css, from_shorthand: None, .. } => {
                dest.write_str(css)
            }
            // https://drafts.csswg.org/css-variables/#variables-in-shorthands
            DeclaredValue::WithVariables { .. } => Ok(()),
            DeclaredValue::Initial => dest.write_str("initial"),
            DeclaredValue::Inherit => dest.write_str("inherit"),
        }
    }
}

#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum PropertyDeclaration {
    % for property in data.longhands:
        ${property.camel_case}(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
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

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PropertyDeclarationParseResult {
    UnknownProperty,
    ExperimentalProperty,
    InvalidValue,
    AnimationPropertyInKeyframeBlock,
    ValidOrIgnoredDeclaration,
}

#[derive(Eq, PartialEq, Clone)]
pub enum PropertyDeclarationName {
    Longhand(&'static str),
    Custom(::custom_properties::Name),
    Internal
}

impl PropertyDeclarationName {
    pub fn eq_str_ignore_ascii_case(&self, other: &str) -> bool {
        match *self {
            PropertyDeclarationName::Longhand(s) => s.eq_ignore_ascii_case(other),
            PropertyDeclarationName::Custom(ref n) => n.eq_str_ignore_ascii_case(other),
            PropertyDeclarationName::Internal => false
        }
    }
}

impl PartialEq<str> for PropertyDeclarationName {
    fn eq(&self, other: &str) -> bool {
        match *self {
            PropertyDeclarationName::Longhand(n) => n == other,
            PropertyDeclarationName::Custom(ref n) => {
                n.with_str(|s| ::custom_properties::parse_name(other) == Ok(s))
            }
            PropertyDeclarationName::Internal => false,
        }
    }
}

impl fmt::Display for PropertyDeclarationName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PropertyDeclarationName::Longhand(n) => f.write_str(n),
            PropertyDeclarationName::Custom(ref n) => {
                try!(f.write_str("--"));
                n.with_str(|s| f.write_str(s))
            }
            PropertyDeclarationName::Internal => Ok(()),
        }
    }
}

impl fmt::Debug for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}: ", self.name()));
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
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
    pub fn name(&self) -> PropertyDeclarationName {
        match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(..) =>
                % if not property.derived_from:
                    PropertyDeclarationName::Longhand("${property.name}"),
                % else:
                    PropertyDeclarationName::Internal,
                % endif
            % endfor
            PropertyDeclaration::Custom(ref name, _) => {
                PropertyDeclarationName::Custom(name.clone())
            }
        }
    }

    #[inline]
    pub fn discriminant_value(&self) -> usize {
        match *self {
            % for i, property in enumerate(data.longhands):
                PropertyDeclaration::${property.camel_case}(..) => ${i},
            % endfor
            PropertyDeclaration::Custom(..) => ${len(data.longhands)}
        }
    }

    pub fn value(&self) -> String {
        let mut value = String::new();
        if let Err(_) = self.to_css(&mut value) {
            panic!("unsupported property declaration: {}", self.name());
        }

        value
    }

    /// If this is a pending-substitution value from the given shorthand, return that value
    // Extra space here because < seems to be removed by Mako when immediately followed by &.
    //                                                                          ↓
    pub fn with_variables_from_shorthand(&self, shorthand: Shorthand) -> Option< &str> {
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

    /// Return whether the value is stored as it was in the CSS source, preserving whitespace
    /// (as opposed to being parsed into a more abstract data structure).
    /// This is the case of custom properties and values that contain unsubstituted variables.
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

    pub fn matches(&self, name: &str) -> bool {
        match *self {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(..) =>
                % if not property.derived_from:
                    name.eq_ignore_ascii_case("${property.name}"),
                % else:
                    false,
                % endif
            % endfor
            PropertyDeclaration::Custom(ref declaration_name, _) => {
                declaration_name.with_str(|s| ::custom_properties::parse_name(name) == Ok(s))
            }
        }
    }

    /// The `in_keyframe_block` parameter controls this:
    ///
    /// https://drafts.csswg.org/css-animations/#keyframes
    /// > The <declaration-list> inside of <keyframe-block> accepts any CSS property
    /// > except those defined in this specification,
    /// > but does accept the `animation-play-state` property and interprets it specially.
    pub fn parse(name: &str, context: &ParserContext, input: &mut Parser,
                 result_list: &mut Vec<PropertyDeclaration>,
                 in_keyframe_block: bool)
                 -> PropertyDeclarationParseResult {
        if let Ok(name) = ::custom_properties::parse_name(name) {
            let value = match input.try(CSSWideKeyword::parse) {
                Ok(CSSWideKeyword::UnsetKeyword) |  // Custom properties are alawys inherited
                Ok(CSSWideKeyword::InheritKeyword) => DeclaredValue::Inherit,
                Ok(CSSWideKeyword::InitialKeyword) => DeclaredValue::Initial,
                Err(()) => match ::custom_properties::SpecifiedValue::parse(input) {
                    Ok(value) => DeclaredValue::Value(value),
                    Err(()) => return PropertyDeclarationParseResult::InvalidValue,
                }
            };
            result_list.push(PropertyDeclaration::Custom(Atom::from(name), value));
            return PropertyDeclarationParseResult::ValidOrIgnoredDeclaration;
        }
        match_ignore_ascii_case! { name,
            % for property in data.longhands:
                % if not property.derived_from:
                    "${property.name}" => {
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
                            if !::util::prefs::PREFS.get("${property.experimental}")
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
                    },
                % else:
                    "${property.name}" => PropertyDeclarationParseResult::UnknownProperty,
                % endif
            % endfor
            % for shorthand in data.shorthands:
                "${shorthand.name}" => {
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
                        if !::util::prefs::PREFS.get("${shorthand.experimental}")
                            .as_boolean().unwrap_or(false) {
                            return PropertyDeclarationParseResult::ExperimentalProperty
                        }
                    % endif
                    match input.try(CSSWideKeyword::parse) {
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
                                    DeclaredValue::${"Inherit" if sub_property.style_struct.inherited else "Initial"}
                                ));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Err(()) => match shorthands::${shorthand.ident}::parse(context, input, result_list) {
                            Ok(()) => PropertyDeclarationParseResult::ValidOrIgnoredDeclaration,
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    }
                },
            % endfor

            _ => {
                if cfg!(all(debug_assertions, feature = "gecko")) && !name.starts_with('-') {
                    println!("stylo: Unimplemented property setter: {}", name);
                }
                PropertyDeclarationParseResult::UnknownProperty
            }
        }
    }

    pub fn shorthands(&self) -> &'static [Shorthand] {
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
            static ${property.ident.upper()}: &'static [Shorthand] = &[
                % for shorthand in longhand_to_shorthand_map.get(property.ident, []):
                    Shorthand::${shorthand},
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
        pub struct ${style_struct.name} {
            % for longhand in style_struct.longhands:
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
            % if style_struct.name == "Font":
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
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T) {
                        self.${longhand.ident} = v;
                    }
                    #[allow(non_snake_case)]
                    #[inline]
                    pub fn copy_${longhand.ident}_from(&mut self, other: &Self) {
                        self.${longhand.ident} = other.${longhand.ident}.clone();
                    }
                    % if longhand.need_clone:
                        #[allow(non_snake_case)]
                        #[inline]
                        pub fn clone_${longhand.ident}(&self) -> longhands::${longhand.ident}::computed_value::T {
                            self.${longhand.ident}.clone()
                        }
                    % endif
                % endif
                % if longhand.need_index:
                    #[allow(non_snake_case)]
                    pub fn ${longhand.ident}_count(&self) -> usize {
                        self.${longhand.ident}.0.len()
                    }

                    #[allow(non_snake_case)]
                    pub fn ${longhand.ident}_at(&self, index: usize)
                        -> longhands::${longhand.ident}::computed_value::SingleComputedValue {
                        self.${longhand.ident}.0[index].clone()
                    }
                % endif
            % endfor
            % if style_struct.name == "Border":
                % for side in ["top", "right", "bottom", "left"]:
                    #[allow(non_snake_case)]
                    pub fn border_${side}_has_nonzero_width(&self) -> bool {
                        self.border_${side}_width != ::app_units::Au(0)
                    }
                % endfor
            % elif style_struct.name == "Font":
                pub fn compute_font_hash(&mut self) {
                    // Corresponds to the fields in `gfx::font_template::FontTemplateDescriptor`.
                    let mut hasher: FnvHasher = Default::default();
                    hasher.write_u16(self.font_weight as u16);
                    self.font_stretch.hash(&mut hasher);
                    self.font_family.hash(&mut hasher);
                    self.hash = hasher.finish()
                }
            % elif style_struct.name == "Outline":
                #[inline]
                pub fn outline_has_nonzero_width(&self) -> bool {
                    self.outline_width != ::app_units::Au(0)
                }
            % elif style_struct.name == "Text":
                <% text_decoration_field = 'text_decoration' if product == 'servo' else 'text_decoration_line' %>
                #[inline]
                pub fn has_underline(&self) -> bool {
                    self.${text_decoration_field}.underline
                }
                #[inline]
                pub fn has_overline(&self) -> bool {
                    self.${text_decoration_field}.overline
                }
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
                #[allow(non_snake_case)]
                #[inline]
                pub fn ${longhand.ident}_iter(&self) -> ${longhand.camel_case}Iter {
                    ${longhand.camel_case}Iter {
                        style_struct: self,
                        current: 0,
                        max: self.${longhand.ident}_count(),
                    }
                }

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

#[cfg(feature = "servo")]
pub type ServoComputedValues = ComputedValues;

#[cfg(feature = "servo")]
#[cfg_attr(feature = "servo", derive(Clone, Debug))]
pub struct ComputedValues {
    % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor
    custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

#[cfg(feature = "servo")]
impl ComputedValues {
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

    pub fn initial_values() -> &'static Self { &*INITIAL_SERVO_VALUES }

    #[inline]
    pub fn do_cascade_property<F: FnOnce(&[CascadePropertyFn])>(f: F) {
        f(&CASCADE_PROPERTY)
    }

    % for style_struct in data.active_style_structs():
        #[inline]
        pub fn clone_${style_struct.name_lower}(&self) -> Arc<style_structs::${style_struct.name}> {
                self.${style_struct.ident}.clone()
            }
        #[inline]
        pub fn get_${style_struct.name_lower}(&self) -> &style_structs::${style_struct.name} {
            &self.${style_struct.ident}
        }
        #[inline]
        pub fn mutate_${style_struct.name_lower}(&mut self) -> &mut style_structs::${style_struct.name} {
            Arc::make_mut(&mut self.${style_struct.ident})
        }
    % endfor

    // Cloning the Arc here is fine because it only happens in the case where we have custom
    // properties, and those are both rare and expensive.
    pub fn custom_properties(&self) -> Option<Arc<::custom_properties::ComputedValuesMap>> {
        self.custom_properties.as_ref().map(|x| x.clone())
    }

    pub fn root_font_size(&self) -> Au { self.root_font_size }
    pub fn set_root_font_size(&mut self, size: Au) { self.root_font_size = size }
    pub fn set_writing_mode(&mut self, mode: WritingMode) { self.writing_mode = mode; }

    #[inline]
    pub fn is_multicol(&self) -> bool {
        let style = self.get_column();
        style.column_count.0.is_some() || style.column_width.0.is_some()
    }

    /// Resolves the currentColor keyword.
    /// Any color value form computed values (except for the 'color' property itself)
    /// should go through this method.
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

    #[inline]
    pub fn content_inline_size(&self) -> computed::LengthOrPercentageOrAuto {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() {
            position_style.height
        } else {
            position_style.width
        }
    }

    #[inline]
    pub fn content_block_size(&self) -> computed::LengthOrPercentageOrAuto {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.width } else { position_style.height }
    }

    #[inline]
    pub fn min_inline_size(&self) -> computed::LengthOrPercentage {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.min_height } else { position_style.min_width }
    }

    #[inline]
    pub fn min_block_size(&self) -> computed::LengthOrPercentage {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.min_width } else { position_style.min_height }
    }

    #[inline]
    pub fn max_inline_size(&self) -> computed::LengthOrPercentageOrNone {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.max_height } else { position_style.max_width }
    }

    #[inline]
    pub fn max_block_size(&self) -> computed::LengthOrPercentageOrNone {
        let position_style = self.get_position();
        if self.writing_mode.is_vertical() { position_style.max_width } else { position_style.max_height }
    }

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

    #[inline]
    pub fn logical_border_width(&self) -> LogicalMargin<Au> {
        self.border_width_for_writing_mode(self.writing_mode)
    }

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

    #[inline]
    pub fn get_font_arc(&self) -> Arc<style_structs::Font> {
        self.font.clone()
    }

    // http://dev.w3.org/csswg/css-transforms/#grouping-property-values
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

        if effects.transform_style == transform_style::T::auto {
            if box_.transform.0.is_some() {
                return transform_style::T::flat;
            }
            if let Either::First(ref _length) = effects.perspective {
                return transform_style::T::flat;
            }
        }

        // Return the computed value if not overridden by the above exceptions
        effects.transform_style
    }

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

    pub fn computed_value_to_string(&self, name: &str) -> Result<String, ()> {
        match name {
            % for style_struct in data.active_style_structs():
                % for longhand in style_struct.longhands:
                "${longhand.name}" => Ok(self.${style_struct.ident}.${longhand.ident}.to_css_string()),
                % endfor
            % endfor
            _ => {
                let name = try!(::custom_properties::parse_name(name));
                let map = try!(self.custom_properties.as_ref().ok_or(()));
                let value = try!(map.get(&Atom::from(name)).ok_or(()));
                Ok(value.to_css_string())
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

pub type CascadePropertyFn =
    extern "Rust" fn(declaration: &PropertyDeclaration,
                     inherited_style: &ComputedValues,
                     context: &mut computed::Context,
                     seen: &mut PropertyBitField,
                     cacheable: &mut bool,
                     cascade_info: &mut Option<<&mut CascadeInfo>,
                     error_reporter: &mut StdBox<ParseErrorReporter + Send>);

#[cfg(feature = "servo")]
static CASCADE_PROPERTY: [CascadePropertyFn; ${len(data.longhands)}] = [
    % for property in data.longhands:
        longhands::${property.ident}::cascade_property,
    % endfor
];

bitflags! {
    pub flags CascadeFlags: u8 {
        /// Whether the `ComputedValues` structure to be constructed should be considered
        /// shareable.
        const SHAREABLE = 0x01,
        /// Whether to inherit all styles from the parent. If this flag is not present,
        /// non-inherited styles are reset to their initial values.
        const INHERIT_ALL = 0x02,
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
    where F: Fn() -> I, I: Iterator<Item = &'a PropertyDeclaration>
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
    ComputedValues::do_cascade_property(|cascade_property| {
        % for category_to_cascade_now in ["early", "other"]:
            for declaration in iter_declarations() {
                if let PropertyDeclaration::Custom(..) = *declaration {
                    continue
                }
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

                let discriminant = declaration.discriminant_value();
                (cascade_property[discriminant])(declaration,
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
    });

    let mut style = context.style;

    let positioned = matches!(style.get_box().clone_position(),
        longhands::position::SpecifiedValue::absolute |
        longhands::position::SpecifiedValue::fixed);
    let floated = style.get_box().clone_float() != longhands::float::SpecifiedValue::none;
    let is_flex_item =
        context.inherited_style.get_box().clone_display() == computed_values::display::T::flex;
    if positioned || floated || is_root_element || is_flex_item {
        use computed_values::display::T;

        let specified_display = style.get_box().clone_display();
        let computed_display = match specified_display {
            T::inline_table => {
                Some(T::table)
            }
            T::inline | T::inline_block |
            T::table_row_group | T::table_column |
            T::table_column_group | T::table_header_group |
            T::table_footer_group | T::table_row | T::table_cell |
            T::table_caption => {
                Some(T::block)
            }
            _ => None
        };
        if let Some(computed_display) = computed_display {
            let box_ = style.mutate_box();
            box_.set_display(computed_display);
            % if product == "servo":
                box_.set__servo_display_for_hypothetical_box(if is_root_element || is_flex_item {
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

/// Alters the given style to accommodate replaced content. This is called in flow construction. It
/// handles cases like `<div style="position: absolute">foo bar baz</div>` (in which `foo`, `bar`,
/// and `baz` must not be absolutely-positioned) and cases like `<sup>Foo</sup>` (in which the
/// `vertical-align: top` style of `sup` must not propagate down into `Foo`).
///
/// FIXME(#5625, pcwalton): It would probably be cleaner and faster to do this in the cascade.
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

/// Adjusts borders as appropriate to account for a fragment's status as the first or last fragment
/// within the range of an element.
///
/// Specifically, this function sets border widths to zero on the sides for which the fragment is
/// not outermost.
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

/// Adjusts the `position` property as necessary for the outer fragment wrapper of an inline-block.
#[cfg(feature = "servo")]
#[inline]
pub fn modify_style_for_outer_inline_block_fragment(style: &mut Arc<ComputedValues>) {
    let mut style = Arc::make_mut(style);
    let box_style = Arc::make_mut(&mut style.box_);
    box_style.position = longhands::position::computed_value::T::static_
}

/// Adjusts the `position` and `padding` properties as necessary to account for text.
///
/// Text is never directly relatively positioned; it's always contained within an element that is
/// itself relatively positioned.
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

/// Adjusts the `clip` property so that an inline absolute hypothetical fragment doesn't clip its
/// children.
#[cfg(feature = "servo")]
pub fn modify_style_for_inline_absolute_hypothetical_fragment(style: &mut Arc<ComputedValues>) {
    if style.get_effects().clip.0.is_some() {
        let mut style = Arc::make_mut(style);
        let effects_style = Arc::make_mut(&mut style.effects);
        effects_style.clip.0 = None
    }
}

// FIXME: https://github.com/w3c/csswg-drafts/issues/580
pub fn is_supported_property(property: &str) -> bool {
    match_ignore_ascii_case! { property,
        % for property in data.shorthands + data.longhands:
            "${property.name}" => true,
        % endfor
        _ => property.starts_with("--")
    }
}

#[macro_export]
macro_rules! css_properties_accessors {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in data.shorthands + data.longhands:
                % if not property.derived_from and not property.internal:
                    % if '-' in property.name:
                        [${property.ident.capitalize()}, Set${property.ident.capitalize()}, "${property.name}"],
                    % endif
                    % if property != data.longhands[-1]:
                        [${property.camel_case}, Set${property.camel_case}, "${property.name}"],
                    % else:
                        [${property.camel_case}, Set${property.camel_case}, "${property.name}"]
                    % endif
                % endif
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
