/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, Token};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use string_cache::Atom;

#[derive(Clone)]
pub struct Value {
    /// In CSS syntax
    pub value: String,

    /// Custom property names in var() functions
    pub references: HashSet<Atom>,
}

/// Names (atoms) do not include the `--` prefix.
pub type Map = HashMap<Atom, Value>;

pub fn parse(input: &mut Parser) -> Result<Value, ()> {
    let start = input.position();
    let mut references = HashSet::new();
    try!(parse_declaration_value(input, &mut references));
    Ok(Value {
        value: input.slice_from(start).to_owned(),
        references: references,
    })
}

/// https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value
fn parse_declaration_value(input: &mut Parser, references: &mut HashSet<Atom>) -> Result<(), ()> {
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
                try!(parse_var_function(input, references));
            }

            Token::Function(_) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                try!(parse_declaration_value_block(input, references))
            }

            _ => {}
        }
    }
    Ok(())
}

/// Like parse_declaration_value,
/// but accept `!` and `;` since they are only invalid at the top level
fn parse_declaration_value_block(input: &mut Parser, references: &mut HashSet<Atom>)
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
                try!(parse_var_function(input, references));
            }

            Token::Function(_) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                try!(parse_declaration_value_block(input, references))
            }

            _ => {}
        }
    }
    Ok(())
}

// If the var function is valid, return Ok((custom_property_name, fallback))
fn parse_var_function<'i, 't>(input: &mut Parser<'i, 't>, references: &mut HashSet<Atom>)
                              -> Result<(), ()> {
    // https://drafts.csswg.org/css-variables/#typedef-custom-property-name
    let name = try!(input.expect_ident());
    let name = if name.starts_with("--") {
        &name[2..]
    } else {
        return Err(())
    };
    if input.expect_comma().is_ok() {
        try!(parse_declaration_value(input, references));
    }
    references.insert(Atom::from_slice(name));
    Ok(())
}

pub fn cascade(custom_properties: &mut Option<Map>, inherited_custom_properties: &Option<Arc<Map>>,
               name: &Atom, value: &Value) {
    let map = match *custom_properties {
        Some(ref mut map) => map,
        None => {
            *custom_properties = Some(match *inherited_custom_properties {
                Some(ref arc) => (**arc).clone(),
                None => HashMap::new(),
            });
            custom_properties.as_mut().unwrap()
        }
    };
    map.entry(name.clone()).or_insert(value.clone());
}
