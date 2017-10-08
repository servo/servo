/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Support for [custom properties for cascading variables][custom].
//!
//! [custom]: https://drafts.csswg.org/css-variables/

use Atom;
use cssparser::{Delimiter, Parser, ParserInput, SourcePosition, Token, TokenSerializationType};
use precomputed_hash::PrecomputedHash;
use properties::{CSSWideKeyword, DeclaredValue};
use selector_map::{PrecomputedHashSet, PrecomputedDiagnosticHashMap};
use selectors::parser::SelectorParseError;
use servo_arc::Arc;
use std::ascii::AsciiExt;
use std::borrow::{Borrow, Cow};
use std::fmt;
use std::hash::Hash;
use style_traits::{ToCss, StyleParseError, ParseError};

/// A custom property name is just an `Atom`.
///
/// Note that this does not include the `--` prefix
pub type Name = Atom;

/// Parse a custom property name.
///
/// https://drafts.csswg.org/css-variables/#typedef-custom-property-name
pub fn parse_name(s: &str) -> Result<&str, ()> {
    if s.starts_with("--") {
        Ok(&s[2..])
    } else {
        Err(())
    }
}

/// A specified value for a custom property is just a set of tokens.
///
/// We preserve the original CSS for serialization, and also the variable
/// references to other custom property names.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SpecifiedValue {
    css: String,

    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,

    /// Custom property names in var() functions.
    references: PrecomputedHashSet<Name>,
}

/// This struct is a cheap borrowed version of a `SpecifiedValue`.
pub struct BorrowedSpecifiedValue<'a> {
    css: &'a str,
    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,
    references: &'a PrecomputedHashSet<Name>,
}

/// A computed value is just a set of tokens as well, until we resolve variables
/// properly.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ComputedValue {
    css: String,
    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,
}

impl ToCss for SpecifiedValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        dest.write_str(&self.css)
    }
}

impl ToCss for ComputedValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        dest.write_str(&self.css)
    }
}

/// A map from CSS variable names to CSS variable computed values, used for
/// resolving.
///
/// A consistent ordering is required for CSSDeclaration objects in the
/// DOM. CSSDeclarations expose property names as indexed properties, which
/// need to be stable. So we keep an array of property names which order is
/// determined on the order that they are added to the name-value map.
pub type CustomPropertiesMap = OrderedMap<Name, ComputedValue>;

/// A map that preserves order for the keys, and that is easily indexable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedMap<K, V>
where
    K: PrecomputedHash + Hash + Eq + Clone,
{
    /// Key index.
    index: Vec<K>,
    /// Key-value map.
    values: PrecomputedDiagnosticHashMap<K, V>,
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + PrecomputedHash + Hash + Clone,
{
    /// Creates a new ordered map.
    pub fn new() -> Self {
        OrderedMap {
            index: Vec::new(),
            values: PrecomputedDiagnosticHashMap::default(),
        }
    }

    /// Insert a new key-value pair.
    pub fn insert(&mut self, key: K, value: V) {
        if !self.values.contains_key(&key) {
            self.index.push(key.clone());
        }
        self.values.begin_mutation();
        self.values.try_insert(key, value).unwrap();
        self.values.end_mutation();
    }

    /// Get a value given its key.
    pub fn get(&self, key: &K) -> Option<&V> {
        let value = self.values.get(key);
        debug_assert_eq!(value.is_some(), self.index.contains(key));
        value
    }

    /// Get the key located at the given index.
    pub fn get_key_at(&self, index: u32) -> Option<&K> {
        self.index.get(index as usize)
    }

    /// Get an ordered map iterator.
    pub fn iter<'a>(&'a self) -> OrderedMapIterator<'a, K, V> {
        OrderedMapIterator {
            inner: self,
            pos: 0,
        }
    }

    /// Get the count of items in the map.
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.values.len(), self.index.len());
        self.values.len()
    }

    /// Returns whether this map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove an item given its key.
    fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: PrecomputedHash + Hash + Eq,
    {
        let index = match self.index.iter().position(|k| k.borrow() == key) {
            Some(p) => p,
            None => return None,
        };
        self.index.remove(index);
        self.values.begin_mutation();
        let result = self.values.remove(key);
        self.values.end_mutation();
        result
    }
}

/// An iterator for OrderedMap.
///
/// The iteration order is determined by the order that the values are
/// added to the key-value map.
pub struct OrderedMapIterator<'a, K, V>
where
    K: 'a + Eq + PrecomputedHash + Hash + Clone, V: 'a,
{
    /// The OrderedMap itself.
    inner: &'a OrderedMap<K, V>,
    /// The position of the iterator.
    pos: usize,
}

impl<'a, K, V> Iterator for OrderedMapIterator<'a, K, V>
where
    K: Eq + PrecomputedHash + Hash + Clone,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let key = match self.inner.index.get(self.pos) {
            Some(k) => k,
            None => return None,
        };

        self.pos += 1;
        let value = &self.inner.values.get(key).unwrap();

        Some((key, value))
    }
}

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

impl SpecifiedValue {
    /// Parse a custom property SpecifiedValue.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
    ) -> Result<Box<Self>, ParseError<'i>> {
        let mut references = PrecomputedHashSet::default();

        let (first_token_type, css, last_token_type) =
            parse_self_contained_declaration_value(input, Some(&mut references))?;

        Ok(Box::new(SpecifiedValue {
            css: css.into_owned(),
            first_token_type,
            last_token_type,
            references
        }))
    }
}

/// Parse the value of a non-custom property that contains `var()` references.
pub fn parse_non_custom_with_var<'i, 't>
                                (input: &mut Parser<'i, 't>)
                                -> Result<(TokenSerializationType, Cow<'i, str>), ParseError<'i>> {
    let (first_token_type, css, _) = parse_self_contained_declaration_value(input, None)?;
    Ok((first_token_type, css))
}

fn parse_self_contained_declaration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    references: Option<&mut PrecomputedHashSet<Name>>
) -> Result<
    (TokenSerializationType, Cow<'i, str>, TokenSerializationType),
    ParseError<'i>
>
{
    let start_position = input.position();
    let mut missing_closing_characters = String::new();
    let (first, last) = parse_declaration_value(input, references, &mut missing_closing_characters)?;
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
fn parse_declaration_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    references: Option<&mut PrecomputedHashSet<Name>>,
    missing_closing_characters: &mut String
) -> Result<(TokenSerializationType, TokenSerializationType), ParseError<'i>> {
    input.parse_until_before(Delimiter::Bang | Delimiter::Semicolon, |input| {
        // Need at least one token
        let start = input.state();
        input.next_including_whitespace()?;
        input.reset(&start);

        parse_declaration_value_block(input, references, missing_closing_characters)
    })
}

/// Like parse_declaration_value, but accept `!` and `;` since they are only
/// invalid at the top level
fn parse_declaration_value_block<'i, 't>(
    input: &mut Parser<'i, 't>,
    mut references: Option<&mut PrecomputedHashSet<Name>>,
    missing_closing_characters: &mut String
) -> Result<(TokenSerializationType, TokenSerializationType), ParseError<'i>> {
    let mut token_start = input.position();
    let mut token = match input.next_including_whitespace_and_comments() {
        // FIXME: remove clone() when borrows are non-lexical
        Ok(token) => token.clone(),
        Err(_) => return Ok((TokenSerializationType::nothing(), TokenSerializationType::nothing()))
    };
    let first_token_type = token.serialization_type();
    loop {
        macro_rules! nested {
            () => {
                input.parse_nested_block(|input| {
                    parse_declaration_value_block(
                        input,
                        references.as_mut().map(|r| &mut **r),
                        missing_closing_characters
                    )
                })?
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
            Token::BadUrl(u) =>
                return Err(StyleParseError::BadUrlInDeclarationValueBlock(u).into()),
            Token::BadString(s) =>
                return Err(StyleParseError::BadStringInDeclarationValueBlock(s).into()),
            Token::CloseParenthesis =>
                return Err(StyleParseError::UnbalancedCloseParenthesisInDeclarationValueBlock.into()),
            Token::CloseSquareBracket =>
                return Err(StyleParseError::UnbalancedCloseSquareBracketInDeclarationValueBlock.into()),
            Token::CloseCurlyBracket =>
                return Err(StyleParseError::UnbalancedCloseCurlyBracketInDeclarationValueBlock.into()),
            Token::Function(ref name) => {
                if name.eq_ignore_ascii_case("var") {
                    let args_start = input.state();
                    input.parse_nested_block(|input| {
                        parse_var_function(
                            input,
                            references.as_mut().map(|r| &mut **r),
                        )
                    })?;
                    input.reset(&args_start);
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
            Token::Dimension { unit: ref value, .. } => {
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
        token = match input.next_including_whitespace_and_comments() {
            // FIXME: remove clone() when borrows are non-lexical
            Ok(token) => token.clone(),
            Err(..) => return Ok((first_token_type, last_token_type)),
        };
    }
}

// If the var function is valid, return Ok((custom_property_name, fallback))
fn parse_var_function<'i, 't>(
    input: &mut Parser<'i, 't>,
    references: Option<&mut PrecomputedHashSet<Name>>
) -> Result<(), ParseError<'i>> {
    let name = input.expect_ident_cloned()?;
    let name: Result<_, ParseError> =
        parse_name(&name)
        .map_err(|()| SelectorParseError::UnexpectedIdent(name.clone()).into());
    let name = name?;
    if input.try(|input| input.expect_comma()).is_ok() {
        // Exclude `!` and `;` at the top level
        // https://drafts.csswg.org/css-syntax/#typedef-declaration-value
        input.parse_until_before(Delimiter::Bang | Delimiter::Semicolon, |input| {
            // At least one non-comment token.
            input.next_including_whitespace()?;
            // Skip until the end.
            while let Ok(_) = input.next_including_whitespace_and_comments() {}
            Ok(())
        })?;
    }
    if let Some(refs) = references {
        refs.insert(Atom::from(name));
    }
    Ok(())
}

/// A struct that takes care of encapsulating the cascade process for custom
/// properties.
pub struct CustomPropertiesBuilder<'a> {
    seen: PrecomputedHashSet<&'a Name>,
    may_have_cycles: bool,
    specified_custom_properties: OrderedMap<&'a Name, Option<BorrowedSpecifiedValue<'a>>>,
    inherited: Option<&'a Arc<CustomPropertiesMap>>,
}

impl<'a> CustomPropertiesBuilder<'a> {
    /// Create a new builder, inheriting from a given custom properties map.
    pub fn new(inherited: Option<&'a Arc<CustomPropertiesMap>>) -> Self {
        Self {
            seen: PrecomputedHashSet::default(),
            may_have_cycles: false,
            specified_custom_properties: OrderedMap::new(),
            inherited,
        }
    }

    /// Cascade a given custom property declaration.
    pub fn cascade(
        &mut self,
        name: &'a Name,
        specified_value: DeclaredValue<'a, Box<SpecifiedValue>>,
    ) {
        let was_already_present = !self.seen.insert(name);
        if was_already_present {
            return;
        }

        match specified_value {
            DeclaredValue::Value(ref specified_value) => {
                self.may_have_cycles |= !specified_value.references.is_empty();
                self.specified_custom_properties.insert(name, Some(BorrowedSpecifiedValue {
                    css: &specified_value.css,
                    first_token_type: specified_value.first_token_type,
                    last_token_type: specified_value.last_token_type,
                    references: &specified_value.references,
                }));
            },
            DeclaredValue::WithVariables(_) => unreachable!(),
            DeclaredValue::CSSWideKeyword(keyword) => match keyword {
                CSSWideKeyword::Initial => {
                    self.specified_custom_properties.insert(name, None);
                }
                CSSWideKeyword::Unset | // Custom properties are inherited by default.
                CSSWideKeyword::Inherit => {} // The inherited value is what we already have.
            }
        }
    }

    /// Returns the final map of applicable custom properties.
    ///
    /// If there was any specified property, we've created a new map and now we need
    /// to remove any potential cycles, and wrap it in an arc.
    ///
    /// Otherwise, just use the inherited custom properties map.
    pub fn build(mut self) -> Option<Arc<CustomPropertiesMap>> {
        if self.specified_custom_properties.is_empty() {
            return self.inherited.cloned();
        }

        if self.may_have_cycles {
            remove_cycles(&mut self.specified_custom_properties);
        }
        Some(Arc::new(substitute_all(self.specified_custom_properties, self.inherited)))
    }
}

/// https://drafts.csswg.org/css-variables/#cycles
///
/// The initial value of a custom property is represented by this property not
/// being in the map.
fn remove_cycles(map: &mut OrderedMap<&Name, Option<BorrowedSpecifiedValue>>) {
    let mut to_remove = PrecomputedHashSet::default();
    {
        let mut visited = PrecomputedHashSet::default();
        let mut stack = Vec::new();
        for name in &map.index {
            walk(map, name, &mut stack, &mut visited, &mut to_remove);

            fn walk<'a>(
                map: &OrderedMap<&'a Name, Option<BorrowedSpecifiedValue<'a>>>,
                name: &'a Name,
                stack: &mut Vec<&'a Name>,
                visited: &mut PrecomputedHashSet<&'a Name>,
                to_remove: &mut PrecomputedHashSet<Name>,
            ) {
                let already_visited_before = !visited.insert(name);
                if already_visited_before {
                    return
                }
                if let Some(&Some(ref value)) = map.get(&name) {
                    if !value.references.is_empty() {
                        stack.push(name);
                        for next in value.references {
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
    for name in to_remove {
        map.remove(&name);
    }
}

/// Replace `var()` functions for all custom properties.
fn substitute_all(
    specified_values_map: OrderedMap<&Name, Option<BorrowedSpecifiedValue>>,
    inherited: Option<&Arc<CustomPropertiesMap>>,
) -> CustomPropertiesMap {
    let mut custom_properties_map = match inherited {
        None => CustomPropertiesMap::new(),
        Some(inherited) => (**inherited).clone(),
    };

    let mut invalid = PrecomputedHashSet::default();

    for (name, value) in specified_values_map.iter() {
        let value = match *value {
            Some(ref v) => v,
            None => {
                custom_properties_map.remove(*name);
                continue;
            }
        };

        let _ = substitute_one(
            name,
            value,
            &specified_values_map,
            None,
            &mut custom_properties_map,
            &mut invalid,
        );
    }

    custom_properties_map
}

/// Replace `var()` functions for one custom property.
/// Also recursively record results for other custom properties referenced by `var()` functions.
/// Return `Err(())` for invalid at computed time.
/// or `Ok(last_token_type that was pushed to partial_computed_value)` otherwise.
fn substitute_one(
    name: &Name,
    specified_value: &BorrowedSpecifiedValue,
    specified_values_map: &OrderedMap<&Name, Option<BorrowedSpecifiedValue>>,
    partial_computed_value: Option<&mut ComputedValue>,
    custom_properties_map: &mut CustomPropertiesMap,
    invalid: &mut PrecomputedHashSet<Name>,
) -> Result<TokenSerializationType, ()> {
    if invalid.contains(name) {
        return Err(());
    }

    let computed_value = if specified_value.references.is_empty() {
        // The specified value contains no var() reference
        ComputedValue {
            css: specified_value.css.to_owned(),
            first_token_type: specified_value.first_token_type,
            last_token_type: specified_value.last_token_type,
        }
    } else {
        let mut partial_computed_value = ComputedValue::empty();
        let mut input = ParserInput::new(&specified_value.css);
        let mut input = Parser::new(&mut input);
        let mut position = (input.position(), specified_value.first_token_type);
        let result = substitute_block(
            &mut input, &mut position, &mut partial_computed_value,
            &mut |name, partial_computed_value| {
                if let Some(other_specified_value) = specified_values_map.get(&name).and_then(|v| v.as_ref()) {
                    substitute_one(
                        name,
                        other_specified_value,
                        specified_values_map,
                        Some(partial_computed_value),
                        custom_properties_map,
                        invalid,
                    )
                } else {
                    Err(())
                }
            }
        );
        if let Ok(last_token_type) = result {
            partial_computed_value.push_from(position, &input, last_token_type);
            partial_computed_value
        } else {
            invalid.insert(name.clone());
            return Err(())
        }
    };
    if let Some(partial_computed_value) = partial_computed_value {
        partial_computed_value.push_variable(&computed_value)
    }
    let last_token_type = computed_value.last_token_type;
    custom_properties_map.insert(name.clone(), computed_value);
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
fn substitute_block<'i, 't, F>(
    input: &mut Parser<'i, 't>,
    position: &mut (SourcePosition, TokenSerializationType),
    partial_computed_value: &mut ComputedValue,
    substitute_one: &mut F
) -> Result<TokenSerializationType, ParseError<'i>>
where
    F: FnMut(&Name, &mut ComputedValue) -> Result<TokenSerializationType, ()>
{
    let mut last_token_type = TokenSerializationType::nothing();
    let mut set_position_at_next_iteration = false;
    loop {
        let before_this_token = input.position();
        // FIXME: remove clone() when borrows are non-lexical
        let next = input.next_including_whitespace_and_comments().map(|t| t.clone());
        if set_position_at_next_iteration {
            *position = (before_this_token, match next {
                Ok(ref token) => token.serialization_type(),
                Err(_) => TokenSerializationType::nothing(),
            });
            set_position_at_next_iteration = false;
        }
        let token = match next {
            Ok(token) => token,
            Err(..) => break,
        };
        match token {
            Token::Function(ref name) if name.eq_ignore_ascii_case("var") => {
                partial_computed_value.push(
                    input.slice(position.0..before_this_token), position.1, last_token_type);
                input.parse_nested_block(|input| {
                    // parse_var_function() ensures neither .unwrap() will fail.
                    let name = input.expect_ident_cloned().unwrap();
                    let name = Atom::from(parse_name(&name).unwrap());

                    if let Ok(last) = substitute_one(&name, partial_computed_value) {
                        last_token_type = last;
                        // Skip over the fallback, as `parse_nested_block` would return `Err`
                        // if we don’t consume all of `input`.
                        // FIXME: Add a specialized method to cssparser to do this with less work.
                        while let Ok(_) = input.next() {}
                    } else {
                        input.expect_comma()?;
                        let after_comma = input.state();
                        let first_token_type = input.next_including_whitespace_and_comments()
                            // parse_var_function() ensures that .unwrap() will not fail.
                            .unwrap()
                            .serialization_type();
                        input.reset(&after_comma);
                        let mut position = (after_comma.position(), first_token_type);
                        last_token_type = substitute_block(
                            input, &mut position, partial_computed_value, substitute_one)?;
                        partial_computed_value.push_from(position, input, last_token_type);
                    }
                    Ok(())
                })?;
                set_position_at_next_iteration = true
            }

            Token::Function(_) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                input.parse_nested_block(|input| {
                    substitute_block(input, position, partial_computed_value, substitute_one)
                })?;
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
pub fn substitute<'i>(
    input: &'i str,
    first_token_type: TokenSerializationType,
    computed_values_map: Option<&Arc<CustomPropertiesMap>>,
) -> Result<String, ParseError<'i>> {
    let mut substituted = ComputedValue::empty();
    let mut input = ParserInput::new(input);
    let mut input = Parser::new(&mut input);
    let mut position = (input.position(), first_token_type);
    let last_token_type = substitute_block(
        &mut input, &mut position, &mut substituted, &mut |name, substituted| {
            if let Some(value) = computed_values_map.and_then(|map| map.get(name)) {
                substituted.push_variable(value);
                Ok(value.last_token_type)
            } else {
                Err(())
            }
        }
    )?;
    substituted.push_from(position, &input, last_token_type);
    Ok(substituted.css)
}
