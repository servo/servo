/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{DeclarationListParser, parse_important};
use cssparser::{Parser, AtRuleParser, DeclarationParser, Delimiter};
use error_reporting::ParseErrorReporter;
use parser::{ParserContext, ParserContextExtraData, log_css_error};
use std::ascii::AsciiExt;
use std::boxed::Box as StdBox;
use std::fmt;
use style_traits::ToCss;
use stylesheets::Origin;
use super::*;
use url::Url;


#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Importance {
    /// Indicates a declaration without `!important`.
    Normal,

    /// Indicates a declaration with `!important`.
    Important,
}

impl Importance {
    pub fn important(self) -> bool {
        match self {
            Importance::Normal => false,
            Importance::Important => true,
        }
    }
}

/// Overridden declarations are skipped.
// FIXME (https://github.com/servo/servo/issues/3426)
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct PropertyDeclarationBlock {
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

    pub fn get(&self, property_name: &str) -> Option< &(PropertyDeclaration, Importance)> {
        self.declarations.iter().find(|&&(ref decl, _)| decl.matches(property_name))
    }

    /// Find the value of the given property in this block and serialize it
    ///
    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertyvalue
    pub fn property_value_to_css<W>(&self, property_name: &str, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        // Step 1
        let property = property_name.to_ascii_lowercase();

        // Step 2
        if let Some(shorthand) = Shorthand::from_name(&property) {
            // Step 2.1
            let mut list = Vec::new();

            // Step 2.2
            for longhand in shorthand.longhands() {
                // Step 2.2.1
                let declaration = self.get(longhand);

                // Step 2.2.2 & 2.2.3
                match declaration {
                    Some(&(ref declaration, _importance)) => list.push(declaration),
                    None => return Ok(()),
                }
            }

            // Step 2.3
            // TODO: importance is hardcoded because method does not implement it yet
            let importance = Importance::Normal;
            let appendable_value = shorthand.get_shorthand_appendable_value(list).unwrap();
            return append_declaration_value(dest, appendable_value, importance)
        }

        if let Some(&(ref value, _importance)) = self.get(property_name) {
            // Step 3
            value.to_css(dest)
        } else {
            // Step 4
            Ok(())
        }
    }

    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-getpropertypriority
    pub fn property_priority(&self, property_name: &str) -> Importance {
        // Step 1
        let property = property_name.to_ascii_lowercase();

        // Step 2
        if let Some(shorthand) = Shorthand::from_name(&property) {
            // Step 2.1 & 2.2 & 2.3
            if shorthand.longhands().iter().all(|l| {
                self.get(l).map_or(false, |&(_, importance)| importance.important())
            }) {
                Importance::Important
            } else {
                Importance::Normal
            }
        } else {
            // Step 3
            self.get(&property).map_or(Importance::Normal, |&(_, importance)| importance)
        }
    }

    pub fn set_parsed_declaration(&mut self, declaration: PropertyDeclaration,
                                  importance: Importance) {
        for slot in &mut *self.declarations {
            if slot.0.name() == declaration.name() {
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

    pub fn set_importance(&mut self, property_names: &[&str], new_importance: Importance) {
        for &mut (ref declaration, ref mut importance) in &mut self.declarations {
            if property_names.iter().any(|p| declaration.matches(p)) {
                match (*importance, new_importance) {
                    (Importance::Normal, Importance::Important) => {
                        self.important_count += 1;
                    }
                    (Importance::Important, Importance::Normal) => {
                        self.important_count -= 1;
                    }
                    _ => {}
                }
                *importance = new_importance;
            }
        }
    }

    /// https://dev.w3.org/csswg/cssom/#dom-cssstyledeclaration-removeproperty
    pub fn remove_property(&mut self, property_name: &str) {
        // Step 2
        let property = property_name.to_ascii_lowercase();

        match Shorthand::from_name(&property) {
            // Step 4
            Some(shorthand) => self.remove_longhands(shorthand.longhands()),
            // Step 5
            None => self.remove_longhands(&[&*property]),
        }
    }

    fn remove_longhands(&mut self, names: &[&str]) {
        let important_count = &mut self.important_count;
        self.declarations.retain(|&(ref declaration, importance)| {
            let retain = !names.iter().any(|n| declaration.matches(n));
            if !retain && importance.important() {
                *important_count -= 1
            }
            retain
        })
    }

    /// Take a declaration block known to contain a single property and serialize it.
    pub fn single_value_to_css<W>(&self, property_name: &str, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        match self.declarations.len() {
            0 => Err(fmt::Error),
            1 if self.declarations[0].0.name().eq_str_ignore_ascii_case(property_name) => {
                self.declarations[0].0.to_css(dest)
            }
            _ => {
                // we use this function because a closure won't be `Clone`
                fn get_declaration(dec: &(PropertyDeclaration, Importance))
                    -> &PropertyDeclaration {
                    &dec.0
                }
                let shorthand = try!(Shorthand::from_name(property_name).ok_or(fmt::Error));
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
}

impl ToCss for PropertyDeclarationBlock {
    // https://drafts.csswg.org/cssom/#serialize-a-css-declaration-block
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut is_first_serialization = true; // trailing serializations should have a prepended space

        // Step 1 -> dest = result list

        // Step 2
        let mut already_serialized = Vec::new();

        // Step 3
        for &(ref declaration, importance) in &*self.declarations {
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
                let mut longhands = self.declarations.iter()
                    .filter(|d| !already_serialized.contains(&d.0.name()))
                    .collect::<Vec<_>>();

                // Step 3.3.2
                for shorthand in shorthands {
                    let properties = shorthand.longhands();

                    // Substep 2 & 3
                    let mut current_longhands = Vec::new();
                    let mut important_count = 0;

                    for &&(ref longhand, longhand_importance) in longhands.iter() {
                        let longhand_name = longhand.name();
                        if properties.iter().any(|p| &longhand_name == *p) {
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
            try!(append_serialization::<W, Cloned<slice::Iter< &PropertyDeclaration>>>(
                dest,
                &property.to_string(),
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

pub enum AppendableValue<'a, I>
where I: Iterator<Item=&'a PropertyDeclaration> {
    Declaration(&'a PropertyDeclaration),
    DeclarationsForShorthand(Shorthand, I),
    Css(&'a str)
}

fn handle_first_serialization<W>(dest: &mut W, is_first_serialization: &mut bool) -> fmt::Result where W: fmt::Write {
    // after first serialization(key: value;) add whitespace between the pairs
    if !*is_first_serialization {
        try!(write!(dest, " "));
    } else {
        *is_first_serialization = false;
    }

    Ok(())
}

pub fn append_declaration_value<'a, W, I>
                           (dest: &mut W,
                            appendable_value: AppendableValue<'a, I>,
                            importance: Importance)
                            -> fmt::Result
                            where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
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

  if importance.important() {
      try!(write!(dest, " !important"));
  }

  Ok(())
}

pub fn append_serialization<'a, W, I>(dest: &mut W,
                                  property_name: &str,
                                  appendable_value: AppendableValue<'a, I>,
                                  importance: Importance,
                                  is_first_serialization: &mut bool)
                                  -> fmt::Result
                                  where W: fmt::Write, I: Iterator<Item=&'a PropertyDeclaration> {
    try!(handle_first_serialization(dest, is_first_serialization));

    // Overflow does not behave like a normal shorthand. When overflow-x and overflow-y are not of equal
    // values, they no longer use the shared property name "overflow" and must be handled differently
    if shorthands::is_overflow_shorthand(&appendable_value) {
        return append_declaration_value(dest, appendable_value, importance);
    }

    try!(write!(dest, "{}:", property_name));

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
         &AppendableValue::DeclarationsForShorthand(..) => try!(write!(dest, " "))
    }

    try!(append_declaration_value(dest, appendable_value, importance));
    write!(dest, ";")
}

pub fn parse_style_attribute(input: &str, base_url: &Url, error_reporter: StdBox<ParseErrorReporter + Send>,
                             extra_data: ParserContextExtraData)
                             -> PropertyDeclarationBlock {
    let context = ParserContext::new_with_extra_data(Origin::Author, base_url, error_reporter, extra_data);
    parse_property_declaration_list(&context, &mut Parser::new(input))
}

pub fn parse_one_declaration(name: &str, input: &str, base_url: &Url, error_reporter: StdBox<ParseErrorReporter + Send>,
                             extra_data: ParserContextExtraData)
                             -> Result<Vec<PropertyDeclaration>, ()> {
    let context = ParserContext::new_with_extra_data(Origin::Author, base_url, error_reporter, extra_data);
    let mut results = vec![];
    match PropertyDeclaration::parse(name, &context, &mut Parser::new(input), &mut results, false) {
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
    type AtRule = (Vec<PropertyDeclaration>, Importance);
}


impl<'a, 'b> DeclarationParser for PropertyDeclarationParser<'a, 'b> {
    type Declaration = (Vec<PropertyDeclaration>, Importance);

    fn parse_value(&mut self, name: &str, input: &mut Parser)
                   -> Result<(Vec<PropertyDeclaration>, Importance), ()> {
        let mut results = vec![];
        try!(input.parse_until_before(Delimiter::Bang, |input| {
            match PropertyDeclaration::parse(name, self.context, input, &mut results, false) {
                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(()),
                _ => Err(())
            }
        }));
        let importance = match input.try(parse_important) {
            Ok(()) => Importance::Important,
            Err(()) => Importance::Normal,
        };
        Ok((results, importance))
    }
}


pub fn parse_property_declaration_list(context: &ParserContext, input: &mut Parser)
                                       -> PropertyDeclarationBlock {
    let mut declarations = Vec::new();
    let mut important_count = 0;
    let parser = PropertyDeclarationParser {
        context: context,
    };
    let mut iter = DeclarationListParser::new(input, parser);
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok((results, importance)) => {
                if importance.important() {
                    important_count += results.len() as u32;
                }
                declarations.extend(results.into_iter().map(|d| (d, importance)))
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
        declarations: declarations,
        important_count: important_count,
    };
    super::deduplicate_property_declarations(&mut block);
    block
}

