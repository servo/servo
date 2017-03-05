/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A property declaration block.

#![deny(missing_docs)]

use cssparser::{DeclarationListParser, parse_important};
use cssparser::{Parser, AtRuleParser, DeclarationParser, Delimiter};
use error_reporting::ParseErrorReporter;
use parser::{ParserContext, ParserContextExtraData, log_css_error};
use servo_url::ServoUrl;
use std::boxed::Box as StdBox;
use std::fmt;
use style_traits::ToCss;
use stylesheets::Origin;
use super::*;

/// A declaration [importance][importance].
///
/// [importance]: https://drafts.csswg.org/css-cascade/#importance
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Importance {
    /// Indicates a declaration without `!important`.
    Normal,

    /// Indicates a declaration with `!important`.
    Important,
}

impl Importance {
    /// Return whether this is an important declaration.
    pub fn important(self) -> bool {
        match self {
            Importance::Normal => false,
            Importance::Important => true,
        }
    }
}

impl Default for Importance {
    #[inline]
    fn default() -> Self {
        Importance::Normal
    }
}

/// Overridden declarations are skipped.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct PropertyDeclarationBlock {
    /// The group of declarations, along with their importance.
    ///
    /// Only deduplicated declarations appear here.
    #[cfg_attr(feature = "servo", ignore_heap_size_of = "#7038")]
    pub declarations: Vec<(PropertyDeclaration, Importance)>,

    /// The number of entries in `self.declaration` with `Importance::Important`
    pub important_count: u32,
}

impl PropertyDeclarationBlock {
    /// Returns wheather this block contains any declaration with `!important`.
    ///
    /// This is based on the `important_count` counter,
    /// which should be maintained whenever `declarations` is changed.
    // FIXME: make fields private and maintain it here in methods?
    pub fn any_important(&self) -> bool {
        self.important_count > 0
    }

    /// Returns wheather this block contains any declaration without `!important`.
    ///
    /// This is based on the `important_count` counter,
    /// which should be maintained whenever `declarations` is changed.
    // FIXME: make fields private and maintain it here in methods?
    pub fn any_normal(&self) -> bool {
        self.declarations.len() > self.important_count as usize
    }

    /// Get a declaration for a given property.
    ///
    /// NOTE: This is linear time.
    pub fn get(&self, property: PropertyDeclarationId) -> Option< &(PropertyDeclaration, Importance)> {
        self.declarations.iter().find(|&&(ref decl, _)| decl.id() == property)
    }

    /// Find the value of the given property in this block and serialize it
    ///
    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    pub fn property_value_to_css<W>(&self, property: &PropertyId, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        // Step 1.1: done when parsing a string to PropertyId

        // Step 1.2
        match property.as_shorthand() {
            Ok(shorthand) => {
                // Step 1.2.1
                let mut list = Vec::new();
                let mut important_count = 0;

                // Step 1.2.2
                for &longhand in shorthand.longhands() {
                    // Step 1.2.2.1
                    let declaration = self.get(PropertyDeclarationId::Longhand(longhand));

                    // Step 1.2.2.2 & 1.2.2.3
                    match declaration {
                        Some(&(ref declaration, importance)) => {
                            list.push(declaration);
                            if importance.important() {
                                important_count += 1;
                            }
                        },
                        None => return Ok(()),
                    }
                }

                // If there is one or more longhand with important, and one or more
                // without important, we don't serialize it as a shorthand.
                if important_count > 0 && important_count != list.len() {
                    return Ok(());
                }

                // Step 1.2.3
                // We don't print !important when serializing individual properties,
                // so we treat this as a normal-importance property
                match shorthand.get_shorthand_appendable_value(list) {
                    Some(appendable_value) =>
                        append_declaration_value(dest, appendable_value),
                    None => return Ok(()),
                }
            }
            Err(longhand_or_custom) => {
                if let Some(&(ref value, _importance)) = self.get(longhand_or_custom) {
                    // Step 2
                    value.to_css(dest)
                } else {
                    // Step 3
                    Ok(())
                }
            }
        }
    }

    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    pub fn property_priority(&self, property: &PropertyId) -> Importance {
        // Step 1: done when parsing a string to PropertyId

        // Step 2
        match property.as_shorthand() {
            Ok(shorthand) => {
                // Step 2.1 & 2.2 & 2.3
                if shorthand.longhands().iter().all(|&l| {
                    self.get(PropertyDeclarationId::Longhand(l))
                        .map_or(false, |&(_, importance)| importance.important())
                }) {
                    Importance::Important
                } else {
                    Importance::Normal
                }
            }
            Err(longhand_or_custom) => {
                // Step 3
                self.get(longhand_or_custom).map_or(Importance::Normal, |&(_, importance)| importance)
            }
        }
    }

    /// Adds or overrides the declaration for a given property in this block,
    /// without taking into account any kind of priority.
    pub fn set_parsed_declaration(&mut self,
                                  declaration: PropertyDeclaration,
                                  importance: Importance) {
        for slot in &mut *self.declarations {
            if slot.0.id() == declaration.id() {
                match (slot.1, importance) {
                    (Importance::Normal, Importance::Important) => {
                        self.important_count += 1;
                    }
                    (Importance::Important, Importance::Normal) => {
                        self.important_count -= 1;
                    }
                    _ => {}
                }
                *slot = (declaration, importance);
                return
            }
        }

        self.declarations.push((declaration, importance));
        if importance.important() {
            self.important_count += 1;
        }
    }

    /// Set the declaration importance for a given property, if found.
    ///
    /// Returns whether any declaration was updated.
    pub fn set_importance(&mut self, property: &PropertyId, new_importance: Importance) -> bool {
        let mut updated_at_least_one = false;
        for &mut (ref declaration, ref mut importance) in &mut self.declarations {
            if declaration.id().is_or_is_longhand_of(property) {
                match (*importance, new_importance) {
                    (Importance::Normal, Importance::Important) => {
                        self.important_count += 1;
                    }
                    (Importance::Important, Importance::Normal) => {
                        self.important_count -= 1;
                    }
                    _ => {
                        continue;
                    }
                }
                updated_at_least_one = true;
                *importance = new_importance;
            }
        }
        updated_at_least_one
    }

    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    ///
    /// Returns whether any declaration was actually removed.
    pub fn remove_property(&mut self, property: &PropertyId) -> bool {
        let important_count = &mut self.important_count;
        let mut removed_at_least_one = false;
        self.declarations.retain(|&(ref declaration, importance)| {
            let remove = declaration.id().is_or_is_longhand_of(property);
            if remove {
                removed_at_least_one = true;
                if importance.important() {
                    *important_count -= 1
                }
            }
            !remove
        });

        removed_at_least_one
    }

    /// Take a declaration block known to contain a single property and serialize it.
    pub fn single_value_to_css<W>(&self, property: &PropertyId, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match property.as_shorthand() {
            Err(_longhand_or_custom) => {
                if self.declarations.len() == 1 {
                    self.declarations[0].0.to_css(dest)
                } else {
                    Err(fmt::Error)
                }
            }
            Ok(shorthand) => {
                // we use this function because a closure won't be `Clone`
                fn get_declaration(dec: &(PropertyDeclaration, Importance))
                    -> &PropertyDeclaration {
                    &dec.0
                }
                if !self.declarations.iter().all(|decl| decl.0.shorthands().contains(&shorthand)) {
                    return Err(fmt::Error)
                }
                let iter = self.declarations.iter().map(get_declaration as fn(_) -> _);
                match shorthand.get_shorthand_appendable_value(iter) {
                    Some(AppendableValue::Css(css)) => dest.write_str(css),
                    Some(AppendableValue::DeclarationsForShorthand(_, decls)) => {
                        shorthand.longhands_to_css(decls, dest)
                    }
                    _ => Ok(())
                }
            }
        }
    }

    /// Only keep the "winning" declaration for any given property, by importance then source order.
    pub fn deduplicate(&mut self) {
        let mut deduplicated = Vec::with_capacity(self.declarations.len());
        let mut seen_normal = PropertyDeclarationIdSet::new();
        let mut seen_important = PropertyDeclarationIdSet::new();

        for (declaration, importance) in self.declarations.drain(..).rev() {
            if importance.important() {
                let id = declaration.id();
                if seen_important.contains(id) {
                    self.important_count -= 1;
                    continue
                }
                if seen_normal.contains(id) {
                    let previous_len = deduplicated.len();
                    deduplicated.retain(|&(ref d, _)| PropertyDeclaration::id(d) != id);
                    debug_assert_eq!(deduplicated.len(), previous_len - 1);
                }
                seen_important.insert(id);
            } else {
                let id = declaration.id();
                if seen_normal.contains(id) ||
                   seen_important.contains(id) {
                    continue
                }
                seen_normal.insert(id)
            }
            deduplicated.push((declaration, importance))
        }
        deduplicated.reverse();
        self.declarations = deduplicated;
    }
}

impl ToCss for PropertyDeclarationBlock {
    // https://drafts.csswg.org/cssom/#serialize-a-css-declaration-block
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        let mut is_first_serialization = true; // trailing serializations should have a prepended space

        // Step 1 -> dest = result list

        // Step 2
        let mut already_serialized = Vec::new();

        // Step 3
        for &(ref declaration, importance) in &*self.declarations {
            // Step 3.1
            let property = declaration.id();

            // Step 3.2
            if already_serialized.contains(&property) {
                continue;
            }

            // Step 3.3
            let shorthands = declaration.shorthands();
            if !shorthands.is_empty() {
                // Step 3.3.1
                let mut longhands = self.declarations.iter()
                    .filter(|d| !already_serialized.contains(&d.0.id()))
                    .collect::<Vec<_>>();

                // Step 3.3.2
                for &shorthand in shorthands {
                    let properties = shorthand.longhands();

                    // Substep 2 & 3
                    let mut current_longhands = Vec::new();
                    let mut important_count = 0;

                    for &&(ref longhand, longhand_importance) in longhands.iter() {
                        if longhand.id().is_longhand_of(shorthand) {
                            current_longhands.push(longhand);
                            if longhand_importance.important() {
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
                    let importance = if is_important {
                        Importance::Important
                    } else {
                        Importance::Normal
                    };

                    // Substep 5 - Let value be the result of invoking serialize a CSS
                    // value of current longhands.
                    let mut value = String::new();
                    match shorthand.get_shorthand_appendable_value(current_longhands.iter().cloned()) {
                        None => continue,
                        Some(appendable_value) => {
                            try!(append_declaration_value(&mut value, appendable_value));
                        }
                    }

                    // Substep 6
                    if value.is_empty() {
                        continue
                    }

                    // Substeps 7 and 8
                    try!(append_serialization::<W, Cloned<slice::Iter< _>>, _>(
                             dest, &shorthand, AppendableValue::Css(&value), importance,
                             &mut is_first_serialization));

                    for current_longhand in current_longhands {
                        // Substep 9
                        already_serialized.push(current_longhand.id());
                        let index_to_remove = longhands.iter().position(|l| l.0 == *current_longhand);
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
            try!(append_serialization::<W, Cloned<slice::Iter< &PropertyDeclaration>>, _>(
                dest,
                &property,
                AppendableValue::Declaration(declaration),
                importance,
                &mut is_first_serialization));

            // Step 3.3.8
            already_serialized.push(property);
        }

        // Step 4
        Ok(())
    }
}

/// A convenient enum to represent different kinds of stuff that can represent a
/// _value_ in the serialization of a property declaration.
pub enum AppendableValue<'a, I>
    where I: Iterator<Item=&'a PropertyDeclaration>,
{
    /// A given declaration, of which we'll serialize just the value.
    Declaration(&'a PropertyDeclaration),
    /// A set of declarations for a given shorthand.
    ///
    /// FIXME: This needs more docs, where are the shorthands expanded? We print
    /// the property name before-hand, don't we?
    DeclarationsForShorthand(ShorthandId, I),
    /// A raw CSS string, coming for example from a property with CSS variables,
    /// or when storing a serialized shorthand value before appending directly.
    Css(&'a str)
}

/// Potentially appends whitespace after the first (property: value;) pair.
fn handle_first_serialization<W>(dest: &mut W,
                                 is_first_serialization: &mut bool)
                                 -> fmt::Result
    where W: fmt::Write,
{
    if !*is_first_serialization {
        try!(write!(dest, " "));
    } else {
        *is_first_serialization = false;
    }

    Ok(())
}

/// Append a given kind of appendable value to a serialization.
pub fn append_declaration_value<'a, W, I>(dest: &mut W,
                                          appendable_value: AppendableValue<'a, I>)
                                          -> fmt::Result
    where W: fmt::Write,
          I: Iterator<Item=&'a PropertyDeclaration>,
{
    match appendable_value {
        AppendableValue::Css(css) => {
            try!(write!(dest, "{}", css))
        },
        AppendableValue::Declaration(decl) => {
            try!(decl.to_css(dest));
        },
        AppendableValue::DeclarationsForShorthand(shorthand, decls) => {
            try!(shorthand.longhands_to_css(decls, dest));
        }
    }

    Ok(())
}

/// Append a given property and value pair to a serialization.
pub fn append_serialization<'a, W, I, N>(dest: &mut W,
                                         property_name: &N,
                                         appendable_value: AppendableValue<'a, I>,
                                         importance: Importance,
                                         is_first_serialization: &mut bool)
                                         -> fmt::Result
    where W: fmt::Write,
          I: Iterator<Item=&'a PropertyDeclaration>,
          N: ToCss,
{
    try!(handle_first_serialization(dest, is_first_serialization));

    try!(property_name.to_css(dest));
    try!(dest.write_char(':'));

    // for normal parsed values, add a space between key: and value
    match &appendable_value {
        &AppendableValue::Css(_) => {
            try!(write!(dest, " "))
        },
        &AppendableValue::Declaration(decl) => {
            if !decl.value_is_unparsed() {
                // for normal parsed values, add a space between key: and value
                try!(write!(dest, " "));
            }
        },
        // Currently append_serialization is only called with a Css or
        // a Declaration AppendableValue.
        &AppendableValue::DeclarationsForShorthand(..) => unreachable!()
    }

    try!(append_declaration_value(dest, appendable_value));

    if importance.important() {
        try!(write!(dest, " !important"));
    }

    write!(dest, ";")
}

/// A helper to parse the style attribute of an element, in order for this to be
/// shared between Servo and Gecko.
pub fn parse_style_attribute(input: &str,
                             base_url: &ServoUrl,
                             error_reporter: StdBox<ParseErrorReporter + Send>,
                             extra_data: ParserContextExtraData)
                             -> PropertyDeclarationBlock {
    let context = ParserContext::new_with_extra_data(Origin::Author, base_url, error_reporter, extra_data);
    parse_property_declaration_list(&context, &mut Parser::new(input))
}

/// Parse a given property declaration. Can result in multiple
/// `PropertyDeclaration`s when expanding a shorthand, for example.
///
/// The vector returned will not have the importance set;
/// this does not attempt to parse !important at all
pub fn parse_one_declaration(id: PropertyId,
                             input: &str,
                             base_url: &ServoUrl,
                             error_reporter: StdBox<ParseErrorReporter + Send>,
                             extra_data: ParserContextExtraData)
                             -> Result<Vec<(PropertyDeclaration, Importance)>, ()> {
    let context = ParserContext::new_with_extra_data(Origin::Author, base_url, error_reporter, extra_data);
    Parser::new(input).parse_entirely(|parser| {
        let mut results = vec![];
        match PropertyDeclaration::parse(id, &context, parser, &mut results, false) {
            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(results),
            _ => Err(())
        }
    })
}

/// A struct to parse property declarations.
struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
    declarations: Vec<(PropertyDeclaration, Importance)>
}


/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = (u32, Importance);
}


impl<'a, 'b> DeclarationParser for PropertyDeclarationParser<'a, 'b> {
    /// Declarations are pushed to the internal vector. This will only
    /// let you know if the decl was !important and how many longhands
    /// it expanded to
    type Declaration = (u32, Importance);

    fn parse_value(&mut self, name: &str, input: &mut Parser)
                   -> Result<(u32, Importance), ()> {
        let id = try!(PropertyId::parse(name.into()));
        let old_len = self.declarations.len();
        let parse_result = input.parse_until_before(Delimiter::Bang, |input| {
            match PropertyDeclaration::parse(id, self.context, input, &mut self.declarations, false) {
                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(()),
                _ => Err(())
            }
        });
        if let Err(_) = parse_result {
            // rollback
            self.declarations.truncate(old_len);
            return Err(())
        }
        let importance = match input.try(parse_important) {
            Ok(()) => Importance::Important,
            Err(()) => Importance::Normal,
        };
        // In case there is still unparsed text in the declaration, we should roll back.
        if !input.is_exhausted() {
            self.declarations.truncate(old_len);
            return Err(())
        }
        for decl in &mut self.declarations[old_len..] {
            decl.1 = importance
        }
        Ok(((self.declarations.len() - old_len) as u32, importance))
    }
}


/// Parse a list of property declarations and return a property declaration
/// block.
pub fn parse_property_declaration_list(context: &ParserContext,
                                       input: &mut Parser)
                                       -> PropertyDeclarationBlock {
    let mut important_count = 0;
    let parser = PropertyDeclarationParser {
        context: context,
        declarations: vec![],
    };
    let mut iter = DeclarationListParser::new(input, parser);
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok((count, importance)) => {
                if importance.important() {
                    important_count += count;
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
    let mut block = PropertyDeclarationBlock {
        declarations: iter.parser.declarations,
        important_count: important_count,
    };
    block.deduplicate();
    block
}
