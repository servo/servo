/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Support for [custom properties for cascading variables][custom].
//!
//! [custom]: https://drafts.csswg.org/css-variables/

use Atom;
use cssparser::{self, Delimiter, Parser, ParserInput, SourcePosition, Token, TokenSerializationType};
use parser::ParserContext;
use properties::{CSSWideKeyword, DeclaredValue};
use properties_and_values;
use selectors::parser::SelectorParseError;
use std::ascii::AsciiExt;
use std::borrow::{Borrow, Cow};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;
use style_traits::{HasViewportPercentage, ToCss, StyleParseError, ParseError};
use stylesheets::UrlExtraData;

/// A custom property name is just an `Atom`.
///
/// Note that this does not include the `--` prefix
pub type Name = Atom;

/// Parse a custom property name.
///
/// https://drafts.csswg.org/css-variables/#typedef-custom-property-name
pub fn parse_name(s: &str) -> Result<&str, ()> {
    let mut input = ParserInput::new(s);
    let mut input = Parser::new(&mut input);

    match input.expect_ident() {
        Ok(_) if s.starts_with("--") => {
            match input.expect_exhausted() {
                Ok(()) => Ok(&s[2..]),
                Err(_) => Err(()),
            }
        },
        _ => Err(())
    }
}

/// Extra data that we need to pass along with a custom property's specified
/// value in order to compute it, if it ends up being registered as being able
/// to contain URLs through Properties & Values.
/// When the specified value comes from a declaration, we keep track of the
/// associated UrlExtraData. However, specified values can also come from
/// animations: in that case we are able to carry along a copy of the computed
/// value so that we can skip computation altogether (and hopefully avoid bugs
/// with resolving URLs wrong).
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ExtraData {
    /// The specified value comes from a declaration (whence we get
    /// the UrlExtraData).
    Specified(UrlExtraData),

    /// The specified value comes from an animation or an inherited typed custom
    /// property (whence we get the properties_and_values::ComputedValue).
    Precomputed(properties_and_values::ComputedValue),
}

impl<'a> Into<BorrowedExtraData<'a>> for &'a ExtraData {
    fn into(self) -> BorrowedExtraData<'a> {
        match *self {
            ExtraData::Specified(ref x) => BorrowedExtraData::Specified(x),
            ExtraData::Precomputed(ref x) => BorrowedExtraData::Precomputed(x),
        }
    }
}

/// A borrowed version of an ExtraData. Used for BorrowedSpecifiedValue. Has an
/// extra variant, InheritedUntyped, for when the specified value is really
/// borrowed from an inherited value and the property is unregistered. In that
/// case the token stream value is already the computed value.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum BorrowedExtraData<'a> {
    /// The specified value comes from a declaration (whence we get the
    /// UrlExtraData).
    Specified(&'a UrlExtraData),

    /// The specified value comes from an animation or an inherited typed custom
    /// property (whence we get the properties_and_values::ComputedValue).
    Precomputed(&'a properties_and_values::ComputedValue),

    /// The specified value comes from an inherited value for an untyped custom
    /// property, and we should just use the token stream value as the computed
    /// value.
    InheritedUntyped,
}

/// A token stream, represented as a string and boundary tokens.
/// Custom properties' specified values are token streams.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct TokenStream {
    /// The specified text.
    pub css: String,

    /// The first token in the serialization.
    /// Used when resolving variable references, because we would like to
    /// substitute token streams rather than variables; in particular, if
    /// `foo: 5` and we write `width: var(--foo) em`, width's should not be
    /// declared to be `5em`, a dimension token with value , but rather the
    /// integer token `5` followed by the identifier `em`, which is invalid.
    /// We implement this by adding /**/ when necessary (see
    /// ComputedValue::push).
    pub first_token_type: TokenSerializationType,

    /// The last token in the serialization.
    pub last_token_type: TokenSerializationType,
}

impl Default for TokenStream {
    fn default() -> Self {
        TokenStream {
            css: "".to_owned(),
            first_token_type: TokenSerializationType::nothing(),
            last_token_type: TokenSerializationType::nothing(),
        }
    }
}

/// A specified value for a custom property is just a set of tokens.
///
/// We preserve the original CSS for serialization, and also the variable
/// references to other custom property names.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SpecifiedValue {
    /// The specified token stream.
    pub token_stream: TokenStream,

    /// Custom property names in var() functions.
    /// This being None should be treated exactly the same as it being an empty
    /// HashSet; it exists so we don't have to create a new HashSet every time
    /// we are applying an interpolated custom property.
    references: Option<HashSet<Name>>,

    /// Extra data needed to compute the specified value. See the comment on
    /// ExtraData.
    pub extra: ExtraData,
}

impl Deref for SpecifiedValue {
    type Target = TokenStream;

    fn deref(&self) -> &TokenStream {
        &self.token_stream
    }
}

impl HasViewportPercentage for SpecifiedValue {
    fn has_viewport_percentage(&self) -> bool {
        panic!("has_viewport_percentage called before resolving!");
    }
}

impl<'a> From<&'a properties_and_values::ComputedValue> for SpecifiedValue {
    fn from(other: &'a properties_and_values::ComputedValue) -> Self {
        SpecifiedValue {
            token_stream: other.into(),
            references: None,
            extra: ExtraData::Precomputed(other.clone()),
        }
    }
}

/// This struct is a cheap borrowed version of a `SpecifiedValue`.
pub struct BorrowedSpecifiedValue<'a> {
    token_stream: &'a TokenStream,
    references: Option<&'a HashSet<Name>>,
    /// Extra data needed to compute the specified value. See the comment on
    /// ExtraData.
    pub extra: BorrowedExtraData<'a>,
}

impl<'a> Deref for BorrowedSpecifiedValue<'a> {
    type Target = TokenStream;

    fn deref(&self) -> &TokenStream {
        &self.token_stream
    }
}

/// A computed value is just a set of tokens as well, until we resolve variables
/// properly.
pub type ComputedValue = TokenStream;

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

impl<'a> From<&'a properties_and_values::ComputedValue> for TokenStream {
    fn from(other: &'a properties_and_values::ComputedValue) -> Self {
        let mut css = String::new();
        other.to_css::<String>(&mut css).unwrap();
        let (first, last) = {
            let mut missing_closing_characters = String::new();
            let mut input = ParserInput::new(&css);
            let mut input = Parser::new(&mut input);
            // XXX agh! why do we need to parse again just to get
            // these guys.
            parse_declaration_value_block(
                &mut input,
                &mut None,
                &mut missing_closing_characters
            ).unwrap()
        };
        TokenStream {
            css: css,
            first_token_type: first,
            last_token_type: last,
        }
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
/// Outside of this module, this map will normally be accessed through a
/// `properties_and_values::CustomPropertiesMap`, which composes it and stores
/// computed values for typed custom properties as well.
pub type CustomPropertiesMap = OrderedMap<Name, ComputedValue>;

/// A map that preserves order for the keys, and that is easily indexable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Key index.
    index: Vec<K>,
    /// Key-value map.
    values: HashMap<K, V>,
}

impl<K, V> OrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Creates a new ordered map.
    pub fn new() -> Self {
        OrderedMap {
            index: Vec::new(),
            values: HashMap::new(),
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

    /// Remove an item given its key.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = match self.index.iter().position(|k| k.borrow() == key) {
            Some(p) => p,
            None => return None,
        };
        self.index.remove(index);
        self.values.remove(key)
    }
}

/// An iterator for OrderedMap.
///
/// The iteration order is determined by the order that the values are
/// added to the key-value map.
pub struct OrderedMapIterator<'a, K, V>
where
    K: 'a + Eq + Hash + Clone, V: 'a,
{
    /// The OrderedMap itself.
    inner: &'a OrderedMap<K, V>,
    /// The position of the iterator.
    pos: usize,
}

impl<'a, K, V> Iterator for OrderedMapIterator<'a, K, V>
where
    K: Eq + Hash + Clone,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let ref index = self.inner.index;
        if self.pos >= index.len() {
            return None;
        }

        let ref key = index[index.len() - self.pos - 1];
        self.pos += 1;
        let value = self.inner.values.get(key).unwrap();
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
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<Box<Self>, ParseError<'i>> {
        let mut references = Some(HashSet::new());
        let (first, css, last) = parse_self_contained_declaration_value(input, &mut references)?;
        Ok(Box::new(SpecifiedValue {
            token_stream: TokenStream {
                css: css.into_owned(),
                first_token_type: first,
                last_token_type: last,
            },
            references: references,
            extra: ExtraData::Specified(context.url_data.clone()),
        }))
    }

    /// Returns whether or not this specified value contains any variable
    /// references.
    pub fn has_references(&self) -> bool {
        !self.references.as_ref().map(|x| x.is_empty()).unwrap_or(true)
    }
}

/// Parse the value of a non-custom property that contains `var()` references.
pub fn parse_non_custom_with_var<'i, 't>
                                (input: &mut Parser<'i, 't>)
                                -> Result<(TokenSerializationType, Cow<'i, str>), ParseError<'i>> {
    let (first_token_type, css, _) = parse_self_contained_declaration_value(input, &mut None)?;
    Ok((first_token_type, css))
}

fn parse_self_contained_declaration_value<'i, 't>
                                         (input: &mut Parser<'i, 't>,
                                          references: &mut Option<HashSet<Name>>)
                                          -> Result<(
                                              TokenSerializationType,
                                              Cow<'i, str>,
                                              TokenSerializationType
                                          ), ParseError<'i>> {
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
fn parse_declaration_value<'i, 't>
                          (input: &mut Parser<'i, 't>,
                           references: &mut Option<HashSet<Name>>,
                           missing_closing_characters: &mut String)
                          -> Result<(TokenSerializationType, TokenSerializationType), ParseError<'i>> {
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
pub fn parse_declaration_value_block<'i, 't>
                                (input: &mut Parser<'i, 't>,
                                 references: &mut Option<HashSet<Name>>,
                                 missing_closing_characters: &mut String)
                                 -> Result<(TokenSerializationType, TokenSerializationType),
                                           ParseError<'i>> {
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
                    parse_declaration_value_block(input, references, missing_closing_characters)
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
                        parse_var_function(input, references)
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
fn parse_var_function<'i, 't>(input: &mut Parser<'i, 't>,
                              references: &mut Option<HashSet<Name>>)
                              -> Result<(), ParseError<'i>> {
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
    if let Some(ref mut refs) = *references {
        refs.insert(Atom::from(name));
    }
    Ok(())
}

/// Add one custom property declaration to a map, unless another with the same
/// name was already there.
///
/// `inherited_computed(name)` should return the computed value for inherited
/// typed custom properties and None otherwise.
///
/// `handle_keyword(name, keyword)` should return `true` in the case where, if
/// the property with name `name` is declared with CSS-wide keyword `keyword`,
/// the property should inherit.
pub fn cascade<'a, F, G>(
    custom_properties: &mut Option<OrderedMap<&'a Name, BorrowedSpecifiedValue<'a>>>,
    inherited: Option<&'a CustomPropertiesMap>,
    inherited_computed: &F,
    handle_keyword: G,
    seen: &mut HashSet<&'a Name>,
    name: &'a Name,
    specified_value: DeclaredValue<'a, Box<SpecifiedValue>>
)
where F: Fn(&'a Name) -> Option<&'a properties_and_values::ComputedValue>,
      G: Fn(&'a Name, CSSWideKeyword) -> bool,
{
    let was_already_present = !seen.insert(name);
    if was_already_present {
        return;
    }

    #[inline]
    fn inherit<'a, F>(
        map: &mut OrderedMap<&'a Name, BorrowedSpecifiedValue<'a>>,
        get_computed: F,
        name: &'a Name,
        value: &'a ComputedValue,
    )
    where F: Fn(&'a Name) -> Option<&'a properties_and_values::ComputedValue>,
    {
        let extra =
            get_computed(name)
            .map(BorrowedExtraData::Precomputed)
            .unwrap_or(BorrowedExtraData::InheritedUntyped);
        let borrowed = BorrowedSpecifiedValue {
            token_stream: value,
            references: None,
            extra: extra,
        };
        map.insert(name, borrowed);
    }

    if let None = *custom_properties {
        let mut map = OrderedMap::new();
        if let Some(ref inherited) = inherited {
            for name in &inherited.index {
                let inherited_value = inherited.get(name).unwrap();
                if handle_keyword(name, CSSWideKeyword::Unset) {
                    // We should inherit.
                    inherit(&mut map, inherited_computed, name, inherited_value);
                }
            }
        }
        *custom_properties = Some(map);
    }

    let mut map = custom_properties.as_mut().unwrap();

    match specified_value {
        DeclaredValue::Value(ref specified_value) => {
            map.insert(name, BorrowedSpecifiedValue {
                token_stream: &specified_value.token_stream,
                references: specified_value.references.as_ref(),
                extra: (&specified_value.extra).into(),
            });
        },
        DeclaredValue::WithVariables(_) => unreachable!(),
        DeclaredValue::CSSWideKeyword(keyword) => {
            if handle_keyword(name, keyword) {
                // We should inherit.
                // The inherited value is what we already have, if we are an
                // inherited custom property (whence we initialize
                // *custom_properties above). But we might not be!
                if !map.get(&name).is_some() {
                    let inherited_value =
                        inherited
                        .as_ref()
                        .and_then(|inherited| inherited.get(name));
                    if let Some(inherited_value) = inherited_value {
                        inherit(&mut map, inherited_computed, name, inherited_value);
                    }
                }
            } else {
                // We should use the initial value. Remove the value we
                // inherited, if any.
                map.remove(&name);
            }
        }
    }
}


/// Replace `var()` functions for all custom properties.
pub fn substitute_all<C, H>(
    specified_values_map: &OrderedMap<&Name, BorrowedSpecifiedValue>,
    to_substitute: &Option<HashSet<Name>>,
    custom_properties_map: &mut CustomPropertiesMap,
    mut compute: &mut C,
    mut handle_invalid: &mut H,
)
where C: FnMut(&Name, ComputedValue) -> Result<ComputedValue, ()>,
      H: FnMut(&Name) -> Result<ComputedValue, ()>,
{
    let mut invalid = HashSet::new();
    for (&name, value) in specified_values_map.iter() {
        if !to_substitute.as_ref().map(|x| x.contains(name)).unwrap_or(true) {
            continue
        }
        // If this value is invalid at computed-time it won’t be inserted in
        // computed_values_map. Nothing else to do.
        let _ = substitute_one(
            name, value, specified_values_map, None, custom_properties_map,
            &mut invalid, &mut compute, &mut handle_invalid);
    }
}

/// Identify and remove cyclical variable declarations, identify if font-size is
/// involved in a cycle, and identify those variables that must be computed
/// before font-size.
///
/// specified[p] is removed if p is involved in a cycle.
///
/// Finally, we return whether font-size is involved in a cycle as well as a set
/// of those variables to compute before computing early properties, including
/// those variables (possibly transitively) referenced by the declaration of
/// font-size. We count references in fallbacks (as does cycle detection).
///
/// For the purposes of cycle detection, we imagine font-size as another
/// variable, and create an edge in the dependency graph from a variable to
/// font-size if the variable is in possibly_font_size_dependent and if we parse
/// a dimension token inside. The caller should put those variables in
/// possibly_font_size_dependent that have as a possible syntax at least one of
/// <length>, <length-percentage>, or <transform-function> (syntaxes whose
/// computation might involve computing absolute lengths from relative lengths).
pub fn compute_ordering(
    specified: &mut OrderedMap<&Name, BorrowedSpecifiedValue>,
    referenced_by_font_size: &HashSet<Name>,
    referenced_by_others: &HashSet<Name>,
    possibly_font_size_dependent: &HashSet<Name>,
) -> (bool, HashSet<Name>, HashSet<Name>) {
    fn walk<'a>(
        specified: &HashMap<&'a Name, BorrowedSpecifiedValue<'a>>,
        // None if we aren't descending down font-size.
        // Some((set, might_require_computation)) if we are descending down
        // font-size; might_require_computation is set to true if we are looking
        // at a dependency of a custom property that might require computation.
        // Determine to be cyclical if the set contains us and we have an em in
        // our declaration, or if one of our descendants in the tree (for which
        // might_require_computation is set to true) contains an em.
        font_size_data: Option<(&HashSet<Name>, bool)>,
        name: &'a Name,
        stack: &mut Vec<&'a Name>,
        visited: &mut HashSet<&'a Name>,
        cyclical: &mut HashSet<Name>,
        font_size_cyclical: &mut bool,
    ) {
        let already_visited_before = !visited.insert(name);
        if already_visited_before {
            return
        }

        let (declaration, references) = {
            if let Some(declaration) = specified.get(name) {
                if let Some(references) = declaration.references {
                    (declaration, references)
                } else {
                    // No variables referenced.
                    return
                }
            } else {
                // Invalid variable reference--will handle later.
                return
            }
        };

        stack.push(name);

        // Recurse into variable references.
        for next in references {
            if let Some(position) = stack.iter().position(|&x| x == next) {
                // Found a cycle!
                for in_cycle in &stack[position..] {
                    cyclical.insert((*in_cycle).clone());
                }
            } else {
                let font_size_data = match font_size_data {
                    Some((possibly_font_size_dependent, false))
                    if possibly_font_size_dependent.contains(name) => {
                        Some((possibly_font_size_dependent, true))
                    },
                    _ => font_size_data,
                };
                walk(specified, font_size_data, next, stack, visited, cyclical,
                     font_size_cyclical);
            }
        }

        // If this is a registered custom property whose computation
        // calculation requires font-size, recurse into the
        // references for font-size.
        let might_cause_font_size_cycle = {
            if let Some((possibly_font_size_dependent, might_require_computation)) = font_size_data {
                might_require_computation || possibly_font_size_dependent.contains(name)
            } else {
                false
            }
        };
        if might_cause_font_size_cycle {
            fn detect_ems<'i, 'tt>(input: &mut Parser<'i, 'tt>)
                                   -> Result<bool, cssparser::ParseError<'i, ()>> {
                while !input.is_exhausted() {
                    let token = input.next()?.clone();
                    // We have to descend into functions.
                    match token {
                        Token::Function(_) => {
                            if input.parse_nested_block(detect_ems).unwrap() {
                                return Ok(true)
                            }
                        },
                        Token::Dimension { ref unit, .. } => {
                            if unit.eq_ignore_ascii_case("em") ||
                               unit.eq_ignore_ascii_case("rem") {
                                return Ok(true)
                            }
                        }
                        _ => ()
                    }
                }
                Ok(false)
            }

            let mut input = ParserInput::new(&declaration.css);
            let mut input = Parser::new(&mut input);

            if detect_ems(&mut input).unwrap() {
                // Found a cycle involving font-size!
                *font_size_cyclical = true;
                for in_cycle in stack.iter() {
                    cyclical.insert((*in_cycle).clone());
                }
            }
        }
        stack.pop();
    }

    // Identify cycles.

    let mut cyclical = HashSet::new();
    let mut font_size_cyclical = false;
    let mut visited = HashSet::new();
    // We recursively follow variable references in declarations, and push
    // the current variable to the stack, so that the declaration for
    // stack[i] has a variable reference to stack[i+1]. So if we see a
    // reference to a variable which already appears on the stack, we know
    // that we have a cycle.
    let mut stack = Vec::new();

    // Check variables referenced by font-size first, to identify if there is a
    // cycle involving font-size.
    // Then, check variables referenced by other properties which we need to
    // check (i.e. "early" properties).
    // Then we can just take visited to get those variables which are
    // transitively referenced by it.
    for name in referenced_by_font_size {
        walk(&specified.values, Some((possibly_font_size_dependent, false)),
             &name, &mut stack, &mut visited, &mut cyclical,
             &mut font_size_cyclical);
    }
    for name in referenced_by_others {
        walk(&specified.values, None, &name, &mut stack, &mut visited,
             &mut cyclical, &mut font_size_cyclical);
    }
    let mut dependencies = visited.clone();
    let dependencies: HashSet<Name> = dependencies.drain().map(|x| x.clone()).collect();

    for name in specified.values.keys() {
        walk(&specified.values, None, name, &mut stack, &mut visited,
             &mut cyclical, &mut font_size_cyclical);
    }

    // If we wanted dependencies to be completely accurate, we would
    // remove z if x depends on y depends on z yet y is involved in a cycle.
    // But we don't really need that; we just want to include all those
    // properties that are depended on by early properties. Properties that
    // are in dependencies can't depend on those early properties (the only
    // such dependency we can have here is one on font-size) because in that
    // case we would have detected the cycle; by resetting to the initial
    // value (which is computationally independent) we no longer have the
    // cycle.
    //
    // Really, what dependencies describe is the custom properties that we can
    // compute early and those that we must.

    (font_size_cyclical, dependencies, cyclical)
}

/// Replace `var()` functions for one custom property.
/// Also recursively record results for other custom properties referenced by `var()` functions.
/// Return `Err(())` for invalid at computed time.
/// or `Ok(last_token_type that was pushed to partial_computed_value)` otherwise.
fn substitute_one<C, H>(
    name: &Name,
    specified_value: &BorrowedSpecifiedValue,
    specified_values_map: &OrderedMap<&Name, BorrowedSpecifiedValue>,
    mut partial_computed_value: Option<&mut ComputedValue>,
    custom_properties_map: &mut CustomPropertiesMap,
    invalid: &mut HashSet<Name>,
    compute: &mut C,
    handle_invalid: &mut H,
) -> Result<TokenSerializationType, ()>
where C: FnMut(&Name, ComputedValue) -> Result<ComputedValue, ()>,
      H: FnMut(&Name) -> Result<ComputedValue, ()>,
{
    if let Some(computed_value) = custom_properties_map.get(&name) {
        if let Some(partial_computed_value) = partial_computed_value {
            partial_computed_value.push_variable(computed_value)
        }
        return Ok(computed_value.last_token_type)
    }

    if invalid.contains(name) {
        return Err(());
    }
    let computed_value = if !specified_value.references.map(|set| set.is_empty()).unwrap_or(true) {
        let mut partial_computed_value = ComputedValue::empty();
        let mut input = ParserInput::new(&specified_value.css);
        let mut input = Parser::new(&mut input);
        let mut position = (input.position(), specified_value.first_token_type);
        let result = substitute_block(
            &mut input, &mut position, &mut partial_computed_value,
            &mut |name, partial_computed_value| {
                if let Some(other_specified_value) = specified_values_map.get(&name) {
                    substitute_one(name, other_specified_value, specified_values_map,
                                   Some(partial_computed_value), custom_properties_map, invalid,
                                   compute, handle_invalid)
                } else {
                    Err(())
                }
            }
        );
        if let Ok(last_token_type) = result {
            partial_computed_value.push_from(position, &input, last_token_type);
            Ok(partial_computed_value)
        } else {
            Err(())
        }
    } else {
        // The specified value contains no var() reference
        Ok(ComputedValue {
            css: specified_value.css.to_owned(),
            first_token_type: specified_value.first_token_type,
            last_token_type: specified_value.last_token_type,
        })
    };

    computed_value
    .and_then(|x| compute(name, x))
    .or_else(|()| {
        let result = handle_invalid(name);
        if let Err(()) = result {
            invalid.insert(name.clone());
        }
        result
    })
    .and_then(|x| {
        if let Some(ref mut partial_computed_value) = partial_computed_value {
            partial_computed_value.push_variable(&x)
        }
        let last_token_type = x.last_token_type;
        custom_properties_map.insert(name.clone(), x);
        Ok(last_token_type)
    })
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
fn substitute_block<'i, 't, F>(input: &mut Parser<'i, 't>,
                               position: &mut (SourcePosition, TokenSerializationType),
                               partial_computed_value: &mut ComputedValue,
                               substitute_one: &mut F)
                               -> Result<TokenSerializationType, ParseError<'i>>
                       where F: FnMut(&Name, &mut ComputedValue) -> Result<TokenSerializationType, ()> {
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
                        // Try to substitute any variable references in the
                        // fallback.
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

/// Replace `var()` functions for a non-custom property declaration.
/// Return `Err(..)` if the declaration should be invalid at computed-value time
/// (that is, if resolution fails).
pub fn substitute<'i>(input: &'i str, first_token_type: TokenSerializationType,
                      computed_values_map: Option<&CustomPropertiesMap>)
                      -> Result<String, ParseError<'i>> {
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
