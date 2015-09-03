/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, Token, SourcePosition};
use properties::DeclaredValue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use string_cache::Atom;

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
pub struct Value {
    /// In CSS syntax
    value: String,

    /// Custom property names in var() functions.
    references: HashSet<Name>,
}

pub struct BorrowedValue<'a> {
    value: &'a str,
    references: Option<&'a HashSet<Name>>,
}

pub fn parse(input: &mut Parser) -> Result<Value, ()> {
    let start = input.position();
    let mut references = Some(HashSet::new());
    // FIXME: don’t consume a top-level `!` as that would prevent parsing `!important`.
    // Maybe using Parser::parse_until_before?
    try!(parse_declaration_value(input, &mut references));
    Ok(Value {
        value: input.slice_from(start).to_owned(),
        references: references.unwrap(),
    })
}

/// https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value
pub fn parse_declaration_value(input: &mut Parser, references: &mut Option<HashSet<Name>>)
                               -> Result<(), ()> {
    if input.is_exhausted() {
        // Need at least one token
        return Err(())
    }
    while let Ok(token) = input.next() {
        match token {
            Token::BadUrl |
            Token::BadString |
            Token::CloseParenthesis |
            Token::CloseSquareBracket |
            Token::CloseCurlyBracket |

            Token::Semicolon |
            Token::Delim('!') => {
                return Err(())
            }

            Token::Function(ref name) if name == "var" => {
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
    Ok(())
}

/// Like parse_declaration_value,
/// but accept `!` and `;` since they are only invalid at the top level
fn parse_declaration_value_block(input: &mut Parser, references: &mut Option<HashSet<Name>>)
                                 -> Result<(), ()> {
    while let Ok(token) = input.next() {
        match token {
            Token::BadUrl |
            Token::BadString |
            Token::CloseParenthesis |
            Token::CloseSquareBracket |
            Token::CloseCurlyBracket => {
                return Err(())
            }

            Token::Function(ref name) if name == "var" => {
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
    Ok(())
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
pub fn cascade<'a>(custom_properties: &mut Option<HashMap<&'a Name, BorrowedValue<'a>>>,
                   inherited: &'a Option<Arc<HashMap<Name, String>>>,
                   seen: &mut HashSet<&'a Name>,
                   name: &'a Name,
                   value: &'a DeclaredValue<Value>) {
    let was_not_already_present = seen.insert(name);
    if was_not_already_present {
        let map = match *custom_properties {
            Some(ref mut map) => map,
            None => {
                *custom_properties = Some(match *inherited {
                    Some(ref inherited) => inherited.iter().map(|(key, value)| {
                        (key, BorrowedValue { value: &value, references: None })
                    }).collect(),
                    None => HashMap::new(),
                });
                custom_properties.as_mut().unwrap()
            }
        };
        match *value {
            DeclaredValue::Value(ref value) => {
                map.insert(name, BorrowedValue {
                    value: &value.value,
                    references: Some(&value.references),
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

pub fn finish_cascade(custom_properties: Option<HashMap<&Name, BorrowedValue>>,
                      inherited: &Option<Arc<HashMap<Name, String>>>)
                      -> Option<Arc<HashMap<Name, String>>> {
    if let Some(mut map) = custom_properties {
        remove_cycles(&mut map);
        Some(Arc::new(substitute_all(map, inherited)))
    } else {
        inherited.clone()
    }
}

/// https://drafts.csswg.org/css-variables/#cycles
/// The initial value of a custom property is represented by this property not being in the map.
fn remove_cycles(map: &mut HashMap<&Name, BorrowedValue>) {
    let mut to_remove = HashSet::new();
    {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        for name in map.keys() {
            walk(map, name, &mut stack, &mut visited, &mut to_remove);

            fn walk<'a>(map: &HashMap<&'a Name, BorrowedValue<'a>>,
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
fn substitute_all(custom_properties: HashMap<&Name, BorrowedValue>,
                  inherited: &Option<Arc<HashMap<Name, String>>>)
                  -> HashMap<Name, String> {
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
                  value: &BorrowedValue,
                  custom_properties: &HashMap<&Name, BorrowedValue>,
                  inherited: &Option<Arc<HashMap<Name, String>>>,
                  substituted: Option<&mut String>,
                  substituted_map: &mut HashMap<Name, String>,
                  invalid: &mut HashSet<Name>)
                  -> Result<(), ()> {
    if let Some(value) = substituted_map.get(name) {
        if let Some(substituted) = substituted {
            substituted.push_str(value)
        }
        return Ok(())
    }

    if invalid.contains(name) {
        return Err(());
    }
    let value = if value.references.map(|set| set.is_empty()) == Some(false) {
        let mut substituted = String::new();
        let mut input = Parser::new(&value.value);
        let mut start = input.position();
        if substitute_block(&mut input, &mut start, &mut substituted, &mut |name, substituted| {
            if let Some(value) = custom_properties.get(name) {
                substitute_one(name, value, custom_properties, inherited,
                               Some(substituted), substituted_map, invalid)
            } else {
                Err(())
            }
        }).is_ok() {
            substituted.push_str(input.slice_from(start));
            substituted
        } else {
            // Invalid at computed-value time. Use the inherited value.
            if let Some(value) = inherited.as_ref().and_then(|i| i.get(name)) {
                value.clone()
            } else {
                invalid.insert(name.clone());
                return Err(())
            }
        }
    } else {
        value.value.to_owned()
    };
    if let Some(substituted) = substituted {
        substituted.push_str(&value)
    }
    substituted_map.insert(name.clone(), value);
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
        let input_slice = input.slice_from(*start);
        let token = if let Ok(token) = input.next() { token } else { break };
        match token {
            Token::Function(ref name) if name == "var" => {
                substituted.push_str(input_slice);
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
pub fn substitute(input: &str, custom_properties: &Option<Arc<HashMap<Name, String>>>)
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
            substituted.push_str(value);
            Ok(())
        } else {
            Err(())
        }
    }));
    substituted.push_str(input.slice_from(start));
    Ok(substituted)
}
