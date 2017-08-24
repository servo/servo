/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Support for the [Properties & Values API][spec].
//!
//! [spec]: https://drafts.css-houdini.org/css-properties-values-api-1/

use Atom;
use cssparser::{BasicParseError, ParseError, ParserInput, Parser, Token};
use custom_properties::{self, Name};
use parser::{Parse, ParserContext};
use properties::CSSWideKeyword;
use properties::longhands::transform;
use selectors::parser::SelectorParseError;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::vec::Vec;
use style_traits::{ParseError as StyleTraitsParseError, StyleParseError};
use style_traits::values::{OneOrMoreSeparated, Space, ToCss};
use values;
use values::computed::{self, ComputedValueAsSpecified, ToComputedValue};
use values::specified;

/// A registration for a custom property.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Registration {
    /// The custom property name, sans leading '--'.
    pub name: Name,

    /// The syntax of the custom property.
    pub syntax: Syntax,

    /// Whether the custom property is inherited down the DOM tree.
    pub inherits: bool,

    /// The initial value of the custom property.
    ///
    /// Ideally we'd merge this with `syntax` so that illegal states would be
    /// unrepresentable. But while we could do that by turning the fields of the
    /// SpecifiedVariable variants into Option<T>'s, we would need a more
    /// expressive type system to do this with disjunctions.
    ///
    /// Ideally this would also be a ComputedValue. But to reuse the
    /// to_computed_value code we need a style::values::computed::Context, which
    /// is a real pain to construct in a nice way for both Stylo & Servo.
    /// Instead we just store the specified value and compute this later; the
    /// is_computationally_independent check should mean this doesn't matter.
    pub initial_value: Option<SpecifiedValue>,
}

/// A versioned set of registered custom properties, stored on the document.
/// The [[registeredPropertySet]] of the spec. We keep track of the version to
/// know if we need to recompute which declarations are valid.
#[derive(Default)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct RegisteredPropertySet {
    /// The set of registered custom properties. Names are sans leading '--'.
    registrations: HashMap<Name, Registration>,

    /// The current version. Must be incremented whenever `registrations` is
    /// modified.
    generation: u32,
}

impl RegisteredPropertySet {
    /// Attempt to register a custom property.
    ///
    /// If a custom property has already been registered with that name, return
    /// Err(()), otherwise return Ok(()) and increment the generation.
    pub fn register_property(&mut self, registration: Registration) -> Result<(), ()> {
        match self.registrations.insert(registration.name.clone(), registration) {
            Some(_) => Err(()),
            None => {
                self.generation += 1;
                Ok(())
            }
        }
    }

    /// Attempt to unregister a custom property.
    ///
    /// If no custom property has been registered with that name, return
    /// Err(()), otherwise return Ok(()) and increment the generation.
    pub fn unregister_property(&mut self, name: &Name) -> Result<(), ()> {
        match self.registrations.remove(name) {
            Some(_) => {
                self.generation += 1;
                Ok(())
            },
            None => Err(())
        }
    }

    /// Return the current generation.
    ///
    /// The generation is incremented every time the set of custom property
    /// registrations changes. It's used by the Stylist to keep track of when it
    /// has to restyle.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Attempt to get the registration for the custom property with the given
    /// name.
    pub fn get(&self, name: &Name) -> Option<&Registration> {
        self.registrations.get(name)
    }

    /// Return the set of all uninherited custom properties.
    ///
    /// Used by style::properties::compute_early_custom_properties to insert
    /// initial values when needed.
    pub fn uninherited_properties(&self) -> HashSet<&Name> {
        self.registrations
            .iter()
            .filter(|&(_, registration)| !registration.inherits)
            .map(|(name, _)| name)
            .collect()
    }

    /// Returns (computed) initial values for all custom properties except those
    /// specified in `except`. If `except` is None, it's treated as the empty
    /// set, and we return the initial values for all custom properties.
    ///
    /// Initial values are computationally idempotent, so they should not need
    /// actual computation. Unfortunately the most convenient way to convert a
    /// specified value to a computed value is to use the to_computed_value
    /// method, which requires a context.
    pub fn initial_values_except(
        &self,
        context: &computed::Context,
        except: Option<&HashSet<&Name>>
    ) -> CustomPropertiesMap {
        let mut map = CustomPropertiesMap::new();
        for (name, registration) in self.registrations.iter() {
            if except.map(|x| x.contains(name)).unwrap_or(false) {
                continue
            }
            if let Some(ref initial) = registration.initial_value {
                let computed =
                    (&initial.clone().to_computed_value(context)
                     .expect("The initial value should never fail to compute."))
                    .into();
                map.insert((*name).clone(), computed);
            }
        }
        map
    }
}

/// The result of a call to register_property or unregister_property,
/// corresponding to the errors that can be thrown by CSS.(un)registerProperty.
///
/// Should be kept in sync with mozilla::PropertyRegistrationResult on the Gecko
/// side.
pub enum PropertyRegistrationResult {
    /// Indicates that the call was successful. The caller should return without
    /// error.
    Ok = 0,
    /// Indicates that the call failed, and that the caller should throw a
    /// SyntaxError.
    SyntaxError,
    /// Indicates that the call failed, and that the caller should throw an
    /// InvalidModificationError.
    InvalidModificationError,
    /// Indicates that the call failed, and that the caller should throw a
    /// NotFoundError.
    NotFoundError,
}

/// Attempt to register a custom property.
///
/// This is used by the CSS.registerProperty implementations for Servo and Gecko
/// in order to share logic. The caller should handle the returned
/// PropertyRegistraitonResult by throwing the appropriate DOM error.
pub fn register_property(
    registered_property_set: &mut RegisteredPropertySet,
    parser_context: &ParserContext,
    name: &str,
    syntax: &str,
    inherits: bool,
    initial_value: Option<&str>
) -> PropertyRegistrationResult {
    let name = match custom_properties::parse_name(name) {
        Ok(name) => name,
        Err(()) => return PropertyRegistrationResult::SyntaxError,
    };

    let syntax = match Syntax::from_string(syntax) {
        Ok(syntax) => syntax,
        Err(()) => return PropertyRegistrationResult::SyntaxError,
    };

    let initial_value = match initial_value {
        Some(ref specified) => {
            let mut input = ParserInput::new(specified);
            let mut input = Parser::new(&mut input);
            match syntax.parse(parser_context, &mut input) {
                Ok(parsed) => {
                    if parsed.is_computationally_independent() {
                        Some(parsed)
                    } else {
                        return PropertyRegistrationResult::SyntaxError
                    }
                },
                _ => return PropertyRegistrationResult::SyntaxError,
            }
        },
        None if matches!(syntax, Syntax::Anything) => None,
        // initialValue is required if the syntax is not '*'.
        _ => return PropertyRegistrationResult::SyntaxError,
    };

    let result =
        registered_property_set
        .register_property(Registration {
            name: name.into(),
            syntax: syntax,
            inherits: inherits,
            initial_value: initial_value,
        });

    match result {
        Ok(_) => PropertyRegistrationResult::Ok,
        Err(_) => PropertyRegistrationResult::InvalidModificationError,
    }
}

/// Attempt to unregister a custom property.
///
/// This is used by the CSS.registerProperty implementations for Servo and Gecko
/// in order to share logic. The caller should handle the returned
/// PropertyRegistraitonResult by throwing the appropriate DOM error.
pub fn unregister_property(
    registered_property_set: &mut RegisteredPropertySet,
    name: &str
) -> PropertyRegistrationResult {
    let name = match custom_properties::parse_name(name) {
        Ok(name) => name,
        Err(()) => return PropertyRegistrationResult::SyntaxError,
    };

    let result =
        registered_property_set
        .unregister_property(&name.into());

    match result {
        Ok(_) => PropertyRegistrationResult::Ok,
        Err(_) => PropertyRegistrationResult::NotFoundError,
    }
}



/// A CSS <custom-ident>.
///
/// We make a newtype for Atom and implement ToCss ourselves because the
/// ToCss implementation for atom in `style_traits::values` uses `cssparsers`'s
/// `serialize_string` function, which writes a double-quoted CSS string. We're
/// only storing <custom-idents>, which should be serialized as specified.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Ident(pub Atom);

impl ComputedValueAsSpecified for Ident {}

impl ToCss for Ident {
    #[cfg(feature = "servo")]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(&*self.0)
    }

    #[cfg(feature = "gecko")]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str(&self.0.to_string())
    }
}

impl Deref for Ident {
    type Target = Atom;

    fn deref(&self) -> &Atom {
        &self.0
    }
}

/// A computed <url> value.
///
/// FIXME: While as_str() resolves URLs on the Servo side, it currently does not
/// resolve them on the Gecko side (see the comment on Gecko's
/// specified::url::SpecifiedUrl implementation). That means we don't actually
/// resolve URLs when running in Stylo yet.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ComputedUrl(pub String);

impl ComputedUrl {
    #[cfg(feature = "servo")]
    fn from_specified(url: &specified::url::SpecifiedUrl) -> Result<ComputedUrl, ()> {
        url.url().map(|x| ComputedUrl(x.as_str().to_owned())).ok_or(())
    }

    #[cfg(feature = "gecko")]
    fn from_specified(url: &specified::url::SpecifiedUrl) -> Result<ComputedUrl, ()> {
        // Doesn't work.
        // url.extra_data.join(url.as_str()).map(|x| ComputedUrl(x.as_str().to_owned())).ok_or(())
        Ok(ComputedUrl(url.as_str().to_owned()))
        // Note: the Gecko binding doesn't currently output resolved URLs.
    }
}

impl ToCss for ComputedUrl {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("url(")?;
        self.0.to_css(dest)?;
        dest.write_str(")")
    }
}

/// A basic custom property syntax string for a custom property that, used to
/// build up disjunctions and list terms.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Type {
    /// Syntax to allow any valid <length> value.
    Length,
    /// Syntax to allow any valid <number> value.
    Number,
    /// Syntax to allow any valid <percentage> value.
    Percentage,
    /// Syntax to allow any valid <length> or <percentage> value, or any valid
    /// <calc()> expression combining <length> and <percentage> components.
    LengthPercentage,
    /// Syntax to allow any valid <color> value.
    Color,
    /// Syntax to allow any valid <image> value.
    Image,
    /// Syntax to allow any valid <url> value.
    Url,
    /// Syntax to allow any valid <integer> value.
    Integer,
    /// Syntax to allow any valid <angle> value.
    Angle,
    /// Syntax to allow any valid <time> value.
    Time,
    // FIXME: We should support <resolution> values as well.
    /// Syntax to allow any valid <transform-list> value.
    TransformList,
    /// Syntax to allow any valid <custom-ident> value.
    CustomIdent,
    /// Syntax to allow a specific identifier (matching the <custom-ident>
    /// production, compared codepoint-wise).
    SpecificIdent(Ident),
}


impl Type {
    /// Attempt to parse `input` as this type.
    pub fn parse<'i, 't>(
        &self,
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<SpecifiedValueItem, StyleTraitsParseError<'i>>
    {
        #[cfg(feature = "servo")]
        fn idents_eq(a: &Atom, b: &str) -> bool {
            a == b
        }

        #[cfg(feature = "gecko")]
        fn idents_eq(a: &Atom, b: &str) -> bool {
            *a == b.into()
        }

        macro_rules! parse {
            ($_self:expr, $context:expr, $input:expr,

             $($typ:ident => $fn:path),*) => {
                match $_self {
                    $(
                        Type::$typ => {
                            $fn(context, input).map(|x| {
                                SpecifiedValueItem::$typ(x)
                            })
                        }
                    ), *

                    // We need to actually compare SpecificIdents,
                    // unfortunately.
                    Type::SpecificIdent(ref x) => {
                        $input.expect_ident_cloned().and_then(|y| {
                            if idents_eq(&**x, &*y) {
                                Ok(SpecifiedValueItem::SpecificIdent(x.clone()))
                            } else {
                                Err(BasicParseError::UnexpectedToken(Token::Ident(y)).into())
                            }
                        }).map_err(|e| e.into())
                    },
                }
            };
        }

        fn parse_custom_ident<'i, 't>(
            _context: &ParserContext,
            input: &mut Parser<'i, 't>,
        ) -> Result<Ident, StyleTraitsParseError<'i>> {
            input.expect_ident_cloned()
                 .map(|x| Ident((&*x).into()))
                 .map_err(|e| e.into())
        }

        parse! {
            *self, context, input,

            Length => specified::Length::parse,
            Number => specified::Number::parse,
            Percentage => specified::Percentage::parse,
            LengthPercentage => specified::LengthOrPercentage::parse,
            Color => specified::Color::parse,
            Image => specified::Image::parse,
            Url => specified::url::SpecifiedUrl::parse,
            Integer => specified::Integer::parse,
            Angle => specified::Angle::parse,
            Time => specified::Time::parse,
            TransformList => transform::parse,
            CustomIdent => parse_custom_ident
        }
    }
}


/// A custom property syntax string that is either some basic syntax string
/// (e.g. some <url> value) or some list term. A list term syntax string allows
/// a space-separated list of one or more repetitions of the type specified by
/// the string. Used to build up disjunctions.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Term {
    /// The type of the term (e.g. <integer>).
    pub typ: Type,
    /// Whether or not the term is a list, i.e., if the syntax string was
    /// <integer>+.
    pub list: bool,
}

/// A custom property syntax string.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Syntax {
    /// Syntax to allow any token stream (written '*').
    Anything,
    /// Syntax to allow some disjunction of terms (possibly list terms), which
    /// allows any value matching one of the items in the combination, matched
    /// in specified order (written 'a | b | ...').
    Disjunction(Vec<Term>),
}

impl Syntax {
    /// Parse a syntax string given to `CSS.registerProperty`.
    pub fn from_string(input: &str) -> Result<Syntax, ()> {
        // Syntax strings are DOMStrings, but in Servo we assume they are valid
        // UTF-8. See
        // https://doc.servo.org/script/dom/bindings/str/struct.DOMString.html
        // . This justifies iteration by |char|.

        // Can identifiers in syntax strings contain escapes? No.
        //
        // "Currently the answer is no - I've clarified the "literal ident"
        //  part to be specifically a name-start char followed by 0+ name chars.
        //  Would prefer to avoid having to do CSS parsing on the syntax string.
        //  ^_^"
        // https://github.com/w3c/css-houdini-drafts/issues/112
        //
        // A 'specific ident' is any sequence consisting of a name-start code
        // point, followed by zero
        // or more name code points, which matches the <custom-ident>
        // production
        // https://drafts.css-houdini.org/css-properties-values-api-1/#supported-syntax-strings
        //
        // ...
        // This generic data type is denoted by <custom-ident>, and represents
        // any valid CSS identifier that would not be misinterpreted as a
        // pre-defined keyword in that property’s value definition.
        // https://drafts.csswg.org/css-values-4/#identifier-value
        //
        // So! In order to make sure specific identifiers don't contain
        // escapes, we need to check for escapes, which are only introduced by
        // backslashes, which shouldn't show up anyhow.
        // https://drafts.csswg.org/css-syntax-3/#escaping
        let mut contains_backslash = false;
        for c in input.chars() {
            if c == '\\' {
                contains_backslash = true;
                break
            }
        }
        if contains_backslash {
            return Err(())
        }

        // The parsed syntax string, which we'll build up as we scan tokens.
        let mut syntax = None;

        // The syntax string isn't really CSS, but hopefully this maximizes
        // code reuse.
        let mut parser_input = ParserInput::new(input);
        let mut parser = Parser::new(&mut parser_input);

        #[derive(Debug)]
        enum State {
            // *.
            AfterAsterisk,
            // <.
            AfterOpenAngle,
            // +.
            AfterPlus,
            // <type>.
            // ident.
            AfterType { after_whitespace: bool },
            // <type.
            AfterTypeName,
            // .
            // |.
            Start { after_bar: bool },
        }

        let mut state = State::Start { after_bar: false };

        /// Add a `Type` to the disjunction. It might turn out this is a list
        /// term, in which case we'll modify the `Term` later.
        fn push_type(syntax: &mut Option<Syntax>, t: Type) {
            if let Some(Syntax::Disjunction(ref mut ts)) = *syntax {
                ts.push(Term { typ: t, list: false })
            } else { unreachable!() }
        }

        /// Signal that we expect to be parsing some term in a disjunction.
        fn expect_disjunction(syntax: &mut Option<Syntax>) {
            if let Some(Syntax::Disjunction(_)) = *syntax {
                // Good!
            } else {
                assert!(*syntax == None);
                *syntax = Some(Syntax::Disjunction(vec![]))
            }
        }

        /// Handle the next token in the syntax string (including whitespace).
        fn handle_token(syntax: &mut Option<Syntax>, state: State, token: &Token) -> Result<State, ()> {
            debug!("{:?} - {:?}", state, token);
            match (state, token) {
                (_, &Token::Comment(_)) => Err(()),

                (State::Start { .. }, &Token::WhiteSpace(_)) => {
                    // Ignore whitespace.
                    Ok(State::Start { after_bar: false })
                },
                (State::Start { after_bar: false }, &Token::Delim('*')) => {
                    // If we see a '*', that should be it (modulo whitespace).
                    if syntax != &None {
                        Err(())
                    } else {
                        *syntax = Some(Syntax::Anything);
                        Ok(State::AfterAsterisk)
                    }
                },
                (State::Start { .. }, &Token::Delim('<')) => {
                    // A '<' should mean we're in the start of a '<type>'.
                    expect_disjunction(syntax);
                    Ok(State::AfterOpenAngle)
                },
                (State::Start { .. }, &Token::Ident(ref id)) => {
                    // An identifier by itself should mean we're about to see a
                    // specific identifier. Note that for <custom-idents> we
                    // have that they "[must] not be misinterpreted as a
                    // pre-defined keyword in that property’s value
                    // definition". Here that means CSS-wide keywords!
                    expect_disjunction(syntax);
                    match CSSWideKeyword::from_ident(id) {
                        Some(_) => Err(()), None => {
                            push_type(syntax, Type::SpecificIdent(Ident((**id).into())));
                            Ok(State::AfterType { after_whitespace: false })
                        }
                    }
                },
                (State::Start { .. }, _) => Err(()),

                (State::AfterOpenAngle, &Token::Ident(ref id)) => {
                    // We should be in something like '<length>' here.
                    // https://drafts.css-houdini.org/css-properties-values-api/#supported-syntax-strings
                    push_type(syntax, match &**id {
                        "length" => Type::Length,
                        "number" => Type::Number,
                        "percentage" => Type::Percentage,
                        "length-percentage" => Type::LengthPercentage,
                        "color" => Type::Color,
                        "image" => Type::Image,
                        "url" => Type::Url,
                        "integer" => Type::Integer,
                        "angle" => Type::Angle,
                        "time" => Type::Time,
                        //"resolution" => Type::Resolution,
                        "transform-list" => Type::TransformList,
                        "custom-ident" => Type::CustomIdent,
                        _ => return Err(())
                    });
                    Ok(State::AfterTypeName)
                },
                (State::AfterOpenAngle, _) => Err(()),

                (State::AfterTypeName, &Token::Delim('>')) => {
                    // This should be the end of something like '<length>'.
                    Ok(State::AfterType { after_whitespace: false })
                },
                (State::AfterTypeName, _) => Err(()),

                (State::AfterType { .. }, &Token::WhiteSpace(_)) => {
                    // Ignore whitespace.
                    Ok(State::AfterType { after_whitespace: true })
                },
                (State::AfterType { after_whitespace: false }, &Token::Delim('+')) => {
                    // We should be following some type.
                    // We should panic if we're not, because we should only get
                    // here from Start -> AfterOpenAngle -> AfterTypeName (in
                    // the case of something like '<length>') or
                    // Start (in the case of something like 'my-ident'), both
                    // of which should have pushed a type.
                    if let Some(Syntax::Disjunction(ref mut ts)) = *syntax {
                        let term = &mut ts.last_mut().unwrap();
                        if term.typ == Type::TransformList {
                            // <transform-list>+ is specifically disallowed.
                            return Err(())
                        }
                        term.list = true
                    } else { unreachable!() }
                    Ok(State::AfterPlus)
                },
                (State::AfterType { .. }, &Token::Delim('|')) => {
                    // Some other term in the disjunction should follow.
                    Ok(State::Start { after_bar: true })
                },
                (State::AfterType { .. }, _) => Err(()),

                (State::AfterAsterisk, &Token::WhiteSpace(_)) => Ok(State::AfterAsterisk),
                (State::AfterAsterisk, _) => Err(()),
                (State::AfterPlus, &Token::WhiteSpace(_)) => Ok(State::AfterPlus),
                (State::AfterPlus, &Token::Delim('|')) => {
                    // Some other term in the disjunction should follow.
                    Ok(State::Start { after_bar: true })
                }
                (State::AfterPlus, _) => Err(()),
            }
        }

        // Loop over all of the tokens in the syntax string.
        loop {
            match parser.next_including_whitespace_and_comments() {
                Err(BasicParseError::EndOfInput) => {
                    match state {
                        State::Start { after_bar: false } |
                        State::AfterType { .. } |
                        State::AfterAsterisk |
                        State::AfterPlus => break,

                        // We shouldn't reach EOF in the middle of something.
                        State::Start { after_bar: true } |
                        State::AfterOpenAngle |
                        State::AfterTypeName => return Err(())
                    }
                },
                Err(_) => return Err(()),
                Ok(token) => {
                    match handle_token(&mut syntax, state, token) {
                        Ok(s) => state = s,
                        Err(()) => return Err(())
                    }
                }
            }
        }

        syntax.ok_or(())
    }

    /// Parse some value following this syntax.
    ///
    /// It's the responsibility of the caller to appropriately delimit the
    /// input, and to make sure that the expected amount of input was consumed.
    /// This should accordingly be called with a delimited parser.
    /// This is a difference from the `parse` function provided by the `Parse`
    /// trait, along with the fact that this returns a `SpecifiedValue`
    /// rather than `Self`.
    pub fn parse<'i, 't>(
        &self,
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<SpecifiedValue, StyleTraitsParseError<'i>> {
        let start = input.state();

        // Check to make sure the entirety of the input isn't some CSS-wide
        // keyword.
        if let Ok(ident) = input.expect_ident_cloned() {
            if input.is_exhausted() {
                match CSSWideKeyword::from_ident(&ident) {
                    Some(_) => return Err(BasicParseError::UnexpectedToken(Token::Ident(ident)).into()),
                    None => (),
                }
            }
        }

        input.reset(&start);

        match *self {
            Syntax::Anything => {
                custom_properties::SpecifiedValue::parse(context, input)
                    // Don't allow variable references: they should have been
                    // expanded by now.
                    .and_then(|x| {
                        if x.has_references() {
                            Err(BasicParseError::UnexpectedToken(Token::Function("var".into())).into())
                        } else {
                            Ok(x)
                        }
                    })
                    .map(|x| SpecifiedValue::Item(SpecifiedValueItem::TokenStream(*x)))
            },
            Syntax::Disjunction(ref terms) => {
                for term in terms {
                    if term.list {
                        let mut outputs = Vec::new();
                        loop {
                            // TODO(jyc) Extend parse_until_before to take
                            // space as a delimiter?
                            match term.typ.parse(context, input) {
                                Err(_) => break,
                                Ok(x) => outputs.push(x)
                            }
                            // Need at least one.
                            if input.is_exhausted() {
                                return Ok(SpecifiedValue::List(outputs))
                            }
                            if let Err(_) = input.expect_whitespace() {
                                break
                            }
                        }
                    } else {
                        // If we fail to parse, try again!
                        if let Ok(x) = term.typ.parse(context, input) {
                            return Ok(SpecifiedValue::Item(x))
                        }
                    }
                    input.reset(&start)
                }
                Err(ParseError::Custom(SelectorParseError::Custom(StyleParseError::UnspecifiedError)))
            },
        }
    }
}

/// A single specified typed value.
#[derive(Clone, ToCss)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum SpecifiedValueItem {
    /// A single specified <length> value.
    Length(specified::Length),
    /// A single specified <number> value.
    Number(specified::Number),
    /// A single specified <percentage> value.
    Percentage(specified::Percentage),
    /// A single specified <length-percentage> value.
    LengthPercentage(specified::LengthOrPercentage),
    /// A single specified <color> value.
    Color(specified::Color),
    /// A single specified <image> value.
    Image(specified::Image),
    /// A single specified <url> value.
    Url(specified::url::SpecifiedUrl),
    /// A single specified <integer> value.
    Integer(specified::Integer),
    /// A single specified <angle> value.
    Angle(specified::Angle),
    /// A single specified <time> value.
    Time(specified::Time),
    // FIXME: We should support <resolution> as well.
    /// A single specified <transform-list> value (note that this is composed of
    /// multiple <transform-functions>.
    TransformList(transform::SpecifiedValue),
    /// A single specified <custom-ident> value.
    CustomIdent(Ident),
    /// A single specified <custom-ident> value that matches the specific ident
    /// contained herein.
    SpecificIdent(Ident),
    /// A <token-stream> value.
    TokenStream(custom_properties::SpecifiedValue),
}

impl OneOrMoreSeparated for SpecifiedValueItem {
    type S = Space;
}

impl SpecifiedValueItem {
    /// Returns whether or not this specified value is computationally
    /// independent, that is, 'if it can be converted into a computed value
    /// using only the value of the property on the element, and "global"
    /// information that cannot be changed by CSS.'
    /// https://drafts.css-houdini.org/css-properties-values-api-1/#computationally-independent
    pub fn is_computationally_independent(&self) -> bool {
        use self::specified::{CalcLengthOrPercentage, Length, LengthOrPercentage, LengthOrPercentageOrNumber};
        use self::specified::NoCalcLength;
        use self::transform::SpecifiedOperation;
        use self::values::Either::*;
        use self::values::generics::transform::Matrix;

        fn check_no_calc_length(length: &NoCalcLength) -> bool {
            match *length {
                NoCalcLength::Absolute(_) => true,
                // FIXME: 0em should be computationally independent.
                NoCalcLength::FontRelative(_) => false,
                NoCalcLength::ViewportPercentage(_) => false,
                NoCalcLength::ServoCharacterWidth(_) => false,
                // mozmm, depends on DPI. Computation resolves to lengths in
                // pixels and percentages, so these are not computationally
                // independent.
                #[cfg(feature = "gecko")]
                NoCalcLength::Physical(_) => false,
            }
        }

        fn check_calc(calc: &Box<CalcLengthOrPercentage>) -> bool {
            for part in &[&(**calc).em, &(**calc).ex, &(**calc).ch, &(**calc).rem] {
                match **part {
                    None => (),
                    Some(x) => {
                        if x != 0.0 {
                            return false
                        }
                    },
                }
            }
            true
        }

        fn check_length(length: &Length) -> bool {
            match *length {
                Length::NoCalc(ref length) => check_no_calc_length(length),
                Length::Calc(ref calc) => check_calc(calc),
            }
        }

        fn check_length_or_percentage(length_or_percentage: &LengthOrPercentage) -> bool {
            match *length_or_percentage {
                LengthOrPercentage::Length(ref length) => check_no_calc_length(length),
                LengthOrPercentage::Percentage(_) => true,
                LengthOrPercentage::Calc(ref calc) => check_calc(calc),
            }
        }

        fn check_lo_po_number(lo_po_number: &LengthOrPercentageOrNumber) -> bool {
            match *lo_po_number {
                First(_) => true,
                Second(ref length_or_percentage) => check_length_or_percentage(length_or_percentage),
            }
        }

        fn check_transform_list(transform_list: &transform::SpecifiedValue) -> bool {
            transform_list.0.iter().all(|operation| {
                match *operation {
                    SpecifiedOperation::Matrix(_) => true,
                    SpecifiedOperation::PrefixedMatrix(Matrix { ref e, ref f, .. }) => {
                        check_lo_po_number(e) &&
                        check_lo_po_number(f)
                    },
                    SpecifiedOperation::Matrix3D { .. } => true,
                    SpecifiedOperation::PrefixedMatrix3D { ref m41, ref m42, ref m43, .. } => {
                        check_lo_po_number(m41) &&
                        check_lo_po_number(m42) &&
                        (match *m43 {
                            First(ref length) => check_length(length),
                            Second(_) => true,
                        })
                    },
                    SpecifiedOperation::Skew(_, _) => true,
                    SpecifiedOperation::SkewX(_) => true,
                    SpecifiedOperation::SkewY(_) => true,
                    SpecifiedOperation::Translate(ref tx, ref ty) => {
                        check_length_or_percentage(tx) &&
                        (match ty {
                            &Some(ref ty) => check_length_or_percentage(ty),
                            &None => true,
                        })
                    },
                    SpecifiedOperation::TranslateX(ref tx) => check_length_or_percentage(tx),
                    SpecifiedOperation::TranslateY(ref ty) => check_length_or_percentage(ty),
                    SpecifiedOperation::TranslateZ(ref length) => check_length(length),
                    SpecifiedOperation::Translate3D(ref tx, ref ty, ref tz) => {
                        check_length_or_percentage(tx) &&
                        check_length_or_percentage(ty) &&
                        check_length(tz)
                    },
                    SpecifiedOperation::Scale(_, _) => true,
                    SpecifiedOperation::ScaleX(_) => true,
                    SpecifiedOperation::ScaleY(_) => true,
                    SpecifiedOperation::ScaleZ(_) => true,
                    SpecifiedOperation::Scale3D(_, _, _) => true,
                    SpecifiedOperation::Rotate(_) => true,
                    SpecifiedOperation::RotateX(_) => true,
                    SpecifiedOperation::RotateY(_) => true,
                    SpecifiedOperation::RotateZ(_) => true,
                    SpecifiedOperation::Rotate3D(_, _, _, _) => true,
                    SpecifiedOperation::Perspective(ref length) => check_length(length),
                    SpecifiedOperation::InterpolateMatrix { ref from_list, ref to_list, .. } => {
                        check_transform_list(from_list) &&
                        check_transform_list(to_list)
                    },
                    SpecifiedOperation::AccumulateMatrix { ref from_list, ref to_list, .. } => {
                        check_transform_list(from_list) &&
                        check_transform_list(to_list)
                    },
                }
            })
        }

        use self::SpecifiedValueItem::*;

        match *self {
            Length(ref length) => check_length(length),
            Number(_) => true,
            Percentage(_) => true,
            LengthPercentage(ref length_or_percentage) => check_length_or_percentage(length_or_percentage),
            Color(_) => true,
            Image(_) => true,
            Url(_) => true,
            Integer(_) => true,
            Angle(_) => true,
            Time(_) => true,
            TransformList(ref transform_list) => check_transform_list(transform_list),
            CustomIdent(_) => true,
            SpecificIdent(_) => true,
            TokenStream(_) => true,
        }
    }

    /// Attempts to convert this specified value to a computed value using the
    /// given context.
    pub fn to_computed_value(&self, context: &computed::Context) -> Result<ComputedValueItem, ()> {
        macro_rules! compute {
            ($_self:expr, $context:expr,

             $($typ:ident),*) => {
                match $_self {
                    $(
                        SpecifiedValueItem::$typ(ref value) => {
                            Ok(ComputedValueItem::$typ(value.to_computed_value($context)))
                        }
                    ), *

                    // Special cases.
                    // Would put in the match syntax, but we can't have things
                    // expand to match cases.

                    SpecifiedValueItem::Url(ref url) =>
                        ComputedUrl::from_specified(url).map(|x| ComputedValueItem::Url(x)),
                    SpecifiedValueItem::CustomIdent(ref ident) =>
                        Ok(ComputedValueItem::CustomIdent(ident.clone())),
                    SpecifiedValueItem::SpecificIdent(ref ident) =>
                        Ok(ComputedValueItem::SpecificIdent(ident.clone())),
                    SpecifiedValueItem::TokenStream(ref stream) =>
                        Ok(ComputedValueItem::TokenStream(stream.token_stream.clone())),
                }
            };
        }

        compute! {
            *self, context,

            Length, Number, Percentage, LengthPercentage, Color, Image, Integer,
            Angle, Time, TransformList
        }
    }
}

/// A specified typed value.
///
/// A value can either be a single item or a list of items.
#[derive(Clone, ToCss)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum SpecifiedValue {
    /// A single specified value.
    Item(SpecifiedValueItem),
    /// A list of one or more specified values.
    /// Note that we cannot have lists of <transform-lists>.
    List(Vec<SpecifiedValueItem>),
}

/// If `list` and `x` are both `Ok(..)`, append the contents of `x` to `list`.
/// To be used as an argument to fold to convert a sequence of `Result<T, E>`s
/// to a `Result<Vec<T>, E>` (pulling the `Result` outside).
fn all<T, E>(list: Result<Vec<T>, E>, x: Result<T, E>) -> Result<Vec<T>, E> {
    list.and_then(|mut list| {
        x.map(move |x| {
            list.push(x);
            list
        })
    })
}

impl SpecifiedValue {
    /// Returns whether or not this specified value is computationally
    /// independent.
    /// See the comment on SpecifiedValueItem::is_computationally_independent.
    pub fn is_computationally_independent(&self) -> bool {
        match *self {
            SpecifiedValue::Item(ref item) => item.is_computationally_independent(),
            SpecifiedValue::List(ref items) => items.iter().all(|x| x.is_computationally_independent()),
        }
    }

    /// Attempts to convert this specified value to a computed value.
    pub fn to_computed_value(&self, context: &computed::Context) -> Result<ComputedValue, ()> {
        match *self {
            SpecifiedValue::Item(ref item) => {
                item.to_computed_value(context).map(|x| ComputedValue::Item(x))
            }
            SpecifiedValue::List(ref items) => {
                // All of the items must compute successfully.
                items.iter()
                    .map(|x| x.to_computed_value(context))
                    .fold(Ok(Vec::new()), all)
                    .map(ComputedValue::List)
            },
        }
    }
}

/// A single computed typed value.
#[derive(Clone, Debug, PartialEq, ToCss)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ComputedValueItem {
    /// A single computed <length> value.
    Length(computed::Length),
    /// A single computed <number> value.
    Number(computed::Number),
    /// A single computed <percentage> value.
    Percentage(computed::Percentage),
    /// A single computed <length-percentage> value.
    LengthPercentage(computed::LengthOrPercentage),
    /// A single computed <color> value.
    Color(computed::Color),
    /// A single computed <image> value.
    Image(computed::Image),
    /// A single computed <url> value.
    Url(ComputedUrl),
    /// A single computed <integer> value.
    Integer(computed::Integer),
    /// A single computed <angle> value.
    Angle(computed::Angle),
    /// A single computed <time> value.
    Time(computed::Time),
    // FIXME: We should support <resolution> values as well.
    /// A single computed <transform-list> value (note that this is composed of
    /// multiple <transform-functions>.
    TransformList(transform::computed_value::T),
    /// A single computed <custom-ident> value.
    CustomIdent(Ident),
    /// A single specified <custom-ident> value that matches the specific ident
    /// contained herein.
    SpecificIdent(Ident),
    /// A computed <token-stream> value (the same as an uncomputed
    /// <token-stream> value).
    TokenStream(custom_properties::TokenStream),
}

/// A computed typed value.
#[derive(Clone, Debug, PartialEq, ToCss)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ComputedValue {
    /// A single value.
    Item(ComputedValueItem),
    /// A list of one or more values.
    List(Vec<ComputedValueItem>),
}

impl OneOrMoreSeparated for ComputedValueItem {
    type S = Space;
}

/// A map from CSS variable names to CSS variable computed values, used for
/// resolving.
///
/// This composes a custom_properties::CustomPropertiesMap, which maps CSS
/// variable names to their token stream values.
///
/// We also keep track of whether or not this contains any uninherited
/// properties, in order to just clone an Arc when possible.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomPropertiesMap {
    untyped_map: custom_properties::CustomPropertiesMap,
    typed_map: HashMap<Name, ComputedValue>,
    has_uninherited: bool,
}

impl CustomPropertiesMap {
    /// Creates a new custom properties map.
    pub fn new() -> Self {
        CustomPropertiesMap {
            untyped_map: custom_properties::CustomPropertiesMap::new(),
            typed_map: Default::default(),
            has_uninherited: false,
        }
    }

    // We reuse almost everything from custom_properties::CustomPropertiesMap
    // due to our Deref implementation.

    /// Insert both an uncomputed and computed value for a typed custom property
    /// if they have not previously been inserted. This must not be called for
    /// untyped custom properties. The untyped custom property is not inserted
    /// if it is not provided (currently this is only used so that
    /// custom_properties::substitute_one doesn't need to be aware of typed
    /// custom properties).
    pub fn insert_typed(
        &mut self,
        name: &Name,
        untyped_value: Option<custom_properties::ComputedValue>,
        typed_value: ComputedValue,
    ) {
        debug_assert!(!self.typed_map.contains_key(name));
        if let Some(untyped_value) = untyped_value {
            self.untyped_map.insert(name.clone(), untyped_value);
        }
        self.typed_map.insert(name.clone(), typed_value);
    }

    /// Get the (typed) computed value.
    /// It is not an error to call this with an untyped custom property: in that
    /// case we just return None (callers should themselves check whether or not
    /// a property has been registered).
    pub fn get_typed(&self, name: &Name) -> Option<&ComputedValue> {
        self.typed_map.get(name)
    }

    /// Mark that this map contains a non-initial value for an uninherited
    /// custom property.
    pub fn set_has_uninherited(&mut self) {
        self.has_uninherited = true
    }

    /// Return whether this map contains a non-initial value for an uninherited
    /// custom property.
    pub fn has_uninherited(&self) -> bool {
        self.has_uninherited
    }
}

impl Deref for CustomPropertiesMap {
    type Target = custom_properties::CustomPropertiesMap;

    fn deref(&self) -> &Self::Target {
        &self.untyped_map
    }
}

impl DerefMut for CustomPropertiesMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.untyped_map
    }
}
