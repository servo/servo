/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{vec, iterator};
use std::ascii::StrAsciiExt;
use cssparser::*;
use style::namespaces::NamespaceMap;


pub struct Selector {
    compound_selectors: CompoundSelector,
    pseudo_element: Option<PseudoElement>,
    specificity: u32,
}

pub static STYLE_ATTRIBUTE_SPECIFICITY: u32 = 1 << 31;


pub enum PseudoElement {
    Before,
    After,
    FirstLine,
    FirstLetter,
}


pub struct CompoundSelector {
    simple_selectors: ~[SimpleSelector],
    next: Option<(~CompoundSelector, Combinator)>,  // c.next is left of c
}

pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

pub enum SimpleSelector {
    IDSelector(~str),
    ClassSelector(~str),
    LocalNameSelector{lowercase_name: ~str, cased_name: ~str},
    NamespaceSelector(~str),

    // Attribute selectors
    AttrExists(AttrSelector),  // [foo]
    AttrEqual(AttrSelector, ~str),  // [foo=bar]
    AttrIncludes(AttrSelector, ~str),  // [foo~=bar]
    AttrDashMatch(AttrSelector, ~str),  // [foo|=bar]
    AttrPrefixMatch(AttrSelector, ~str),  // [foo^=bar]
    AttrSubstringMatch(AttrSelector, ~str),  // [foo*=bar]
    AttrSuffixMatch(AttrSelector, ~str),  // [foo$=bar]

    // Pseudo-classes
    Empty,
    Root,
    Lang(~str),
    NthChild(i32, i32),
    Negation(~[SimpleSelector]),
    // ...
}

pub struct AttrSelector {
    lowercase_name: ~str,
    cased_name: ~str,
    namespace: Option<~str>,
}


type Iter = iterator::Peekable<ComponentValue, vec::MoveIterator<ComponentValue>>;


// None means invalid selector
pub fn parse_selector_list(input: ~[ComponentValue], namespaces: &NamespaceMap)
                           -> Option<~[Selector]> {
    let iter = &mut input.move_iter().peekable();
    let first = match parse_selector(iter, namespaces) {
        None => return None,
        Some(result) => result
    };
    let mut results = ~[first];

    loop {
        skip_whitespace(iter);
        match iter.peek() {
            None => break,  // EOF
            Some(&Comma) => (),
            _ => return None,
        }
        match parse_selector(iter, namespaces) {
            Some(selector) => results.push(selector),
            None => return None,
        }
    }
    Some(results)
}


// None means invalid selector
fn parse_selector(iter: &mut Iter, namespaces: &NamespaceMap)
                  -> Option<Selector> {
    let (first, pseudo_element) = match parse_simple_selectors(iter, namespaces) {
        None => return None,
        Some(result) => result
    };
    let mut compound = CompoundSelector{ simple_selectors: first, next: None };
    let mut pseudo_element = pseudo_element;

    while pseudo_element.is_none() {
        let any_whitespace = skip_whitespace(iter);
        let combinator = match iter.peek() {
            None => break,  // EOF
            Some(&Delim('>')) => { iter.next(); Child },
            Some(&Delim('+')) => { iter.next(); NextSibling },
            Some(&Delim('~')) => { iter.next(); LaterSibling },
            Some(_) => {
                if any_whitespace { Descendant }
                else { return None }
            }
        };
        match parse_simple_selectors(iter, namespaces) {
            None => return None,
            Some((simple_selectors, pseudo)) => {
                compound = CompoundSelector {
                    simple_selectors: simple_selectors,
                    next: Some((~compound, combinator))
                };
                pseudo_element = pseudo;
            }
        }
    }
    let selector = Selector{
        specificity: compute_specificity(&compound, &pseudo_element),
        compound_selectors: compound,
        pseudo_element: pseudo_element,
    };
    Some(selector)
}


fn compute_specificity(mut selector: &CompoundSelector,
                       pseudo_element: &Option<PseudoElement>) -> u32 {
    struct Specificity {
        id_selectors: u32,
        class_like_selectors: u32,
        element_selectors: u32,
    }
    let mut specificity = Specificity {
        id_selectors: 0,
        class_like_selectors: 0,
        element_selectors: 0,
    };
    if pseudo_element.is_some() { specificity.element_selectors += 1 }

    simple_selectors_specificity(selector.simple_selectors, &mut specificity);
    loop {
        match selector.next {
            None => break,
            Some((ref next_selector, _)) => {
                selector = &**next_selector;
                simple_selectors_specificity(selector.simple_selectors, &mut specificity)
            }
        }
    }

    fn simple_selectors_specificity(simple_selectors: &[SimpleSelector],
                                    specificity: &mut Specificity) {
        for simple_selector in simple_selectors.iter() {
            match simple_selector {
                &LocalNameSelector{_} => specificity.element_selectors += 1,
                &IDSelector(*) => specificity.id_selectors += 1,
                &ClassSelector(*)
                | &AttrExists(*) | &AttrEqual(*) | &AttrIncludes(*) | &AttrDashMatch(*)
                | &AttrPrefixMatch(*) | &AttrSubstringMatch(*) | &AttrSuffixMatch(*)
                | &Empty | &Root | &Lang(*) | &NthChild(*)
                => specificity.class_like_selectors += 1,
                &NamespaceSelector(*) => (),
                &Negation(ref negated)
                => simple_selectors_specificity(negated.as_slice(), specificity),
            }
        }
    }

    static MAX_10BIT: u32 = (1u32 << 10) - 1;
    specificity.id_selectors.min(&MAX_10BIT) << 20
    | specificity.class_like_selectors.min(&MAX_10BIT) << 10
    | specificity.id_selectors.min(&MAX_10BIT)
}


// None means invalid selector
fn parse_simple_selectors(iter: &mut Iter, namespaces: &NamespaceMap)
                           -> Option<(~[SimpleSelector], Option<PseudoElement>)> {
    let mut empty = true;
    let mut simple_selectors = match parse_type_selector(iter, namespaces) {
        None => return None,  // invalid selector
        Some(None) => ~[],
        Some(Some(s)) => { empty = false; s }
    };

    let mut pseudo_element = None;
    loop {
        match parse_one_simple_selector(iter, namespaces, /* inside_negation = */ false) {
            None => return None, // invalid selector
            Some(None) => break,
            Some(Some(Left(s))) => simple_selectors.push(s),
            Some(Some(Right(p))) => { pseudo_element = Some(p); break },
        }
    }
    if empty { None }  // An empty selector is invalid
    else { Some((simple_selectors, pseudo_element)) }
}


// None means invalid selector
// Some(None) means no type selector
// Some(Some([...])) is a type selector. Might be empty for *|*
fn parse_type_selector(iter: &mut Iter, namespaces: &NamespaceMap)
                       -> Option<Option<~[SimpleSelector]>> {
    skip_whitespace(iter);
    match parse_qualified_name(iter, /* allow_universal = */ true, namespaces) {
        None => None,  // invalid selector
        Some(None) => Some(None),
        Some(Some((namespace, local_name))) => {
            let mut simple_selectors = ~[];
            match namespace {
                Some(url) => simple_selectors.push(NamespaceSelector(url)),
                None => (),
            }
            match local_name {
                Some(name) => simple_selectors.push(LocalNameSelector{
                    lowercase_name: name.to_ascii_lower(),
                    cased_name: name,
                }),
                None => (),
            }
            Some(Some(simple_selectors))
        }
    }
}


// Parse a simple selector other than a type selector
fn parse_one_simple_selector(iter: &mut Iter, namespaces: &NamespaceMap, inside_negation: bool)
                         -> Option<Option<Either<SimpleSelector, PseudoElement>>> {
    match iter.peek() {
        Some(&IDHash(_)) => match iter.next() {
            Some(IDHash(id)) => Some(Some(Left(IDSelector(id)))),
            _ => fail!("Implementation error, this should not happen."),
        },
        Some(&Delim('.')) => {
            iter.next();
            match iter.next() {
                Some(Ident(class)) => Some(Some(Left(ClassSelector(class)))),
                _ => None,  // invalid selector
            }
        }
        Some(&SquareBracketBlock(_)) => match iter.next() {
            Some(SquareBracketBlock(content))
            => match parse_attribute_selector(content, namespaces) {
                None => None,
                Some(simple_selector) => Some(Some(Left(simple_selector))),
            },
            _ => fail!("Implementation error, this should not happen."),
        },
        Some(&Delim(':')) => {
            iter.next();
            match iter.next() {
                Some(Ident(name)) => match parse_simple_pseudo_class(name) {
                    None => None,
                    Some(result) => Some(Some(result)),
                },
                Some(Function(name, arguments)) => match parse_functional_pseudo_class(
                        name, arguments, namespaces, inside_negation) {
                    None => None,
                    Some(simple_selector) => Some(Some(Left(simple_selector))),
                },
                Some(Delim(':')) => {
                    match iter.next() {
                        Some(Ident(name)) => match parse_pseudo_element(name) {
                            Some(pseudo_element) => Some(Some(Right(pseudo_element))),
                            _ => None,
                        },
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => Some(None),
    }
}

// None means invalid selector
// Some(None) means not a qualified name
// Some(Some((None, None)) means *|*
// Some(Some((Some(url), None)) means prefix|*
// Some(Some((None, Some(name)) means *|name
// Some(Some((Some(url), Some(name))) means prefix|name
// ... or equivalent
fn parse_qualified_name(iter: &mut Iter, allow_universal: bool, namespaces: &NamespaceMap)
                       -> Option<Option<(Option<~str>, Option<~str>)>> {
    #[inline]
    fn default_namespace(namespaces: &NamespaceMap, local_name: Option<~str>)
                         -> Option<Option<(Option<~str>, Option<~str>)>> {
        match namespaces.default {
            None => Some(Some((None, local_name))),
            Some(ref url) => Some(Some((Some(url.to_owned()), local_name))),
        }
    }

    #[inline]
    fn explicit_namespace(iter: &mut Iter, allow_universal: bool, namespace_url: Option<~str>)
                         -> Option<Option<(Option<~str>, Option<~str>)>> {
        assert!(iter.next() == Some(Delim('|')));
        match iter.peek() {
            Some(&Delim('*')) if allow_universal => {
                iter.next();
                Some(Some((namespace_url, None)))
            },
            Some(&Ident(_)) => {
                let local_name = get_next_ident(iter);
                Some(Some((namespace_url, Some(local_name))))
            },
            _ => None,  // invalid selector
        }
    }

    match iter.peek() {
        Some(&Ident(_)) => {
            let value = get_next_ident(iter);
            match iter.peek() {
                Some(&Delim('|')) => default_namespace(namespaces, Some(value)),
                _ => {
                    let namespace_url = match namespaces.prefix_map.find(&value) {
                        None => return None,  // Undeclared namespace prefix: invalid selector
                        Some(ref url) => url.to_owned(),
                    };
                    explicit_namespace(iter, allow_universal, Some(namespace_url))
                },
            }
        },
        Some(&Delim('*')) => {
            iter.next();  // Consume '*'
            match iter.peek() {
                Some(&Delim('|')) => {
                    if allow_universal { default_namespace(namespaces, None) }
                    else { None }
                },
                _ => explicit_namespace(iter, allow_universal, None),
            }
        },
        Some(&Delim('|')) => explicit_namespace(iter, allow_universal, Some(~"")),
        _ => return None,
    }
}


fn parse_attribute_selector(content: ~[ComponentValue], namespaces: &NamespaceMap)
                            -> Option<SimpleSelector> {
    let iter = &mut content.move_iter().peekable();
    let attr = match parse_qualified_name(iter, /* allow_universal = */ false, namespaces) {
        None => return None,  // invalid selector
        Some(None) => return None,
        Some(Some((_, None))) => fail!("Implementation error, this should not happen."),
        Some(Some((namespace, Some(local_name)))) => AttrSelector {
            namespace: namespace,
            lowercase_name: local_name.to_ascii_lower(),
            cased_name: local_name,
        },
    };
    skip_whitespace(iter);
    macro_rules! get_value( () => {{
        skip_whitespace(iter);
        match iter.next() {
            Some(Ident(value)) | Some(String(value)) => value,
            _ => return None,
        }
    }};)
    let result = match iter.next() {
        None => AttrExists(attr),  // [foo]
        Some(Delim('=')) => AttrEqual(attr, get_value!()),  // [foo=bar]
        Some(IncludeMatch) => AttrIncludes(attr, get_value!()),  // [foo~=bar]
        Some(DashMatch) => AttrDashMatch(attr, get_value!()),  // [foo|=bar]
        Some(PrefixMatch) => AttrPrefixMatch(attr, get_value!()),  // [foo^=bar]
        Some(SubstringMatch) => AttrSubstringMatch(attr, get_value!()),  // [foo*=bar]
        Some(SuffixMatch) => AttrSuffixMatch(attr, get_value!()),  // [foo$=bar]
        _ => return None
    };
    skip_whitespace(iter);
    if iter.next().is_none() { Some(result) } else { None }
}


fn parse_simple_pseudo_class(name: ~str) -> Option<Either<SimpleSelector, PseudoElement>> {
    match name.to_ascii_lower().as_slice() {
        "root" => Some(Left(Root)),
        "empty" => Some(Left(Empty)),

        // Supported CSS 2.1 pseudo-elements only.
        "before" => Some(Right(Before)),
        "after" => Some(Right(After)),
        "first-line" => Some(Right(FirstLine)),
        "first-letter" => Some(Right(FirstLetter)),
        _ => None
    }
}


fn parse_functional_pseudo_class(name: ~str, arguments: ~[ComponentValue],
                                 namespaces: &NamespaceMap, inside_negation: bool)
                                 -> Option<SimpleSelector> {
    match name.to_ascii_lower().as_slice() {
        "lang" => parse_lang(arguments),
        "nth-child" => parse_nth(arguments).map(|&(a, b)| NthChild(a, b)),
        "not" => if inside_negation { None } else { parse_negation(arguments, namespaces) },
        _ => None
    }
}


fn parse_pseudo_element(name: ~str) -> Option<PseudoElement> {
    match name.to_ascii_lower().as_slice() {
        // All supported pseudo-elements
        "before" => Some(Before),
        "after" => Some(After),
        "first-line" => Some(FirstLine),
        "first-letter" => Some(FirstLetter),
        _ => None
    }
}


fn parse_lang(arguments: ~[ComponentValue]) -> Option<SimpleSelector> {
    let mut iter = arguments.move_skip_whitespace();
    match iter.next() {
        Some(Ident(value)) => {
            if "" == value || iter.next().is_some() { None }
            else { Some(Lang(value)) }
        },
        _ => None,
    }
}


// Level 3: Parse ONE simple_selector
fn parse_negation(arguments: ~[ComponentValue], namespaces: &NamespaceMap)
                  -> Option<SimpleSelector> {
    let iter = &mut arguments.move_iter().peekable();
    Some(Negation(match parse_type_selector(iter, namespaces) {
        None => return None,  // invalid selector
        Some(Some(s)) => s,
        Some(None) => {
            match parse_one_simple_selector(iter, namespaces, /* inside_negation = */ true) {
                Some(Some(Left(s))) => ~[s],
                _ => return None
            }
        },
    }))
}


/// Assuming the next token is an ident, consume it and return its value
#[inline]
fn get_next_ident(iter: &mut Iter) -> ~str {
    match iter.next() {
        Some(Ident(value)) => value,
        _ => fail!("Implementation error, this should not happen."),
    }
}


#[inline]
fn skip_whitespace(iter: &mut Iter) -> bool {
    let mut any_whitespace = false;
    loop {
        if iter.peek() != Some(&WhiteSpace) { return any_whitespace }
        any_whitespace = true;
        iter.next();
    }
}
