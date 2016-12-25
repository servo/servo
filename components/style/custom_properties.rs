/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Support for [custom properties for cascading variables][custom].
//!
//! [custom]: https://drafts.csswg.org/css-variables/

use Atom;
use cssparser::{Delimiter, Parser, SourcePosition, Token, TokenSerializationType};
use parser::{Parse, ParserContext};
use properties::DeclaredValue;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use style_traits::ToCss;

// Does not include the `--` prefix
pub type Name = Atom;

// https://drafts.csswg.org/css-variables/#typedef-custom-property-name
pub fn parse_name(s: &str) -> Result<&str, ()> {
    if s.starts_with("--") {
        Ok(&s[2..])
    } else {
        Err(())
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SpecifiedValue {
    css: String,

    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,

    /// Custom property names in var() functions.
    references: HashSet<Name>,
}

impl ::values::HasViewportPercentage for SpecifiedValue {
    fn has_viewport_percentage(&self) -> bool {
        panic!("has_viewport_percentage called before resolving!");
    }
}

pub struct BorrowedSpecifiedValue<'a> {
    css: &'a str,
    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,
    references: Option<&'a HashSet<Name>>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ComputedValue {
    css: String,
    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,
}

impl ToCss for SpecifiedValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(&self.css)
    }
}

impl ToCss for ComputedValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(&self.css)
    }
}

pub type ComputedValuesMap = HashMap<Name, ComputedValue>;

impl ComputedValue {
    fn empty() -> ComputedValue {
        ComputedValue {
            css: String::new(),
            last_token_type: TokenSerializationType::nothing(),
            first_token_type: TokenSerializationType::nothing(),
        }
    }

    fn push(&mut self, css: &str, css_first_token_type: TokenSerializationType,
            css_last_token_type: TokenSerializationType) {
        // This happens e.g. between to subsequent var() functions: `var(--a)var(--b)`.
        // In that case, css_*_token_type is non-sensical.
        if css.is_empty() {
            return
        }
        self.first_token_type.set_if_nothing(css_first_token_type);
        // If self.first_token_type was nothing,
        // self.last_token_type is also nothing and this will be false:
        if self.last_token_type.needs_separator_when_before(css_first_token_type) {
            self.css.push_str("/**/")
        }
        self.css.push_str(css);
        self.last_token_type = css_last_token_type
    }

    fn push_from(&mut self, position: (SourcePosition, TokenSerializationType),
                 input: &Parser, last_token_type: TokenSerializationType) {
        self.push(input.slice_from(position.0), position.1, last_token_type)
    }

    fn push_variable(&mut self, variable: &ComputedValue) {
        self.push(&variable.css, variable.first_token_type, variable.last_token_type)
    }
}

impl Parse for SpecifiedValue {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut references = Some(HashSet::new());
        let (first, css, last) = try!(parse_self_contained_declaration_value(input, &mut references));
        Ok(SpecifiedValue {
            css: css.into_owned(),
            first_token_type: first,
            last_token_type: last,
            references: references.unwrap(),
        })
    }
}

/// Parse the value of a non-custom property that contains `var()` references.
pub fn parse_non_custom_with_var<'i, 't>
                                (input: &mut Parser<'i, 't>)
                                -> Result<(TokenSerializationType, Cow<'i, str>), ()> {
    let (first_token_type, css, _) = try!(parse_self_contained_declaration_value(input, &mut None));
    Ok((first_token_type, css))
}

fn parse_self_contained_declaration_value<'i, 't>
                                         (input: &mut Parser<'i, 't>,
                                          references: &mut Option<HashSet<Name>>)
                                          -> Result<(
                                              TokenSerializationType,
                                              Cow<'i, str>,
                                              TokenSerializationType
                                          ), ()> {
    let start_position = input.position();
    let mut missing_closing_characters = String::new();
    let (first, last) = try!(
        parse_declaration_value(input, references, &mut missing_closing_characters));
    let mut css: Cow<str> = input.slice_from(start_position).into();
    if !missing_closing_characters.is_empty() {
        // Unescaped backslash at EOF in a quoted string is ignored.
        if css.ends_with("\\") && matches!(missing_closing_characters.as_bytes()[0], b'"' | b'\'') {
            css.to_mut().pop();
        }
        css.to_mut().push_str(&missing_closing_characters);
    }
    Ok((first, css, last))
}

/// https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value
fn parse_declaration_value<'i, 't>
                          (input: &mut Parser<'i, 't>,
                           references: &mut Option<HashSet<Name>>,
                           missing_closing_characters: &mut String)
                          -> Result<(TokenSerializationType, TokenSerializationType), ()> {
    input.parse_until_before(Delimiter::Bang | Delimiter::Semicolon, |input| {
        // Need at least one token
        let start_position = input.position();
        try!(input.next_including_whitespace());
        input.reset(start_position);

        parse_declaration_value_block(input, references, missing_closing_characters)
    })
}

/// Like parse_declaration_value,
/// but accept `!` and `;` since they are only invalid at the top level
fn parse_declaration_value_block(input: &mut Parser,
                                 references: &mut Option<HashSet<Name>>,
                                 missing_closing_characters: &mut String)
                                 -> Result<(TokenSerializationType, TokenSerializationType), ()> {
    let mut token_start = input.position();
    let mut token = match input.next_including_whitespace_and_comments() {
        Ok(token) => token,
        Err(()) => return Ok((TokenSerializationType::nothing(), TokenSerializationType::nothing()))
    };
    let first_token_type = token.serialization_type();
    loop {
        macro_rules! nested {
            () => {
                try!(input.parse_nested_block(|input| {
                    parse_declaration_value_block(input, references, missing_closing_characters)
                }))
            }
        }
        macro_rules! check_closed {
            ($closing: expr) => {
                if !input.slice_from(token_start).ends_with($closing) {
                    missing_closing_characters.push_str($closing)
                }
            }
        }
        let last_token_type = match token {
            Token::Comment(_) => {
                let token_slice = input.slice_from(token_start);
                if !token_slice.ends_with("*/") {
                    missing_closing_characters.push_str(
                        if token_slice.ends_with('*') { "/" } else { "*/" })
                }
                token.serialization_type()
            }
            Token::BadUrl |
            Token::BadString |
            Token::CloseParenthesis |
            Token::CloseSquareBracket |
            Token::CloseCurlyBracket => {
                return Err(())
            }
            Token::Function(ref name) => {
                if name.eq_ignore_ascii_case("var") {
                    let position = input.position();
                    try!(input.parse_nested_block(|input| {
                        parse_var_function(input, references)
                    }));
                    input.reset(position);
                }
                nested!();
                check_closed!(")");
                Token::CloseParenthesis.serialization_type()
            }
            Token::ParenthesisBlock => {
                nested!();
                check_closed!(")");
                Token::CloseParenthesis.serialization_type()
            }
            Token::CurlyBracketBlock => {
                nested!();
                check_closed!("}");
                Token::CloseCurlyBracket.serialization_type()
            }
            Token::SquareBracketBlock => {
                nested!();
                check_closed!("]");
                Token::CloseSquareBracket.serialization_type()
            }
            Token::QuotedString(_) => {
                let token_slice = input.slice_from(token_start);
                let quote = &token_slice[..1];
                debug_assert!(matches!(quote, "\"" | "'"));
                if !(token_slice.ends_with(quote) && token_slice.len() > 1) {
                    missing_closing_characters.push_str(quote)
                }
                token.serialization_type()
            }
            Token::Ident(ref value) |
            Token::AtKeyword(ref value) |
            Token::Hash(ref value) |
            Token::IDHash(ref value) |
            Token::UnquotedUrl(ref value) |
            Token::Dimension(_, ref value) => {
                if value.ends_with("�") && input.slice_from(token_start).ends_with("\\") {
                    // Unescaped backslash at EOF in these contexts is interpreted as U+FFFD
                    // Check the value in case the final backslash was itself escaped.
                    // Serialize as escaped U+FFFD, which is also interpreted as U+FFFD.
                    // (Unescaped U+FFFD would also work, but removing the backslash is annoying.)
                    missing_closing_characters.push_str("�")
                }
                if matches!(token, Token::UnquotedUrl(_)) {
                    check_closed!(")");
                }
                token.serialization_type()
            }
            _ => {
                token.serialization_type()
            }
        };

        token_start = input.position();
        token = if let Ok(token) = input.next_including_whitespace_and_comments() {
            token
        } else {
            return Ok((first_token_type, last_token_type))
        }
    }
}

// If the var function is valid, return Ok((custom_property_name, fallback))
fn parse_var_function<'i, 't>(input: &mut Parser<'i, 't>,
                              references: &mut Option<HashSet<Name>>)
                              -> Result<(), ()> {
    let name = try!(input.expect_ident());
    let name = try!(parse_name(&name));
    if input.try(|input| input.expect_comma()).is_ok() {
        // Exclude `!` and `;` at the top level
        // https://drafts.csswg.org/css-syntax/#typedef-declaration-value
        try!(input.parse_until_before(Delimiter::Bang | Delimiter::Semicolon, |input| {
            // At least one non-comment token.
            try!(input.next_including_whitespace());
            // Skip until the end.
            while let Ok(_) = input.next_including_whitespace_and_comments() {}
            Ok(())
        }));
    }
    if let Some(ref mut refs) = *references {
        refs.insert(Atom::from(name));
    }
    Ok(())
}

/// Add one custom property declaration to a map,
/// unless another with the same name was already there.
pub fn cascade<'a>(custom_properties: &mut Option<HashMap<&'a Name, BorrowedSpecifiedValue<'a>>>,
                   inherited: &'a Option<Arc<HashMap<Name, ComputedValue>>>,
                   seen: &mut HashSet<&'a Name>,
                   name: &'a Name,
                   specified_value: &'a DeclaredValue<SpecifiedValue>) {
    let was_not_already_present = seen.insert(name);
    if was_not_already_present {
        let map = match *custom_properties {
            Some(ref mut map) => map,
            None => {
                *custom_properties = Some(match *inherited {
                    Some(ref inherited) => inherited.iter().map(|(key, inherited_value)| {
                        (key, BorrowedSpecifiedValue {
                            css: &inherited_value.css,
                            first_token_type: inherited_value.first_token_type,
                            last_token_type: inherited_value.last_token_type,
                            references: None
                        })
                    }).collect(),
                    None => HashMap::new(),
                });
                custom_properties.as_mut().unwrap()
            }
        };
        match *specified_value {
            DeclaredValue::Value(ref specified_value) => {
                map.insert(name, BorrowedSpecifiedValue {
                    css: &specified_value.css,
                    first_token_type: specified_value.first_token_type,
                    last_token_type: specified_value.last_token_type,
                    references: Some(&specified_value.references),
                });
            },
            DeclaredValue::WithVariables { .. } => unreachable!(),
            DeclaredValue::Initial => {
                map.remove(&name);
            }
            DeclaredValue::Unset | // Custom properties are inherited by default.
            DeclaredValue::Inherit => {}  // The inherited value is what we already have.
        }
    }
}

pub fn finish_cascade(specified_values_map: Option<HashMap<&Name, BorrowedSpecifiedValue>>,
                      inherited: &Option<Arc<HashMap<Name, ComputedValue>>>)
                      -> Option<Arc<HashMap<Name, ComputedValue>>> {
    if let Some(mut map) = specified_values_map {
        remove_cycles(&mut map);
        Some(Arc::new(substitute_all(map, inherited)))
    } else {
        inherited.clone()
    }
}

/// https://drafts.csswg.org/css-variables/#cycles
/// The initial value of a custom property is represented by this property not being in the map.
fn remove_cycles(map: &mut HashMap<&Name, BorrowedSpecifiedValue>) {
    let mut to_remove = HashSet::new();
    {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        for name in map.keys() {
            walk(map, name, &mut stack, &mut visited, &mut to_remove);

            fn walk<'a>(map: &HashMap<&'a Name, BorrowedSpecifiedValue<'a>>,
                        name: &'a Name,
                        stack: &mut Vec<&'a Name>,
                        visited: &mut HashSet<&'a Name>,
                        to_remove: &mut HashSet<Name>) {
                let already_visited_before = !visited.insert(name);
                if already_visited_before {
                    return
                }
                if let Some(value) = map.get(name) {
                    if let Some(references) = value.references {
                        stack.push(name);
                        for next in references {
                            if let Some(position) = stack.iter().position(|&x| x == next) {
                                // Found a cycle
                                for &in_cycle in &stack[position..] {
                                    to_remove.insert(in_cycle.clone());
                                }
                            } else {
                                walk(map, next, stack, visited, to_remove);
                            }
                        }
                        stack.pop();
                    }
                }
            }
        }
    }
    for name in &to_remove {
        map.remove(name);
    }
}

/// Replace `var()` functions for all custom properties.
fn substitute_all(specified_values_map: HashMap<&Name, BorrowedSpecifiedValue>,
                  inherited: &Option<Arc<HashMap<Name, ComputedValue>>>)
                  -> HashMap<Name, ComputedValue> {
    let mut computed_values_map = HashMap::new();
    let mut invalid = HashSet::new();
    for (&name, value) in &specified_values_map {
        // If this value is invalid at computed-time it won’t be inserted in computed_values_map.
        // Nothing else to do.
        let _ = substitute_one(
            name, value, &specified_values_map, inherited, None,
            &mut computed_values_map, &mut invalid);
    }
    computed_values_map
}

/// Replace `var()` functions for one custom property.
/// Also recursively record results for other custom properties referenced by `var()` functions.
/// Return `Err(())` for invalid at computed time.
/// or `Ok(last_token_type that was pushed to partial_computed_value)` otherwise.
fn substitute_one(name: &Name,
                  specified_value: &BorrowedSpecifiedValue,
                  specified_values_map: &HashMap<&Name, BorrowedSpecifiedValue>,
                  inherited: &Option<Arc<HashMap<Name, ComputedValue>>>,
                  partial_computed_value: Option<&mut ComputedValue>,
                  computed_values_map: &mut HashMap<Name, ComputedValue>,
                  invalid: &mut HashSet<Name>)
                  -> Result<TokenSerializationType, ()> {
    if let Some(computed_value) = computed_values_map.get(name) {
        if let Some(partial_computed_value) = partial_computed_value {
            partial_computed_value.push_variable(computed_value)
        }
        return Ok(computed_value.last_token_type)
    }

    if invalid.contains(name) {
        return Err(());
    }
    let computed_value = if specified_value.references.map(|set| set.is_empty()) == Some(false) {
        let mut partial_computed_value = ComputedValue::empty();
        let mut input = Parser::new(&specified_value.css);
        let mut position = (input.position(), specified_value.first_token_type);
        let result = substitute_block(
            &mut input, &mut position, &mut partial_computed_value,
            &mut |name, partial_computed_value| {
                if let Some(other_specified_value) = specified_values_map.get(name) {
                    substitute_one(name, other_specified_value, specified_values_map, inherited,
                                   Some(partial_computed_value), computed_values_map, invalid)
                } else {
                    Err(())
                }
            }
        );
        if let Ok(last_token_type) = result {
            partial_computed_value.push_from(position, &input, last_token_type);
            partial_computed_value
        } else {
            // Invalid at computed-value time. Use the inherited value.
            if let Some(inherited_value) = inherited.as_ref().and_then(|i| i.get(name)) {
                inherited_value.clone()
            } else {
                invalid.insert(name.clone());
                return Err(())
            }
        }
    } else {
        // The specified value contains no var() reference
        ComputedValue {
            css: specified_value.css.to_owned(),
            first_token_type: specified_value.first_token_type,
            last_token_type: specified_value.last_token_type,
        }
    };
    if let Some(partial_computed_value) = partial_computed_value {
        partial_computed_value.push_variable(&computed_value)
    }
    let last_token_type = computed_value.last_token_type;
    computed_values_map.insert(name.clone(), computed_value);
    Ok(last_token_type)
}

/// Replace `var()` functions in an arbitrary bit of input.
///
/// The `substitute_one` callback is called for each `var()` function in `input`.
/// If the variable has its initial value,
/// the callback should return `Err(())` and leave `partial_computed_value` unchanged.
/// Otherwise, it should push the value of the variable (with its own `var()` functions replaced)
/// to `partial_computed_value` and return `Ok(last_token_type of what was pushed)`
///
/// Return `Err(())` if `input` is invalid at computed-value time.
/// or `Ok(last_token_type that was pushed to partial_computed_value)` otherwise.
fn substitute_block<F>(input: &mut Parser,
                       position: &mut (SourcePosition, TokenSerializationType),
                       partial_computed_value: &mut ComputedValue,
                       substitute_one: &mut F)
                       -> Result<TokenSerializationType, ()>
                       where F: FnMut(&Name, &mut ComputedValue) -> Result<TokenSerializationType, ()> {
    let mut last_token_type = TokenSerializationType::nothing();
    let mut set_position_at_next_iteration = false;
    loop {
        let before_this_token = input.position();
        let next = input.next_including_whitespace_and_comments();
        if set_position_at_next_iteration {
            *position = (before_this_token, match next {
                Ok(ref token) => token.serialization_type(),
                Err(()) => TokenSerializationType::nothing(),
            });
            set_position_at_next_iteration = false;
        }
        let token = if let Ok(token) = next {
            token
        } else {
            break
        };
        match token {
            Token::Function(ref name) if name.eq_ignore_ascii_case("var") => {
                partial_computed_value.push(
                    input.slice(position.0..before_this_token), position.1, last_token_type);
                try!(input.parse_nested_block(|input| {
                    // parse_var_function() ensures neither .unwrap() will fail.
                    let name = input.expect_ident().unwrap();
                    let name = Atom::from(parse_name(&name).unwrap());

                    if let Ok(last) = substitute_one(&name, partial_computed_value) {
                        last_token_type = last;
                        // Skip over the fallback, as `parse_nested_block` would return `Err`
                        // if we don’t consume all of `input`.
                        // FIXME: Add a specialized method to cssparser to do this with less work.
                        while let Ok(_) = input.next() {}
                    } else {
                        try!(input.expect_comma());
                        let position = input.position();
                        let first_token_type = input.next_including_whitespace_and_comments()
                            // parse_var_function() ensures that .unwrap() will not fail.
                            .unwrap()
                            .serialization_type();
                        input.reset(position);
                        let mut position = (position, first_token_type);
                        last_token_type = try!(substitute_block(
                            input, &mut position, partial_computed_value, substitute_one));
                        partial_computed_value.push_from(position, input, last_token_type);
                    }
                    Ok(())
                }));
                set_position_at_next_iteration = true
            }

            Token::Function(_) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                try!(input.parse_nested_block(|input| {
                    substitute_block(input, position, partial_computed_value, substitute_one)
                }));
                // It’s the same type for CloseCurlyBracket and CloseSquareBracket.
                last_token_type = Token::CloseParenthesis.serialization_type();
            }

            _ => last_token_type = token.serialization_type()
        }
    }
    // FIXME: deal with things being implicitly closed at the end of the input. E.g.
    // ```html
    // <div style="--color: rgb(0,0,0">
    // <p style="background: var(--color) var(--image) top left; --image: url('a.png"></p>
    // </div>
    // ```
    Ok(last_token_type)
}

/// Replace `var()` functions for a non-custom property.
/// Return `Err(())` for invalid at computed time.
pub fn substitute(input: &str, first_token_type: TokenSerializationType,
                  computed_values_map: &Option<Arc<HashMap<Name, ComputedValue>>>)
                  -> Result<String, ()> {
    let mut substituted = ComputedValue::empty();
    let mut input = Parser::new(input);
    let mut position = (input.position(), first_token_type);
    let last_token_type = try!(substitute_block(
        &mut input, &mut position, &mut substituted, &mut |name, substituted| {
            if let Some(value) = computed_values_map.as_ref().and_then(|map| map.get(name)) {
                substituted.push_variable(value);
                Ok(value.last_token_type)
            } else {
                Err(())
            }
        }
    ));
    substituted.push_from(position, &input, last_token_type);
    Ok(substituted.css)
}
