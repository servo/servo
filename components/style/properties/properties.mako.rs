/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

// Please note that valid Rust syntax may be mangled by the Mako parser.
// For example, Vec<&Foo> will be mangled as Vec&Foo>. To work around these issues, the code
// can be escaped. In the above example, Vec<<&Foo> achieves the desired result of Vec<&Foo>.

<%namespace name="helpers" file="/helpers.mako.rs" />

use std::ascii::AsciiExt;
use std::boxed::Box as StdBox;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Write;
use std::intrinsics;
use std::mem;
use std::sync::Arc;

use app_units::Au;
use cssparser::Color as CSSParserColor;
use cssparser::{Parser, RGBA, AtRuleParser, DeclarationParser, Delimiter,
                DeclarationListParser, parse_important, ToCss, TokenSerializationType};
use error_reporting::ParseErrorReporter;
use url::Url;
use euclid::SideOffsets2D;
use euclid::size::Size2D;
use string_cache::Atom;
use computed_values;
use logical_geometry::{LogicalMargin, PhysicalSide, WritingMode};
use parser::{ParserContext, log_css_error};
use selectors::matching::DeclarationBlock;
use stylesheets::Origin;
use values::AuExtensionMethods;
use values::computed::{self, TContext, ToComputedValue};
use values::specified::BorderStyle;

use self::property_bit_field::PropertyBitField;

<%!
    from data import Method, Keyword, to_rust_ident
%>

pub mod longhands {
    use cssparser::Parser;
    use parser::ParserContext;
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
    use parser::ParserContext;
    use values::specified;

    fn parse_four_sides<F, T>(input: &mut Parser, parse_one: F) -> Result<(T, T, T, T), ()>
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

    <%include file="/shorthand/background.mako.rs" />
    <%include file="/shorthand/border.mako.rs" />
    <%include file="/shorthand/box.mako.rs" />
    <%include file="/shorthand/column.mako.rs" />
    <%include file="/shorthand/font.mako.rs" />
    <%include file="/shorthand/inherited_text.mako.rs" />
    <%include file="/shorthand/list.mako.rs" />
    <%include file="/shorthand/margin.mako.rs" />
    <%include file="/shorthand/outline.mako.rs" />
    <%include file="/shorthand/padding.mako.rs" />
}


// TODO(SimonSapin): Convert this to a syntax extension rather than a Mako template.
// Maybe submit for inclusion in libstd?
mod property_bit_field {

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
                substitute_variables_${property.ident}_slow(css,
                                                            first_token_type,
                                                            base_url,
                                                            from_shorthand,
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
                base_url: &Url,
                from_shorthand: Option<Shorthand>,
                custom_properties: &Option<Arc<::custom_properties::ComputedValuesMap>>,
                f: F,
                error_reporter: &mut StdBox<ParseErrorReporter + Send>)
                where F: FnOnce(&DeclaredValue<longhands::${property.ident}::SpecifiedValue>) {
            f(&
                ::custom_properties::substitute(css, first_token_type, custom_properties)
                .and_then(|css| {
                    // As of this writing, only the base URL is used for property values:
                    //
                    // FIXME(pcwalton): Cloning the error reporter is slow! But so are custom
                    // properties, so whatever...
                    let context = ParserContext::new(
                        ::stylesheets::Origin::Author, base_url, (*error_reporter).clone());
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


use std::iter::{Iterator, Chain, Zip, Rev, Repeat, repeat};
use std::slice;
/// Overridden declarations are skipped.

// FIXME (https://github.com/servo/servo/issues/3426)
#[derive(Debug, PartialEq, HeapSizeOf)]
pub struct PropertyDeclarationBlock {
    #[ignore_heap_size_of = "#7038"]
    pub important: Arc<Vec<PropertyDeclaration>>,
    #[ignore_heap_size_of = "#7038"]
    pub normal: Arc<Vec<PropertyDeclaration>>,
}

impl PropertyDeclarationBlock {
    /// Provides an iterator of all declarations, with indication of !important value
    pub fn declarations(&self) -> Chain<
        Zip<Rev<slice::Iter<PropertyDeclaration>>, Repeat<bool>>,
        Zip<Rev<slice::Iter<PropertyDeclaration>>, Repeat<bool>>
    > {
        // Declarations are stored in reverse order.
        let normal = self.normal.iter().rev().zip(repeat(false));
        let important = self.important.iter().rev().zip(repeat(true));
        normal.chain(important)
    }
}

impl ToCss for PropertyDeclarationBlock {
    // https://drafts.csswg.org/cssom/#serialize-a-css-declaration-block
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut is_first_serialization = true; // trailing serializations should have a prepended space

        // Step 1 -> dest = result list

        // Step 2
        let mut already_serialized = Vec::new();

        // Step 3
        for (declaration, important) in self.declarations() {
            // Step 3.1
            let property = declaration.name();

            // Step 3.2
            if already_serialized.contains(&property) {
                continue;
            }

            // Step 3.3
            let shorthands = declaration.shorthands();
            if !shorthands.is_empty() {

                // Step 3.3.1
                let mut longhands = self.declarations()
                    .filter(|d| !already_serialized.contains(&d.0.name()))
                    .collect::<Vec<_>>();

                // Step 3.3.2
                for shorthand in shorthands {
                    let properties = shorthand.longhands();

                    // Substep 2 & 3
                    let mut current_longhands = Vec::new();
                    let mut important_count = 0;

                    for &(longhand, longhand_important) in longhands.iter() {
                        let longhand_name = longhand.name();
                        if properties.iter().any(|p| &longhand_name == *p) {
                            current_longhands.push(longhand);
                            if longhand_important {
                                important_count += 1;
                            }
                        }
                    }

                    // Substep 1
                    /* Assuming that the PropertyDeclarationBlock contains no duplicate entries,
                    if the current_longhands length is equal to the properties length, it means
                    that the properties that map to shorthand are present in longhands */
                    if current_longhands.is_empty() || current_longhands.len() != properties.len() {
                        continue;
                    }

                    // Substep 4
                    let is_important = important_count > 0;
                    if is_important && important_count != current_longhands.len() {
                        continue;
                    }

                    // TODO: serialize shorthand does not take is_important into account currently
                    // Substep 5
                    let was_serialized =
                        try!(
                            shorthand.serialize_shorthand_to_buffer(
                                dest,
                                current_longhands.iter().cloned(),
                                &mut is_first_serialization
                            )
                        );
                    // If serialization occured, Substep 7 & 8 will have been completed

                    // Substep 6
                    if !was_serialized {
                        continue;
                    }

                    for current_longhand in current_longhands {
                        // Substep 9
                        already_serialized.push(current_longhand.name());
                        let index_to_remove = longhands.iter().position(|l| l.0 == current_longhand);
                        if let Some(index) = index_to_remove {
                            // Substep 10
                            longhands.remove(index);
                        }
                     }
                 }
            }

            // Step 3.3.4
            if already_serialized.contains(&property) {
                continue;
            }

            use std::iter::Cloned;
            use std::slice;

            // Steps 3.3.5, 3.3.6 & 3.3.7
            // Need to specify an iterator type here even though it’s unused to work around
            // "error: unable to infer enough type information about `_`;
            //  type annotations or generic parameter binding required [E0282]"
            // Use the same type as earlier call to reuse generated code.
            try!(append_serialization::<W, Cloned<slice::Iter< &PropertyDeclaration>>>(
                dest,
                &property.to_string(),
                AppendableValue::Declaration(declaration),
                important,
                &mut is_first_serialization));

            // Step 3.3.8
            already_serialized.push(property);
        }

        // Step 4
        Ok(())
    }
}

enum AppendableValue<'a, I>
where I: Iterator<Item=&'a PropertyDeclaration> {
    Declaration(&'a PropertyDeclaration),
    DeclarationsForShorthand(I),
    Css(&'a str, bool)
}

fn append_property_name<W>(dest: &mut W,
                           property_name: &str,
                           is_first_serialization: &mut bool)
                           -> fmt::Result where W: fmt::Write {

    // after first serialization(key: value;) add whitespace between the pairs
    if !*is_first_serialization {
        try!(write!(dest, " "));
    }
    else {
        *is_first_serialization = false;
    }

    write!(dest, "{}", property_name)
}

fn append_declaration_value<'a, W, I>
                           (dest: &mut W,
                            appendable_value: AppendableValue<'a, I>,
                            is_important: bool)
                            -> fmt::Result
                            where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
  match appendable_value {
      AppendableValue::Css(css, _) => {
          try!(write!(dest, "{}", css))
      },
      AppendableValue::Declaration(decl) => {
          try!(decl.to_css(dest));
       },
       AppendableValue::DeclarationsForShorthand(decls) => {
           let mut decls = decls.peekable();
           while let Some(decl) = decls.next() {
               try!(decl.to_css(dest));

               if decls.peek().is_some() {
                   try!(write!(dest, " "));
               }
           }
       }
  }

  if is_important {
      try!(write!(dest, " !important"));
  }

  Ok(())
}

fn append_serialization<'a, W, I>(dest: &mut W,
                                  property_name: &str,
                                  appendable_value: AppendableValue<'a, I>,
                                  is_important: bool,
                                  is_first_serialization: &mut bool)
                                  -> fmt::Result
                                  where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {

    try!(append_property_name(dest, property_name, is_first_serialization));
    try!(write!(dest, ":"));

    // for normal parsed values, add a space between key: and value
    match &appendable_value {
        &AppendableValue::Css(_, is_unparsed) => {
            if !is_unparsed {
                try!(write!(dest, " "))
            }
        },
        &AppendableValue::Declaration(decl) => {
            if !decl.value_is_unparsed() {
                // for normal parsed values, add a space between key: and value
                try!(write!(dest, " "));
            }
         },
         &AppendableValue::DeclarationsForShorthand(_) => try!(write!(dest, " "))
    }

    try!(append_declaration_value(dest, appendable_value, is_important));
    write!(dest, ";")
}

pub fn parse_style_attribute(input: &str, base_url: &Url, error_reporter: StdBox<ParseErrorReporter + Send>)
                             -> PropertyDeclarationBlock {
    let context = ParserContext::new(Origin::Author, base_url, error_reporter);
    parse_property_declaration_list(&context, &mut Parser::new(input))
}

pub fn parse_one_declaration(name: &str, input: &str, base_url: &Url, error_reporter: StdBox<ParseErrorReporter + Send>)
                             -> Result<Vec<PropertyDeclaration>, ()> {
    let context = ParserContext::new(Origin::Author, base_url, error_reporter);
    let mut results = vec![];
    match PropertyDeclaration::parse(name, &context, &mut Parser::new(input), &mut results) {
        PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(results),
        _ => Err(())
    }
}

struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = (Vec<PropertyDeclaration>, bool);
}


impl<'a, 'b> DeclarationParser for PropertyDeclarationParser<'a, 'b> {
    type Declaration = (Vec<PropertyDeclaration>, bool);

    fn parse_value(&self, name: &str, input: &mut Parser) -> Result<(Vec<PropertyDeclaration>, bool), ()> {
        let mut results = vec![];
        try!(input.parse_until_before(Delimiter::Bang, |input| {
            match PropertyDeclaration::parse(name, self.context, input, &mut results) {
                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(()),
                _ => Err(())
            }
        }));
        let important = input.try(parse_important).is_ok();
        Ok((results, important))
    }
}


pub fn parse_property_declaration_list(context: &ParserContext, input: &mut Parser)
                                       -> PropertyDeclarationBlock {
    let mut important_declarations = Vec::new();
    let mut normal_declarations = Vec::new();
    let parser = PropertyDeclarationParser {
        context: context,
    };
    let mut iter = DeclarationListParser::new(input, parser);
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok((results, important)) => {
                if important {
                    important_declarations.extend(results);
                } else {
                    normal_declarations.extend(results);
                }
            }
            Err(range) => {
                let pos = range.start;
                let message = format!("Unsupported property declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message, &context);
            }
        }
    }
    PropertyDeclarationBlock {
        important: Arc::new(deduplicate_property_declarations(important_declarations)),
        normal: Arc::new(deduplicate_property_declarations(normal_declarations)),
    }
}


/// Only keep the last declaration for any given property.
/// The input is in source order, output in reverse source order.
fn deduplicate_property_declarations(declarations: Vec<PropertyDeclaration>)
                                     -> Vec<PropertyDeclaration> {
    let mut deduplicated = vec![];
    let mut seen = PropertyBitField::new();
    let mut seen_custom = Vec::new();
    for declaration in declarations.into_iter().rev() {
        match declaration {
            % for property in data.longhands:
                PropertyDeclaration::${property.camel_case}(..) => {
                    % if not property.derived_from:
                        if seen.get_${property.ident}() {
                            continue
                        }
                        seen.set_${property.ident}()
                    % else:
                        unreachable!();
                    % endif
                },
            % endfor
            PropertyDeclaration::Custom(ref name, _) => {
                if seen_custom.contains(name) {
                    continue
                }
                seen_custom.push(name.clone())
            }
        }
        deduplicated.push(declaration)
    }
    deduplicated
}


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CSSWideKeyword {
    InitialKeyword,
    InheritKeyword,
    UnsetKeyword,
}

impl CSSWideKeyword {
    pub fn parse(input: &mut Parser) -> Result<CSSWideKeyword, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "initial" => Ok(CSSWideKeyword::InitialKeyword),
            "inherit" => Ok(CSSWideKeyword::InheritKeyword),
            "unset" => Ok(CSSWideKeyword::UnsetKeyword),
            _ => Err(())
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, HeapSizeOf)]
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

    /// Serializes possible shorthand value to String.
    pub fn serialize_shorthand_value_to_string<'a, I>(self, declarations: I, is_important: bool) -> String
    where I: Iterator<Item=&'a PropertyDeclaration> + Clone {
        let appendable_value = self.get_shorthand_appendable_value(declarations).unwrap();
        let mut result = String::new();
        append_declaration_value(&mut result, appendable_value, is_important).unwrap();
        result
    }

    /// Serializes possible shorthand name with value to input buffer given a list of longhand declarations.
    /// On success, returns true if shorthand value is written and false if no shorthand value is present.
    pub fn serialize_shorthand_to_buffer<'a, W, I>(self,
                                                   dest: &mut W,
                                                   declarations: I,
                                                   is_first_serialization: &mut bool)
                                                   -> Result<bool, fmt::Error>
    where W: Write, I: Iterator<Item=&'a PropertyDeclaration> + Clone {
        match self.get_shorthand_appendable_value(declarations) {
            None => Ok(false),
            Some(appendable_value) => {
                let property_name = self.name();

                append_serialization(
                    dest,
                    property_name,
                    appendable_value,
                    false,
                    is_first_serialization
                ).and_then(|_| Ok(true))
            }
        }
    }

    fn get_shorthand_appendable_value<'a, I>(self, declarations: I) -> Option<AppendableValue<'a, I>>
        where I: Iterator<Item=&'a PropertyDeclaration> + Clone {

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
                   let is_unparsed = first_declaration.value_is_unparsed();
                   return Some(AppendableValue::Css(css, is_unparsed));
               }
               else {
                   return None;
               }
            }

            if !declarations3.any(|d| d.with_variables()) {
                return Some(AppendableValue::DeclarationsForShorthand(declarations));
                // FIXME: this needs property-specific code, which probably should be in style/
                // "as appropriate according to the grammar of shorthand "
                // https://drafts.csswg.org/cssom/#serialize-a-css-value
            }

            None
    }
}

#[derive(Clone, PartialEq, Eq, Debug, HeapSizeOf)]
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

#[derive(PartialEq, Clone, Debug, HeapSizeOf)]
pub enum PropertyDeclaration {
    % for property in data.longhands:
        ${property.camel_case}(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
    Custom(::custom_properties::Name, DeclaredValue<::custom_properties::SpecifiedValue>),
}


#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PropertyDeclarationParseResult {
    UnknownProperty,
    ExperimentalProperty,
    InvalidValue,
    ValidOrIgnoredDeclaration,
}

#[derive(Eq, PartialEq, Clone)]
pub enum PropertyDeclarationName {
    Longhand(&'static str),
    Custom(::custom_properties::Name),
    Internal
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

    pub fn parse(name: &str, context: &ParserContext, input: &mut Parser,
                 result_list: &mut Vec<PropertyDeclaration>) -> PropertyDeclarationParseResult {
        if let Ok(name) = ::custom_properties::parse_name(name) {
            let value = match input.try(CSSWideKeyword::parse) {
                Ok(CSSWideKeyword::UnsetKeyword) |  // Custom properties are alawys inherited
                Ok(CSSWideKeyword::InheritKeyword) => DeclaredValue::Inherit,
                Ok(CSSWideKeyword::InitialKeyword) => DeclaredValue::Initial,
                Err(()) => match ::custom_properties::parse(input) {
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
                        % if property.internal:
                            if context.stylesheet_origin != Origin::UserAgent {
                                return PropertyDeclarationParseResult::UnknownProperty
                            }
                        % endif
                        % if property.experimental:
                            if !::util::prefs::get_pref("${property.experimental}")
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
                    % if shorthand.internal:
                        if context.stylesheet_origin != Origin::UserAgent {
                            return PropertyDeclarationParseResult::UnknownProperty
                        }
                    % endif
                    % if shorthand.experimental:
                        if !::util::prefs::get_pref("${shorthand.experimental}")
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

            _ => PropertyDeclarationParseResult::UnknownProperty
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
}

pub mod style_struct_traits {
    use super::longhands;

    % for style_struct in data.active_style_structs():
        pub trait ${style_struct.trait_name}: Clone {
            % for longhand in style_struct.longhands:
                #[allow(non_snake_case)]
                fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T);
                #[allow(non_snake_case)]
                fn copy_${longhand.ident}_from(&mut self, other: &Self);
                % if longhand.need_clone:
                    #[allow(non_snake_case)]
                    fn clone_${longhand.ident}(&self) -> longhands::${longhand.ident}::computed_value::T;
                % endif
            % endfor
            % for additional in style_struct.additional_methods:
                #[allow(non_snake_case)]
                ${additional.declare()}
            % endfor
        }
    % endfor
}

pub mod style_structs {
    use fnv::FnvHasher;
    use super::longhands;
    use std::hash::{Hash, Hasher};

    % for style_struct in data.active_style_structs():
        % if style_struct.trait_name == "Font":
        #[derive(Clone, HeapSizeOf, Debug)]
        % else:
        #[derive(PartialEq, Clone, HeapSizeOf)]
        % endif
        pub struct ${style_struct.servo_struct_name} {
            % for longhand in style_struct.longhands:
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
            % if style_struct.trait_name == "Font":
                pub hash: u64,
            % endif
        }
        % if style_struct.trait_name == "Font":

        impl PartialEq for ${style_struct.servo_struct_name} {
            fn eq(&self, other: &${style_struct.servo_struct_name}) -> bool {
                self.hash == other.hash
                % for longhand in style_struct.longhands:
                    && self.${longhand.ident} == other.${longhand.ident}
                % endfor
            }
        }
        % endif

        impl super::style_struct_traits::${style_struct.trait_name} for ${style_struct.servo_struct_name} {
            % for longhand in style_struct.longhands:
                fn set_${longhand.ident}(&mut self, v: longhands::${longhand.ident}::computed_value::T) {
                    self.${longhand.ident} = v;
                }
                fn copy_${longhand.ident}_from(&mut self, other: &Self) {
                    self.${longhand.ident} = other.${longhand.ident}.clone();
                }
            % endfor
            % if style_struct.trait_name == "Border":
                % for side in ["top", "right", "bottom", "left"]:
                fn clone_border_${side}_style(&self) -> longhands::border_${side}_style::computed_value::T {
                    self.border_${side}_style.clone()
                }
                fn border_${side}_has_nonzero_width(&self) -> bool {
                    self.border_${side}_width != ::app_units::Au(0)
                }
                % endfor
            % elif style_struct.trait_name == "Box":
                fn clone_display(&self) -> longhands::display::computed_value::T {
                    self.display.clone()
                }
                fn clone_position(&self) -> longhands::position::computed_value::T {
                    self.position.clone()
                }
                fn clone_float(&self) -> longhands::float::computed_value::T {
                    self.float.clone()
                }
                fn clone_overflow_x(&self) -> longhands::overflow_x::computed_value::T {
                    self.overflow_x.clone()
                }
                fn clone_overflow_y(&self) -> longhands::overflow_y::computed_value::T {
                    self.overflow_y.clone()
                }
                fn transition_count(&self) -> usize {
                    self.transition_property.0.len()
                }
            % elif style_struct.trait_name == "Color":
                fn clone_color(&self) -> longhands::color::computed_value::T {
                    self.color.clone()
                }
            % elif style_struct.trait_name == "Font":
                fn clone_font_size(&self) -> longhands::font_size::computed_value::T {
                    self.font_size.clone()
                }
                fn clone_font_weight(&self) -> longhands::font_weight::computed_value::T {
                    self.font_weight.clone()
                }
                fn compute_font_hash(&mut self) {
                    // Corresponds to the fields in `gfx::font_template::FontTemplateDescriptor`.
                    let mut hasher: FnvHasher = Default::default();
                    hasher.write_u16(self.font_weight as u16);
                    self.font_stretch.hash(&mut hasher);
                    self.font_family.hash(&mut hasher);
                    self.hash = hasher.finish()
                }
            % elif style_struct.trait_name == "InheritedBox":
                fn clone_direction(&self) -> longhands::direction::computed_value::T {
                    self.direction.clone()
                }
                fn clone_writing_mode(&self) -> longhands::writing_mode::computed_value::T {
                    self.writing_mode.clone()
                }
                fn clone_text_orientation(&self) -> longhands::text_orientation::computed_value::T {
                    self.text_orientation.clone()
                }
            % elif style_struct.trait_name == "InheritedText" and product == "servo":
                fn clone__servo_text_decorations_in_effect(&self) ->
                    longhands::_servo_text_decorations_in_effect::computed_value::T {
                    self._servo_text_decorations_in_effect.clone()
                }
            % elif style_struct.trait_name == "Outline":
                fn clone_outline_style(&self) -> longhands::outline_style::computed_value::T {
                    self.outline_style.clone()
                }
                fn outline_has_nonzero_width(&self) -> bool {
                    self.outline_width != ::app_units::Au(0)
                }
            % elif style_struct.trait_name == "Text":
                fn has_underline(&self) -> bool {
                    self.text_decoration.underline
                }
                fn has_overline(&self) -> bool {
                    self.text_decoration.overline
                }
                fn has_line_through(&self) -> bool {
                    self.text_decoration.line_through
                }
            % endif
        }

    % endfor
}

pub trait ComputedValues : Clone + Send + Sync + 'static {
    % for style_struct in data.active_style_structs():
        type Concrete${style_struct.trait_name}: style_struct_traits::${style_struct.trait_name};
    % endfor

        // Temporary bailout case for stuff we haven't made work with the trait
        // yet - panics for non-Servo implementations.
        //
        // Used only for animations. Don't use it in other places.
        fn as_servo<'a>(&'a self) -> &'a ServoComputedValues;
        fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues;

        fn new(custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
               shareable: bool,
               writing_mode: WritingMode,
               root_font_size: Au,
        % for style_struct in data.active_style_structs():
               ${style_struct.ident}: Arc<Self::Concrete${style_struct.trait_name}>,
        % endfor
        ) -> Self;

        fn style_for_child_text_node(parent: &Arc<Self>) -> Arc<Self>;

        fn initial_values() -> &'static Self;

        fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(f: F);

    % for style_struct in data.active_style_structs():
        fn clone_${style_struct.trait_name_lower}(&self) ->
            Arc<Self::Concrete${style_struct.trait_name}>;
        fn get_${style_struct.trait_name_lower}<'a>(&'a self) ->
            &'a Self::Concrete${style_struct.trait_name};
        fn mutate_${style_struct.trait_name_lower}<'a>(&'a mut self) ->
            &'a mut Self::Concrete${style_struct.trait_name};
    % endfor

    fn custom_properties(&self) -> Option<Arc<::custom_properties::ComputedValuesMap>>;
    fn root_font_size(&self) -> Au;
    fn set_root_font_size(&mut self, size: Au);
    fn set_writing_mode(&mut self, mode: WritingMode);
    fn is_multicol(&self) -> bool;
}

#[derive(Clone, HeapSizeOf)]
pub struct ServoComputedValues {
    % for style_struct in data.active_style_structs():
        ${style_struct.ident}: Arc<style_structs::${style_struct.servo_struct_name}>,
    % endfor
    custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl ComputedValues for ServoComputedValues {
    % for style_struct in data.active_style_structs():
        type Concrete${style_struct.trait_name} = style_structs::${style_struct.servo_struct_name};
    % endfor

        fn as_servo<'a>(&'a self) -> &'a ServoComputedValues { self }
        fn as_servo_mut<'a>(&'a mut self) -> &'a mut ServoComputedValues { self }

        fn new(custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
               shareable: bool,
               writing_mode: WritingMode,
               root_font_size: Au,
            % for style_struct in data.active_style_structs():
               ${style_struct.ident}: Arc<style_structs::${style_struct.servo_struct_name}>,
            % endfor
        ) -> Self {
            ServoComputedValues {
                custom_properties: custom_properties,
                shareable: shareable,
                writing_mode: writing_mode,
                root_font_size: root_font_size,
            % for style_struct in data.active_style_structs():
                ${style_struct.ident}: ${style_struct.ident},
            % endfor
            }
        }

        fn style_for_child_text_node(parent: &Arc<Self>) -> Arc<Self> {
            // Text nodes get a copy of the parent style. Inheriting all non-
            // inherited properties into the text node is odd from a CSS
            // perspective, but makes fragment construction easier (by making
            // properties like vertical-align on fragments have values that
            // match the parent element). This is an implementation detail of
            // Servo layout that is not central to how fragment construction
            // works, but would be difficult to change. (Text node style is
            // also not visible to script.)
            parent.clone()
        }

        fn initial_values() -> &'static Self { &*INITIAL_SERVO_VALUES }

        fn do_cascade_property<F: FnOnce(&Vec<Option<CascadePropertyFn<Self>>>)>(f: F) {
            CASCADE_PROPERTY.with(|x| f(x));
        }

    % for style_struct in data.active_style_structs():
        #[inline]
        fn clone_${style_struct.trait_name_lower}(&self) ->
            Arc<Self::Concrete${style_struct.trait_name}> {
                self.${style_struct.ident}.clone()
            }
        #[inline]
        fn get_${style_struct.trait_name_lower}<'a>(&'a self) ->
            &'a Self::Concrete${style_struct.trait_name} {
                &self.${style_struct.ident}
            }
        #[inline]
        fn mutate_${style_struct.trait_name_lower}<'a>(&'a mut self) ->
            &'a mut Self::Concrete${style_struct.trait_name} {
                Arc::make_mut(&mut self.${style_struct.ident})
            }
    % endfor

    // Cloning the Arc here is fine because it only happens in the case where we have custom
    // properties, and those are both rare and expensive.
    fn custom_properties(&self) -> Option<Arc<::custom_properties::ComputedValuesMap>> {
        self.custom_properties.as_ref().map(|x| x.clone())
    }

    fn root_font_size(&self) -> Au { self.root_font_size }
    fn set_root_font_size(&mut self, size: Au) { self.root_font_size = size }
    fn set_writing_mode(&mut self, mode: WritingMode) { self.writing_mode = mode; }

    #[inline]
    fn is_multicol(&self) -> bool {
        let style = self.get_column();
        style.column_count.0.is_some() || style.column_width.0.is_some()
    }
}

impl ServoComputedValues {
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
    pub fn logical_border_width(&self) -> LogicalMargin<Au> {
        let border_style = self.get_border();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            border_style.border_top_width,
            border_style.border_right_width,
            border_style.border_bottom_width,
            border_style.border_left_width,
        ))
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
    pub fn get_font_arc(&self) -> Arc<style_structs::ServoFont> {
        self.font.clone()
    }

    // http://dev.w3.org/csswg/css-transforms/#grouping-property-values
    pub fn get_used_transform_style(&self) -> computed_values::transform_style::T {
        use computed_values::mix_blend_mode;
        use computed_values::transform_style;

        let effects = self.get_effects();

        // TODO(gw): Add clip-path, isolation, mask-image, mask-border-source when supported.
        if effects.opacity < 1.0 ||
           !effects.filter.is_empty() ||
           effects.clip.0.is_some() {
           effects.mix_blend_mode != mix_blend_mode::T::normal ||
            return transform_style::T::flat;
        }

        if effects.transform_style == transform_style::T::auto {
            if effects.transform.0.is_some() {
                return transform_style::T::flat;
            }
            if effects.perspective != computed::LengthOrNone::None {
                return transform_style::T::flat;
            }
        }

        // Return the computed value if not overridden by the above exceptions
        effects.transform_style
    }

    pub fn transform_requires_layer(&self) -> bool {
        // Check if the transform matrix is 2D or 3D
        if let Some(ref transform_list) = self.get_effects().transform.0 {
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
pub fn get_writing_mode<S: style_struct_traits::InheritedBox>(inheritedbox_style: &S) -> WritingMode {
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


/// The initial values for all style structs as defined by the specification.
lazy_static! {
    pub static ref INITIAL_SERVO_VALUES: ServoComputedValues = ServoComputedValues {
        % for style_struct in data.active_style_structs():
            ${style_struct.ident}: Arc::new(style_structs::${style_struct.servo_struct_name} {
                % for longhand in style_struct.longhands:
                    ${longhand.ident}: longhands::${longhand.ident}::get_initial_value(),
                % endfor
                % if style_struct.trait_name == "Font":
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


/// Fast path for the function below. Only computes new inherited styles.
#[allow(unused_mut, unused_imports)]
fn cascade_with_cached_declarations<C: ComputedValues>(
        viewport_size: Size2D<Au>,
        applicable_declarations: &[DeclarationBlock<Vec<PropertyDeclaration>>],
        shareable: bool,
        parent_style: &C,
        cached_style: &C,
        custom_properties: Option<Arc<::custom_properties::ComputedValuesMap>>,
        mut error_reporter: StdBox<ParseErrorReporter + Send>)
        -> C {
    let mut context = computed::Context {
        is_root_element: false,
        viewport_size: viewport_size,
        inherited_style: parent_style,
        style: C::new(
            custom_properties,
            shareable,
            WritingMode::empty(),
            parent_style.root_font_size(),
            % for style_struct in data.active_style_structs():
                % if style_struct.inherited:
                    parent_style
                % else:
                    cached_style
                % endif
                    .clone_${style_struct.trait_name_lower}(),
            % endfor
        ),
    };
    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order,
    // we want them in decreasing order here.
    for sub_list in applicable_declarations.iter().rev() {
        // Declarations are already stored in reverse order.
        for declaration in sub_list.declarations.iter() {
            match *declaration {
                % for style_struct in data.active_style_structs():
                    % for property in style_struct.longhands:
                        % if not property.derived_from:
                            PropertyDeclaration::${property.camel_case}(ref
                                    ${'_' if not style_struct.inherited else ''}declared_value)
                                    => {
                                    use properties::style_struct_traits::${style_struct.trait_name};
                                % if style_struct.inherited:
                                    if seen.get_${property.ident}() {
                                        continue
                                    }
                                    seen.set_${property.ident}();
                                    let custom_props = context.style().custom_properties();
                                    substitute_variables_${property.ident}(
                                        declared_value, &custom_props,
                                        |value| match *value {
                                            DeclaredValue::Value(ref specified_value)
                                            => {
                                                let computed = specified_value.to_computed_value(&context);
                                                context.mutate_style().mutate_${style_struct.trait_name_lower}()
                                                       .set_${property.ident}(computed);
                                            },
                                            DeclaredValue::Initial
                                            => {
                                                // FIXME(bholley): We may want set_X_to_initial_value() here.
                                                let initial = longhands::${property.ident}::get_initial_value();
                                                context.mutate_style().mutate_${style_struct.trait_name_lower}()
                                                       .set_${property.ident}(initial);
                                            },
                                            DeclaredValue::Inherit => {
                                                // This is a bit slow, but this is rare so it shouldn't
                                                // matter.
                                                //
                                                // FIXME: is it still?
                                                let inherited_struct = parent_style.get_${style_struct.ident}();
                                                context.mutate_style().mutate_${style_struct.trait_name_lower}()
                                                       .copy_${property.ident}_from(inherited_struct);
                                            }
                                            DeclaredValue::WithVariables { .. } => unreachable!()
                                        }, &mut error_reporter
                                    );
                                % endif

                                % if property.name in data.derived_longhands:
                                    % for derived in data.derived_longhands[property.name]:
                                            longhands::${derived.ident}
                                                     ::derive_from_${property.ident}(&mut context);
                                    % endfor
                                % endif
                            }
                        % else:
                            PropertyDeclaration::${property.camel_case}(_) => {
                                // Do not allow stylesheets to set derived properties.
                            }
                        % endif
                    % endfor
                % endfor
                PropertyDeclaration::Custom(..) => {}
            }
        }
    }

    if seen.get_font_style() || seen.get_font_weight() || seen.get_font_stretch() ||
            seen.get_font_family() {
        use properties::style_struct_traits::Font;
        context.mutate_style().mutate_font().compute_font_hash();
    }

    context.style
}

pub type CascadePropertyFn<C /*: ComputedValues */> =
    extern "Rust" fn(declaration: &PropertyDeclaration,
                     inherited_style: &C,
                     context: &mut computed::Context<C>,
                     seen: &mut PropertyBitField,
                     cacheable: &mut bool,
                     error_reporter: &mut StdBox<ParseErrorReporter + Send>);

pub fn make_cascade_vec<C: ComputedValues>() -> Vec<Option<CascadePropertyFn<C>>> {
    let mut result: Vec<Option<CascadePropertyFn<C>>> = Vec::new();
    % for style_struct in data.active_style_structs():
        % for property in style_struct.longhands:
            let discriminant;
            unsafe {
                let variant = PropertyDeclaration::${property.camel_case}(intrinsics::uninit());
                discriminant = intrinsics::discriminant_value(&variant) as usize;
                mem::forget(variant);
            }
            while result.len() < discriminant + 1 {
                result.push(None)
            }
            result[discriminant] = Some(longhands::${property.ident}::cascade_property);
        % endfor
    % endfor
    result
}

// This is a thread-local rather than a lazy static to avoid atomic operations when cascading
// properties.
thread_local!(static CASCADE_PROPERTY: Vec<Option<CascadePropertyFn<ServoComputedValues>>> = {
    make_cascade_vec::<ServoComputedValues>()
});

/// Performs the CSS cascade, computing new styles for an element from its parent style and
/// optionally a cached related style. The arguments are:
///
///   * `viewport_size`: The size of the initial viewport.
///
///   * `applicable_declarations`: The list of CSS rules that matched.
///
///   * `shareable`: Whether the `ComputedValues` structure to be constructed should be considered
///     shareable.
///
///   * `parent_style`: The parent style, if applicable; if `None`, this is the root node.
///
///   * `cached_style`: If present, cascading is short-circuited for everything but inherited
///     values and these values are used instead. Obviously, you must be careful when supplying
///     this that it is safe to only provide inherited declarations. If `parent_style` is `None`,
///     this is ignored.
///
/// Returns the computed values and a boolean indicating whether the result is cacheable.
pub fn cascade<C: ComputedValues>(
               viewport_size: Size2D<Au>,
               applicable_declarations: &[DeclarationBlock<Vec<PropertyDeclaration>>],
               shareable: bool,
               parent_style: Option<<&C>,
               cached_style: Option<<&C>,
               mut error_reporter: StdBox<ParseErrorReporter + Send>)
               -> (C, bool) {
    use properties::style_struct_traits::{Border, Box, Font, Outline};
    let initial_values = C::initial_values();
    let (is_root_element, inherited_style) = match parent_style {
        Some(parent_style) => (false, parent_style),
        None => (true, initial_values),
    };

    let inherited_custom_properties = inherited_style.custom_properties();
    let mut custom_properties = None;
    let mut seen_custom = HashSet::new();
    for sub_list in applicable_declarations.iter().rev() {
        // Declarations are already stored in reverse order.
        for declaration in sub_list.declarations.iter() {
            match *declaration {
                PropertyDeclaration::Custom(ref name, ref value) => {
                    ::custom_properties::cascade(
                        &mut custom_properties, &inherited_custom_properties,
                        &mut seen_custom, name, value)
                }
                _ => {}
            }
        }
    }
    let custom_properties = ::custom_properties::finish_cascade(
            custom_properties, &inherited_custom_properties);

    if let (Some(cached_style), Some(parent_style)) = (cached_style, parent_style) {
        let style = cascade_with_cached_declarations(viewport_size,
                                                     applicable_declarations,
                                                     shareable,
                                                     parent_style,
                                                     cached_style,
                                                     custom_properties,
                                                     error_reporter);
        return (style, false)
    }

    let mut context = computed::Context {
        is_root_element: is_root_element,
        viewport_size: viewport_size,
        inherited_style: inherited_style,
        style: C::new(
            custom_properties,
            shareable,
            WritingMode::empty(),
            inherited_style.root_font_size(),
            % for style_struct in data.active_style_structs():
            % if style_struct.inherited:
            inherited_style
            % else:
            initial_values
            % endif
                .clone_${style_struct.trait_name_lower}(),
            % endfor
        ),
    };

    // Set computed values, overwriting earlier declarations for the same property.
    let mut cacheable = true;
    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order, we want them in decreasing
    // order here.
    //
    // We could (and used to) use a pattern match here, but that bloats this function to over 100K
    // of compiled code! To improve i-cache behavior, we outline the individual functions and use
    // virtual dispatch instead.
    C::do_cascade_property(|cascade_property| {
        % for category_to_cascade_now in ["early", "other"]:
            for sub_list in applicable_declarations.iter().rev() {
                // Declarations are already stored in reverse order.
                for declaration in sub_list.declarations.iter() {
                    if let PropertyDeclaration::Custom(..) = *declaration {
                        continue
                    }
                    // The computed value of some properties depends on the (sometimes computed)
                    // value of *other* properties.
                    // So we classify properties into "early" and "other",
                    // such that the only dependencies can be from "other" to "early".
                    // We iterate applicable_declarations twice, first cascading "early" properties
                    // then "other".
                    // Unfortunately, it’s not easy to check that this classification is correct.
                    let is_early_property = matches!(*declaration,
                        PropertyDeclaration::FontSize(_) |
                        PropertyDeclaration::Color(_) |
                        PropertyDeclaration::Position(_) |
                        PropertyDeclaration::Float(_) |
                        PropertyDeclaration::TextDecoration(_)
                    );
                    if
                        % if category_to_cascade_now == "early":
                            !
                        % endif
                        is_early_property
                    {
                        continue
                    }
                    let discriminant = unsafe {
                        intrinsics::discriminant_value(declaration) as usize
                    };
                    (cascade_property[discriminant].unwrap())(declaration,
                                                              inherited_style,
                                                              &mut context,
                                                              &mut seen,
                                                              &mut cacheable,
                                                              &mut error_reporter);
                }
            }
        % endfor
    });

    let mut style = context.style;

    let positioned = matches!(style.get_box().clone_position(),
        longhands::position::SpecifiedValue::absolute |
        longhands::position::SpecifiedValue::fixed);
    let floated = style.get_box().clone_float() != longhands::float::SpecifiedValue::none;
    if positioned || floated || is_root_element {
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
                box_.set__servo_display_for_hypothetical_box(if is_root_element {
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

    // The initial value of border-*-width may be changed at computed value time.
    % for side in ["top", "right", "bottom", "left"]:
        // Like calling to_computed_value, which wouldn't type check.
        if style.get_border().clone_border_${side}_style().none_or_hidden() &&
           style.get_border().border_${side}_has_nonzero_width() {
            style.mutate_border().set_border_${side}_width(Au(0));
        }
    % endfor

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
        use properties::style_struct_traits::Font;
        style.mutate_font().compute_font_hash();
    }

    let mode = get_writing_mode(style.get_inheritedbox());
    style.set_writing_mode(mode);
    (style, cacheable)
}

/// Alters the given style to accommodate replaced content. This is called in flow construction. It
/// handles cases like `<div style="position: absolute">foo bar baz</div>` (in which `foo`, `bar`,
/// and `baz` must not be absolutely-positioned) and cases like `<sup>Foo</sup>` (in which the
/// `vertical-align: top` style of `sup` must not propagate down into `Foo`).
///
/// FIXME(#5625, pcwalton): It would probably be cleaner and faster to do this in the cascade.
#[inline]
pub fn modify_style_for_replaced_content(style: &mut Arc<ServoComputedValues>) {
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
#[inline]
pub fn modify_border_style_for_inline_sides(style: &mut Arc<ServoComputedValues>,
                                            is_first_fragment_of_element: bool,
                                            is_last_fragment_of_element: bool) {
    fn modify_side(style: &mut Arc<ServoComputedValues>, side: PhysicalSide) {
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

/// Adjusts the display and position properties as appropriate for an anonymous table object.
#[inline]
pub fn modify_style_for_anonymous_table_object(
        style: &mut Arc<ServoComputedValues>,
        new_display_value: longhands::display::computed_value::T) {
    let mut style = Arc::make_mut(style);
    let box_style = Arc::make_mut(&mut style.box_);
    box_style.display = new_display_value;
    box_style.position = longhands::position::computed_value::T::static_;
}

/// Adjusts the `position` property as necessary for the outer fragment wrapper of an inline-block.
#[inline]
pub fn modify_style_for_outer_inline_block_fragment(style: &mut Arc<ServoComputedValues>) {
    let mut style = Arc::make_mut(style);
    let box_style = Arc::make_mut(&mut style.box_);
    box_style.position = longhands::position::computed_value::T::static_
}

/// Adjusts the `position` and `padding` properties as necessary to account for text.
///
/// Text is never directly relatively positioned; it's always contained within an element that is
/// itself relatively positioned.
#[inline]
pub fn modify_style_for_text(style: &mut Arc<ServoComputedValues>) {
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

/// Adjusts the `margin` property as necessary to account for the text of an `input` element.
///
/// Margins apply to the `input` element itself, so including them in the text will cause them to
/// be double-counted.
pub fn modify_style_for_input_text(style: &mut Arc<ServoComputedValues>) {
    let mut style = Arc::make_mut(style);
    let margin_style = Arc::make_mut(&mut style.margin);
    margin_style.margin_top = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_right = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_bottom = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_left = computed::LengthOrPercentageOrAuto::Length(Au(0));

    // whitespace inside text input should not be collapsed
    let inherited_text = Arc::make_mut(&mut style.inheritedtext);
    inherited_text.white_space = longhands::white_space::computed_value::T::pre;
}

/// Adjusts the `clip` property so that an inline absolute hypothetical fragment doesn't clip its
/// children.
pub fn modify_style_for_inline_absolute_hypothetical_fragment(style: &mut Arc<ServoComputedValues>) {
    if style.get_effects().clip.0.is_some() {
        let mut style = Arc::make_mut(style);
        let effects_style = Arc::make_mut(&mut style.effects);
        effects_style.clip.0 = None
    }
}

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
