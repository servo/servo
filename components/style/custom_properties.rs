/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, Token, SourcePosition, Delimiter, TokenSerializationType};
use properties::DeclaredValue;
use std::ascii::AsciiExt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use string_cache::Atom;
use util::mem::HeapSizeOf;

// Does not include the `--` prefix
pub type Name = Atom;

// https://drafts.csswg.org/css-variables/#typedef-custom-property-name
pub fn parse_name(s: &str) -> Result<Name, ()> {
    if s.starts_with("--") {
        Ok(Atom::from_slice(&s[2..]))
    } else {
        Err(())
    }
}

#[derive(Clone, PartialEq)]
pub struct SpecifiedValue {
    css: String,

    first_token: TokenSerializationType,
    last_token: TokenSerializationType,

    /// Custom property names in var() functions.
    references: HashSet<Name>,
}

pub struct BorrowedSpecifiedValue<'a> {
    css: &'a str,
    first_token: TokenSerializationType,
    last_token: TokenSerializationType,
    references: Option<&'a HashSet<Name>>,
}

#[derive(Clone, HeapSizeOf)]
pub struct ComputedValue {
    css: String,
    first_token: TokenSerializationType,
    last_token: TokenSerializationType,
}

pub type ComputedValuesMap = HashMap<Name, ComputedValue>;

pub fn parse(input: &mut Parser) -> Result<SpecifiedValue, ()> {
    let start = input.position();
    let mut references = Some(HashSet::new());
    let (first_token, last_token) = try!(parse_declaration_value(input, &mut references));
    Ok(SpecifiedValue {
        css: input.slice_from(start).to_owned(),
        first_token: first_token,
        last_token: last_token,
        references: references.unwrap(),
    })
}

/// https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value
pub fn parse_declaration_value(input: &mut Parser, references: &mut Option<HashSet<Name>>)
                               -> Result<(TokenSerializationType, TokenSerializationType), ()> {
    input.parse_until_before(Delimiter::Bang | Delimiter::Semicolon, |input| {
        // Need at least one token
        let start_position = input.position();
        try!(input.next_including_whitespace());
        input.reset(start_position);

        parse_declaration_value_block(input, references)
    })
}

/// Like parse_declaration_value,
/// but accept `!` and `;` since they are only invalid at the top level
fn parse_declaration_value_block(input: &mut Parser, references: &mut Option<HashSet<Name>>)
                                 -> Result<(TokenSerializationType, TokenSerializationType), ()> {
    let mut first_token_type = TokenSerializationType::nothing();
    let mut last_token_type = TokenSerializationType::nothing();
    while let Ok(token) = input.next_including_whitespace_and_comments() {
        first_token_type.set_if_nothing(token.serialization_type());
        // This may be OpenParen when it should be Other (for the closing paren)
        // but that doesn’t make a difference since OpenParen is only special
        // when it comes *after* an identifier (it would turn into a function)
        // but a "last" token will only be concantenated *before* another unrelated token.
        last_token_type = token.serialization_type();
        match token {
            Token::BadUrl |
            Token::BadString |
            Token::CloseParenthesis |
            Token::CloseSquareBracket |
            Token::CloseCurlyBracket => {
                return Err(())
            }

            Token::Function(ref name) if name.eq_ignore_ascii_case("var") => {
                try!(input.parse_nested_block(|input| {
                    parse_var_function(input, references)
                }));
            }

            Token::Function(_) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                try!(input.parse_nested_block(|input| {
                    parse_declaration_value_block(input, references)
                }));
            }

            _ => {}
        }
    }
    Ok((first_token_type, last_token_type))
}

// If the var function is valid, return Ok((custom_property_name, fallback))
fn parse_var_function<'i, 't>(input: &mut Parser<'i, 't>, references: &mut Option<HashSet<Name>>)
                              -> Result<(), ()> {
    let name = try!(input.expect_ident());
    let name = try!(parse_name(&name));
    if input.expect_comma().is_ok() {
        try!(parse_declaration_value(input, references));
    }
    if let Some(ref mut refs) = *references {
        refs.insert(name);
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
                            first_token: inherited_value.first_token,
                            last_token: inherited_value.last_token,
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
                    first_token: specified_value.first_token,
                    last_token: specified_value.last_token,
                    references: Some(&specified_value.references),
                });
            },
            DeclaredValue::WithVariables { .. } => unreachable!(),
            DeclaredValue::Initial => {
                map.remove(&name);
            }
            DeclaredValue::Inherit => {}  // The inherited value is what we already have.
        }
    }
}

pub fn finish_cascade(custom_properties: Option<HashMap<&Name, BorrowedSpecifiedValue>>,
                      inherited: &Option<Arc<HashMap<Name, ComputedValue>>>)
                      -> Option<Arc<HashMap<Name, ComputedValue>>> {
    if let Some(mut map) = custom_properties {
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
fn substitute_all(custom_properties: HashMap<&Name, BorrowedSpecifiedValue>,
                  inherited: &Option<Arc<HashMap<Name, ComputedValue>>>)
                  -> HashMap<Name, ComputedValue> {
    let mut substituted_map = HashMap::new();
    let mut invalid = HashSet::new();
    for (&name, value) in &custom_properties {
        // If this value is invalid at computed-time it won’t be inserted in substituted_map.
        // Nothing else to do.
        let _ = substitute_one(
            name, value, &custom_properties, inherited, None, &mut substituted_map, &mut invalid);
    }
    substituted_map
}

/// Replace `var()` functions for one custom property.
/// Also recursively record results for other custom properties referenced by `var()` functions.
/// Return `Err(())` for invalid at computed time.
fn substitute_one(name: &Name,
                  specified_value: &BorrowedSpecifiedValue,
                  custom_properties: &HashMap<&Name, BorrowedSpecifiedValue>,
                  inherited: &Option<Arc<HashMap<Name, ComputedValue>>>,
                  substituted: Option<&mut String>,
                  substituted_map: &mut HashMap<Name, ComputedValue>,
                  invalid: &mut HashSet<Name>)
                  -> Result<(), ()> {
    if let Some(computed_value) = substituted_map.get(name) {
        if let Some(substituted) = substituted {
            substituted.push_str(&computed_value.css)
        }
        return Ok(())
    }

    if invalid.contains(name) {
        return Err(());
    }
    let computed_value = if specified_value.references.map(|set| set.is_empty()) == Some(false) {
        let mut substituted = String::new();
        let mut input = Parser::new(&specified_value.css);
        let mut start = input.position();
        if substitute_block(&mut input, &mut start, &mut substituted, &mut |name, substituted| {
            if let Some(other_specified_value) = custom_properties.get(name) {
                substitute_one(name, other_specified_value, custom_properties, inherited,
                               Some(substituted), substituted_map, invalid)
            } else {
                Err(())
            }
        }).is_ok() {
            substituted.push_str(input.slice_from(start));
            ComputedValue {
                css: substituted,
                // FIXME: what if these are `var(` or the corresponding `)`?
                first_token: specified_value.first_token,
                last_token: specified_value.last_token,
            }
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
            first_token: specified_value.first_token,
            last_token: specified_value.last_token,
        }
    };
    if let Some(substituted) = substituted {
        substituted.push_str(&computed_value.css)
    }
    substituted_map.insert(name.clone(), computed_value);
    Ok(())
}

/// Replace `var()` functions in an arbitrary bit of input.
///
/// The `substitute_one` callback is called for each `var()` function in `input`.
/// If the variable has its initial value,
/// the callback should return `Err(())` and leave `substituted` unchanged.
/// Otherwise, it should push the value of the variable (with its own `var()` functions replaced)
/// to `substituted` and return `Ok(())`.
///
/// Return `Err(())` if `input` is invalid at computed-value time.
fn substitute_block<F>(input: &mut Parser,
                       start: &mut SourcePosition,
                       substituted: &mut String,
                       substitute_one: &mut F)
                       -> Result<(), ()>
                       where F: FnMut(&Name, &mut String) -> Result<(), ()> {
    loop {
        let before_this_token = input.position();
        let token = if let Ok(token) = input.next() { token } else { break };
        match token {
            Token::Function(ref name) if name.eq_ignore_ascii_case("var") => {
                substituted.push_str(input.slice(*start..before_this_token));
                try!(input.parse_nested_block(|input| {
                    // parse_var_function() ensures neither .unwrap() will fail.
                    let name = input.expect_ident().unwrap();
                    let name = parse_name(&name).unwrap();

                    if substitute_one(&name, substituted).is_ok() {
                        // Skip over the fallback, as `parse_nested_block` would return `Err`
                        // if we don’t consume all of `input`.
                        // FIXME: Add a specialized method to cssparser to do this with less work.
                        while let Ok(_) = input.next() {}
                    } else {
                        try!(input.expect_comma());
                        let mut start = input.position();
                        try!(substitute_block(input, &mut start, substituted, substitute_one));
                        substituted.push_str(input.slice_from(start));
                    }
                    Ok(())
                }));
                *start = input.position();
            }

            Token::Function(_) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                try!(input.parse_nested_block(|input| {
                    substitute_block(input, start, substituted, substitute_one)
                }));
            }

            _ => {}
        }
    }
    // FIXME: deal with things being implicitly closed at the end of the input. E.g.
    // ```html
    // <div style="--color: rgb(0,0,0">
    // <p style="background: var(--color) var(--image) top left; --image: url('a.png"></p>
    // </div>
    // ```
    Ok(())
}

/// Replace `var()` functions for a non-custom property.
/// Return `Err(())` for invalid at computed time.
pub fn substitute(input: &str, custom_properties: &Option<Arc<HashMap<Name, ComputedValue>>>)
                  -> Result<String, ()> {
    let empty_map;
    let custom_properties = if let &Some(ref arc) = custom_properties {
        &**arc
    } else {
        empty_map = HashMap::new();
        &empty_map
    };
    let mut substituted = String::new();
    let mut input = Parser::new(input);
    let mut start = input.position();
    try!(substitute_block(&mut input, &mut start, &mut substituted, &mut |name, substituted| {
        if let Some(value) = custom_properties.get(name) {
            substituted.push_str(&value.css);
            Ok(())
        } else {
            Err(())
        }
    }));
    substituted.push_str(input.slice_from(start));
    Ok(substituted)
}
