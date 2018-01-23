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
use selector_map::{PrecomputedHashSet, PrecomputedHashMap};
use selectors::parser::SelectorParseErrorKind;
use servo_arc::Arc;
use smallvec::SmallVec;
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::borrow::{Borrow, Cow};
use std::cmp;
use std::fmt::{self, Write};
use std::hash::Hash;
use style_traits::{CssWriter, ToCss, StyleParseErrorKind, ParseError};

/// A custom property name is just an `Atom`.
///
/// Note that this does not include the `--` prefix
pub type Name = Atom;

/// Parse a custom property name.
///
/// <https://drafts.csswg.org/css-variables/#typedef-custom-property-name>
pub fn parse_name(s: &str) -> Result<&str, ()> {
    if s.starts_with("--") {
        Ok(&s[2..])
    } else {
        Err(())
    }
}

/// A value for a custom property is just a set of tokens.
///
/// We preserve the original CSS for serialization, and also the variable
/// references to other custom property names.
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
pub struct VariableValue {
    css: String,

    first_token_type: TokenSerializationType,
    last_token_type: TokenSerializationType,

    /// Custom property names in var() functions.
    references: PrecomputedHashSet<Name>,
}

impl ToCss for SpecifiedValue {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
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
///
/// The variable values are guaranteed to not have references to other
/// properties.
pub type CustomPropertiesMap = OrderedMap<Name, Arc<VariableValue>>;

/// Both specified and computed values are VariableValues, the difference is
/// whether var() functions are expanded.
pub type SpecifiedValue = VariableValue;
/// Both specified and computed values are VariableValues, the difference is
/// whether var() functions are expanded.
pub type ComputedValue = VariableValue;

/// A map that preserves order for the keys, and that is easily indexable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedMap<K, V>
where
    K: PrecomputedHash + Hash + Eq + Clone,
{
    /// Key index.
    index: Vec<K>,
    /// Key-value map.
    values: PrecomputedHashMap<K, V>,
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + PrecomputedHash + Hash + Clone,
{
    /// Creates a new ordered map.
    pub fn new() -> Self {
        OrderedMap {
            index: Vec::new(),
            values: PrecomputedHashMap::default(),
        }
    }

    /// Insert a new key-value pair.
    pub fn insert(&mut self, key: K, value: V) {
        if !self.values.contains_key(&key) {
            self.index.push(key.clone());
        }
        self.values.insert(key, value);
    }

    /// Get a value given its key.
    pub fn get(&self, key: &K) -> Option<&V> {
        let value = self.values.get(key);
        debug_assert_eq!(value.is_some(), self.index.contains(key));
        value
    }

    /// Get whether there's a value on the map for `key`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.values.contains_key(key)
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
        let index = self.index.iter().position(|k| k.borrow() == key)?;
        self.index.remove(index);
        self.values.remove(key)
    }

    fn remove_set<S>(&mut self, set: &::hash::HashSet<K, S>)
        where S: ::std::hash::BuildHasher,
    {
        if set.is_empty() {
            return;
        }
        self.index.retain(|key| !set.contains(key));
        self.values.retain(|key, _| !set.contains(key));
        debug_assert_eq!(self.values.len(), self.index.len());
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
        let key = self.inner.index.get(self.pos)?;

        self.pos += 1;
        let value = &self.inner.values[key];

        Some((key, value))
    }
}

impl VariableValue {
    fn empty() -> Self {
        Self {
            css: String::new(),
            last_token_type: TokenSerializationType::nothing(),
            first_token_type: TokenSerializationType::nothing(),
            references: PrecomputedHashSet::default(),
        }
    }

    fn push(
        &mut self,
        css: &str,
        css_first_token_type: TokenSerializationType,
        css_last_token_type: TokenSerializationType
    ) {
        // This happens e.g. between two subsequent var() functions:
        // `var(--a)var(--b)`.
        //
        // In that case, css_*_token_type is nonsensical.
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

    fn push_from(
        &mut self,
        position: (SourcePosition, TokenSerializationType),
        input: &Parser,
        last_token_type: TokenSerializationType
    ) {
        self.push(input.slice_from(position.0), position.1, last_token_type)
    }

    fn push_variable(&mut self, variable: &ComputedValue) {
        debug_assert!(variable.references.is_empty());
        self.push(&variable.css, variable.first_token_type, variable.last_token_type)
    }
}

impl VariableValue {
    /// Parse a custom property value.
    pub fn parse<'i, 't>(
        input: &mut Parser<'i, 't>,
    ) -> Result<Arc<Self>, ParseError<'i>> {
        let mut references = PrecomputedHashSet::default();

        let (first_token_type, css, last_token_type) =
            parse_self_contained_declaration_value(input, Some(&mut references))?;

        Ok(Arc::new(VariableValue {
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

/// <https://drafts.csswg.org/css-syntax-3/#typedef-declaration-value>
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
            Token::BadUrl(u) => {
                return Err(input.new_custom_error(StyleParseErrorKind::BadUrlInDeclarationValueBlock(u)))
            }
            Token::BadString(s) => {
                return Err(input.new_custom_error(StyleParseErrorKind::BadStringInDeclarationValueBlock(s)))
            }
            Token::CloseParenthesis => {
                return Err(input.new_custom_error(
                    StyleParseErrorKind::UnbalancedCloseParenthesisInDeclarationValueBlock
                ))
            }
            Token::CloseSquareBracket => {
                return Err(input.new_custom_error(
                    StyleParseErrorKind::UnbalancedCloseSquareBracketInDeclarationValueBlock
                ))
            }
            Token::CloseCurlyBracket => {
                return Err(input.new_custom_error(
                    StyleParseErrorKind::UnbalancedCloseCurlyBracketInDeclarationValueBlock
                ))
            }
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
        .map_err(|()| input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name.clone())));
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
    custom_properties: Option<CustomPropertiesMap>,
    inherited: Option<&'a Arc<CustomPropertiesMap>>,
}

impl<'a> CustomPropertiesBuilder<'a> {
    /// Create a new builder, inheriting from a given custom properties map.
    pub fn new(inherited: Option<&'a Arc<CustomPropertiesMap>>) -> Self {
        Self {
            seen: PrecomputedHashSet::default(),
            may_have_cycles: false,
            custom_properties: None,
            inherited,
        }
    }

    /// Cascade a given custom property declaration.
    pub fn cascade(
        &mut self,
        name: &'a Name,
        specified_value: DeclaredValue<'a, Arc<SpecifiedValue>>,
    ) {
        let was_already_present = !self.seen.insert(name);
        if was_already_present {
            return;
        }

        if !self.value_may_affect_style(name, &specified_value) {
            return;
        }

        if self.custom_properties.is_none() {
            self.custom_properties = Some(match self.inherited {
                Some(inherited) => (**inherited).clone(),
                None => CustomPropertiesMap::new(),
            });
        }

        let map = self.custom_properties.as_mut().unwrap();
        match specified_value {
            DeclaredValue::Value(ref specified_value) => {
                self.may_have_cycles |= !specified_value.references.is_empty();
                map.insert(name.clone(), (*specified_value).clone());
            },
            DeclaredValue::WithVariables(_) => unreachable!(),
            DeclaredValue::CSSWideKeyword(keyword) => match keyword {
                CSSWideKeyword::Initial => {
                    map.remove(name);
                }
                // handled in value_may_affect_style
                CSSWideKeyword::Unset |
                CSSWideKeyword::Inherit => unreachable!(),
            }
        }
    }

    fn value_may_affect_style(
        &self,
        name: &Name,
        value: &DeclaredValue<Arc<SpecifiedValue>>
    ) -> bool {
        match *value {
            DeclaredValue::CSSWideKeyword(CSSWideKeyword::Unset) |
            DeclaredValue::CSSWideKeyword(CSSWideKeyword::Inherit) => {
                // Custom properties are inherited by default. So
                // explicit 'inherit' or 'unset' means we can just use
                // any existing value in the inherited CustomPropertiesMap.
                return false;
            }
            _ => {}
        }

        let existing_value =
            self.custom_properties.as_ref().and_then(|m| m.get(name))
                .or_else(|| self.inherited.and_then(|m| m.get(name)));

        match (existing_value, value) {
            (None, &DeclaredValue::CSSWideKeyword(CSSWideKeyword::Initial)) => {
                // The initial value of a custom property is the same as it
                // not existing in the map.
                return false;
            }
            (Some(existing_value), &DeclaredValue::Value(specified_value)) => {
                // Don't bother overwriting an existing inherited value with
                // the same specified value.
                if existing_value == specified_value {
                    return false;
                }
            }
            _ => {}
        }

        true
    }

    /// Returns the final map of applicable custom properties.
    ///
    /// If there was any specified property, we've created a new map and now we need
    /// to remove any potential cycles, and wrap it in an arc.
    ///
    /// Otherwise, just use the inherited custom properties map.
    pub fn build(mut self) -> Option<Arc<CustomPropertiesMap>> {
        let mut map = match self.custom_properties.take() {
            Some(m) => m,
            None => return self.inherited.cloned(),
        };

        if self.may_have_cycles {
            substitute_all(&mut map);
        }
        Some(Arc::new(map))
    }
}

/// Resolve all custom properties to either substituted or invalid.
///
/// It does cycle dependencies removal at the same time as substitution.
fn substitute_all(custom_properties_map: &mut CustomPropertiesMap) {
    // The cycle dependencies removal in this function is a variant
    // of Tarjan's algorithm. It is mostly based on the pseudo-code
    // listed in
    // https://en.wikipedia.org/w/index.php?
    // title=Tarjan%27s_strongly_connected_components_algorithm&oldid=801728495
    //
    // FIXME This function currently does at least one addref to names
    // for each variable regardless whether it has reference. Each
    // variable with any reference would have an additional addref.
    // There is another addref for each reference.
    // Strictly speaking, these addrefs are not necessary, because we
    // don't add/remove entry from custom properties map, and thus keys
    // should be alive in the whole process until we start removing
    // invalids. However, there is no safe way for us to prove this to
    // the compiler. We may be able to fix this issue at some point if
    // the standard library can provide some kind of hashmap wrapper
    // with frozen keys.

    /// Struct recording necessary information for each variable.
    struct VarInfo {
        /// The name of the variable. It will be taken to save addref
        /// when the corresponding variable is popped from the stack.
        /// This also serves as a mark for whether the variable is
        /// currently in the stack below.
        name: Option<Name>,
        /// If the variable is in a dependency cycle, lowlink represents
        /// a smaller index which corresponds to a variable in the same
        /// strong connected component, which is known to be accessible
        /// from this variable. It is not necessarily the root, though.
        lowlink: usize,
    }
    /// Context struct for traversing the variable graph, so that we can
    /// avoid referencing all the fields multiple times.
    struct Context<'a> {
        /// Number of variables visited. This is used as the order index
        /// when we visit a new unresolved variable.
        count: usize,
        /// The map from custom property name to its order index.
        index_map: PrecomputedHashMap<Name, usize>,
        /// Information of each variable indexed by the order index.
        var_info: SmallVec<[VarInfo; 5]>,
        /// The stack of order index of visited variables. It contains
        /// all unfinished strong connected components.
        stack: SmallVec<[usize; 5]>,
        map: &'a mut CustomPropertiesMap,
        /// The set of invalid custom properties.
        invalid: &'a mut PrecomputedHashSet<Name>,
    }

    /// This function combines the traversal for cycle removal and value
    /// substitution. It returns either a signal None if this variable
    /// has been fully resolved (to either having no reference or being
    /// marked invalid), or the order index for the given name.
    ///
    /// When it returns, the variable corresponds to the name would be
    /// in one of the following states:
    /// * It is still in context.stack, which means it is part of an
    ///   potentially incomplete dependency circle.
    /// * It has been added into the invalid set. It can be either that
    ///   the substitution failed, or it is inside a dependency circle.
    ///   When this function put a variable into the invalid set because
    ///   of dependency circle, it would put all variables in the same
    ///   strong connected component to the set together.
    /// * It doesn't have any reference, because either this variable
    ///   doesn't have reference at all in specified value, or it has
    ///   been completely resolved.
    /// * There is no such variable at all.
    fn traverse<'a>(name: Name, context: &mut Context<'a>) -> Option<usize> {
        use hash::map::Entry;

        // Some shortcut checks.
        let (name, value) = if let Some(value) = context.map.get(&name) {
            // This variable has been resolved. Return the signal value.
            if value.references.is_empty()  || context.invalid.contains(&name) {
                return None;
            }
            // Whether this variable has been visited in this traversal.
            let key;
            match context.index_map.entry(name) {
                Entry::Occupied(entry) => { return Some(*entry.get()); }
                Entry::Vacant(entry) => {
                    key = entry.key().clone();
                    entry.insert(context.count);
                }
            }
            // Hold a strong reference to the value so that we don't
            // need to keep reference to context.map.
            (key, value.clone())
        } else {
            // The variable doesn't exist at all.
            return None;
        };

        // Add new entry to the information table.
        let index = context.count;
        context.count += 1;
        debug_assert!(index == context.var_info.len());
        context.var_info.push(VarInfo {
            name: Some(name),
            lowlink: index,
        });
        context.stack.push(index);

        let mut self_ref = false;
        let mut lowlink = index;
        for next in value.references.iter() {
            let next_index = match traverse(next.clone(), context) {
                Some(index) => index,
                // There is nothing to do if the next variable has been
                // fully resolved at this point.
                None => { continue; }
            };
            let next_info = &context.var_info[next_index];
            if next_index > index {
                // The next variable has a larger index than us, so it
                // must be inserted in the recursive call above. We want
                // to get its lowlink.
                lowlink = cmp::min(lowlink, next_info.lowlink);
            } else if next_index == index {
                self_ref = true;
            } else if next_info.name.is_some() {
                // The next variable has a smaller order index and it is
                // in the stack, so we are at the same component.
                lowlink = cmp::min(lowlink, next_index);
            }
        }

        context.var_info[index].lowlink = lowlink;
        if lowlink != index {
            // This variable is in a loop, but it is not the root of
            // this strong connected component. We simply return for
            // now, and the root would add it into the invalid set.
            // This cannot be added into the invalid set here, because
            // otherwise the shortcut check at the beginning of this
            // function would return the wrong value.
            return Some(index);
        }

        // This is the root of a strong-connected component.
        let mut in_loop = self_ref;
        let name;
        loop {
            let var_index = context.stack.pop()
                .expect("The current variable should still be in stack");
            let var_info = &mut context.var_info[var_index];
            // We should never visit the variable again, so it's safe
            // to take the name away, so that we don't do additional
            // reference count.
            let var_name = var_info.name.take()
                .expect("Variable should not be poped from stack twice");
            if var_index == index {
                name = var_name;
                break;
            }
            // Anything here is in a loop which can traverse to the
            // variable we are handling, so we should add it into
            // the invalid set. We should never visit the variable
            // again so it's safe to just take the name away.
            context.invalid.insert(var_name);
            in_loop = true;
        }
        if in_loop {
            // This variable is in loop. Resolve to invalid.
            context.invalid.insert(name);
            return None;
        }

        // Now we have shown that this variable is not in a loop, and
        // all of its dependencies should have been resolved. We can
        // start substitution now.
        let mut computed_value = ComputedValue::empty();
        let mut input = ParserInput::new(&value.css);
        let mut input = Parser::new(&mut input);
        let mut position = (input.position(), value.first_token_type);
        let result = substitute_block(
            &mut input,
            &mut position,
            &mut computed_value,
            &mut |name, partial_computed_value| {
                if let Some(value) = context.map.get(name) {
                    if !context.invalid.contains(name) {
                        partial_computed_value.push_variable(value);
                        return Ok(value.last_token_type);
                    }
                }
                Err(())
            }
        );
        if let Ok(last_token_type) = result {
            computed_value.push_from(position, &input, last_token_type);
            context.map.insert(name, Arc::new(computed_value));
        } else {
            context.invalid.insert(name);
        }

        // All resolved, so return the signal value.
        None
    }

    // We have to clone the names so that we can mutably borrow the map
    // in the context we create for traversal.
    let names = custom_properties_map.index.clone();
    let mut invalid = PrecomputedHashSet::default();
    for name in names.into_iter() {
        let mut context = Context {
            count: 0,
            index_map: PrecomputedHashMap::default(),
            stack: SmallVec::new(),
            var_info: SmallVec::new(),
            map: custom_properties_map,
            invalid: &mut invalid,
        };
        traverse(name, &mut context);
    }

    custom_properties_map.remove_set(&invalid);
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
