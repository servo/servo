/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{vec, iter};
use std::ascii::StrAsciiExt;
use extra::arc::Arc;

use cssparser::ast::*;
use cssparser::parse_nth;

use namespaces::NamespaceMap;


// Only used in tests
impl Eq for Arc<CompoundSelector> {
    fn eq(&self, other: &Arc<CompoundSelector>) -> bool {
        self.get() == other.get()
    }
}


#[deriving(Eq, Clone)]
pub struct Selector {
    compound_selectors: Arc<CompoundSelector>,
    pseudo_element: Option<PseudoElement>,
    specificity: u32,
}

#[deriving(Eq, Clone)]
pub enum PseudoElement {
    Before,
    After,
//    FirstLine,
//    FirstLetter,
}


#[deriving(Eq, Clone)]
pub struct CompoundSelector {
    simple_selectors: ~[SimpleSelector],
    next: Option<(~CompoundSelector, Combinator)>,  // c.next is left of c
}

#[deriving(Eq, Clone)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

#[deriving(Eq, Clone)]
pub enum SimpleSelector {
    IDSelector(~str),
    ClassSelector(~str),
    LocalNameSelector(~str),
    NamespaceSelector(~str),

    // Attribute selectors
    AttrExists(AttrSelector),  // [foo]
    AttrEqual(AttrSelector, ~str),  // [foo=bar]
    AttrIncludes(AttrSelector, ~str),  // [foo~=bar]
    AttrDashMatch(AttrSelector, ~str, ~str),  // [foo|=bar]  Second string is the first + "-"
    AttrPrefixMatch(AttrSelector, ~str),  // [foo^=bar]
    AttrSubstringMatch(AttrSelector, ~str),  // [foo*=bar]
    AttrSuffixMatch(AttrSelector, ~str),  // [foo$=bar]

    // Pseudo-classes
    Negation(~[SimpleSelector]),
    AnyLink,
    Link,
    Visited,
    FirstChild, LastChild, OnlyChild,
//    Empty,
    Root,
//    Lang(~str),
    NthChild(i32, i32),
    NthLastChild(i32, i32),
    NthOfType(i32, i32),
    NthLastOfType(i32, i32),
    FirstOfType,
    LastOfType,
    OnlyOfType
    // ...
}

#[deriving(Eq, Clone)]
pub struct AttrSelector {
    name: ~str,
    namespace: Option<~str>,
}


type Iter = iter::Peekable<ComponentValue, vec::MoveIterator<ComponentValue>>;


/// Parse a comma-separated list of Selectors.
/// aka Selector Group in http://www.w3.org/TR/css3-selectors/#grouping
///
/// Return the Selectors or None if there is an invalid selector.
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
            Some(&Comma) => {
                iter.next();
            }
            _ => return None,
        }
        match parse_selector(iter, namespaces) {
            Some(selector) => results.push(selector),
            None => return None,
        }
    }
    Some(results)
}


/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// None means invalid selector.
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
            Some(&Comma) => break,
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
    Some(Selector {
        specificity: compute_specificity(&compound, &pseudo_element),
        compound_selectors: Arc::new(compound),
        pseudo_element: pseudo_element,
    })
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
                &LocalNameSelector(..) => specificity.element_selectors += 1,
                &IDSelector(..) => specificity.id_selectors += 1,
                &ClassSelector(..)
                | &AttrExists(..) | &AttrEqual(..) | &AttrIncludes(..) | &AttrDashMatch(..)
                | &AttrPrefixMatch(..) | &AttrSubstringMatch(..) | &AttrSuffixMatch(..)
                | &AnyLink | &Link | &Visited
                | &FirstChild | &LastChild | &OnlyChild | &Root
//                | &Empty | &Lang(*)
                | &NthChild(..) | &NthLastChild(..)
                | &NthOfType(..) | &NthLastOfType(..)
                | &FirstOfType | &LastOfType | &OnlyOfType
                => specificity.class_like_selectors += 1,
                &NamespaceSelector(..) => (),
                &Negation(ref negated)
                => simple_selectors_specificity(negated.as_slice(), specificity),
            }
        }
    }

    static MAX_10BIT: u32 = (1u32 << 10) - 1;
    specificity.id_selectors.min(&MAX_10BIT) << 20
    | specificity.class_like_selectors.min(&MAX_10BIT) << 10
    | specificity.element_selectors.min(&MAX_10BIT)
}


/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
/// 
/// None means invalid selector
fn parse_simple_selectors(iter: &mut Iter, namespaces: &NamespaceMap)
                           -> Option<(~[SimpleSelector], Option<PseudoElement>)> {
    let mut empty = true;
    let mut simple_selectors = match parse_type_selector(iter, namespaces) {
        InvalidTypeSelector => return None,
        NotATypeSelector => ~[],
        TypeSelector(s) => { empty = false; s }
    };

    let mut pseudo_element = None;
    loop {
        match parse_one_simple_selector(iter, namespaces, /* inside_negation = */ false) {
            InvalidSimpleSelector => return None,
            NotASimpleSelector => break,
            SimpleSelectorResult(s) => { simple_selectors.push(s); empty = false },
            PseudoElementResult(p) => { pseudo_element = Some(p); break },
        }
    }
    if empty { None }  // An empty selector is invalid
    else { Some((simple_selectors, pseudo_element)) }
}


enum TypeSelectorParseResult {
    InvalidTypeSelector,
    NotATypeSelector,
    TypeSelector(~[SimpleSelector]),  // Length 0 (*|*), 1 (*|E or ns|*) or 2 (|E or ns|E)
}

fn parse_type_selector(iter: &mut Iter, namespaces: &NamespaceMap)
                       -> TypeSelectorParseResult {
    skip_whitespace(iter);
    match parse_qualified_name(iter, /* allow_universal = */ true, namespaces) {
        InvalidQualifiedName => InvalidTypeSelector,
        NotAQualifiedName => NotATypeSelector,
        QualifiedName(namespace, local_name) => {
            let mut simple_selectors = ~[];
            match namespace {
                Some(url) => simple_selectors.push(NamespaceSelector(url)),
                None => (),
            }
            match local_name {
                Some(name) => simple_selectors.push(LocalNameSelector(name)),
                None => (),
            }
            TypeSelector(simple_selectors)
        }
    }
}


enum SimpleSelectorParseResult {
    InvalidSimpleSelector,
    NotASimpleSelector,
    SimpleSelectorResult(SimpleSelector),
    PseudoElementResult(PseudoElement),
}

// Parse a simple selector other than a type selector
fn parse_one_simple_selector(iter: &mut Iter, namespaces: &NamespaceMap, inside_negation: bool)
                         -> SimpleSelectorParseResult {
    match iter.peek() {
        Some(&IDHash(_)) => match iter.next() {
            Some(IDHash(id)) => SimpleSelectorResult(IDSelector(id)),
            _ => fail!("Implementation error, this should not happen."),
        },
        Some(&Delim('.')) => {
            iter.next();
            match iter.next() {
                Some(Ident(class)) => SimpleSelectorResult(ClassSelector(class)),
                _ => InvalidSimpleSelector,
            }
        }
        Some(&SquareBracketBlock(_)) => match iter.next() {
            Some(SquareBracketBlock(content))
            => match parse_attribute_selector(content, namespaces) {
                None => InvalidSimpleSelector,
                Some(simple_selector) => SimpleSelectorResult(simple_selector),
            },
            _ => fail!("Implementation error, this should not happen."),
        },
        Some(&Colon) => {
            iter.next();
            match iter.next() {
                Some(Ident(name)) => match parse_simple_pseudo_class(name) {
                    None => {
                        // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
                        let name_lower = name.to_ascii_lower();
                        match name_lower.as_slice() {
                            // Supported CSS 2.1 pseudo-elements only.
                            // ** Do not add to this list! **
                            "before" => PseudoElementResult(Before),
                            "after" => PseudoElementResult(After),
//                            "first-line" => PseudoElementResult(FirstLine),
//                            "first-letter" => PseudoElementResult(FirstLetter),
                            _ => InvalidSimpleSelector
                        }
                    },
                    Some(result) => SimpleSelectorResult(result),
                },
                Some(Function(name, arguments)) => match parse_functional_pseudo_class(
                        name, arguments, namespaces, inside_negation) {
                    None => InvalidSimpleSelector,
                    Some(simple_selector) => SimpleSelectorResult(simple_selector),
                },
                Some(Colon) => {
                    match iter.next() {
                        Some(Ident(name)) => match parse_pseudo_element(name) {
                            Some(pseudo_element) => PseudoElementResult(pseudo_element),
                            _ => InvalidSimpleSelector,
                        },
                        _ => InvalidSimpleSelector,
                    }
                }
                _ => InvalidSimpleSelector,
            }
        }
        _ => NotASimpleSelector,
    }
}


enum QualifiedNameParseResult {
    InvalidQualifiedName,
    NotAQualifiedName,
    QualifiedName(Option<~str>, Option<~str>)  // Namespace URL, local name. None means '*'
}

fn parse_qualified_name(iter: &mut Iter, allow_universal: bool, namespaces: &NamespaceMap)
                       -> QualifiedNameParseResult {
    #[inline]
    fn default_namespace(namespaces: &NamespaceMap, local_name: Option<~str>)
                         -> QualifiedNameParseResult {
        QualifiedName(namespaces.default.as_ref().map(|url| url.to_owned()), local_name)
    }

    #[inline]
    fn explicit_namespace(iter: &mut Iter, allow_universal: bool, namespace_url: Option<~str>)
                         -> QualifiedNameParseResult {
        assert!(iter.next() == Some(Delim('|')),
                "Implementation error, this should not happen.");
        match iter.peek() {
            Some(&Delim('*')) if allow_universal => {
                iter.next();
                QualifiedName(namespace_url, None)
            },
            Some(&Ident(_)) => {
                let local_name = get_next_ident(iter);
                QualifiedName(namespace_url, Some(local_name))
            },
            _ => InvalidQualifiedName,
        }
    }

    match iter.peek() {
        Some(&Ident(_)) => {
            let value = get_next_ident(iter);
            match iter.peek() {
                Some(&Delim('|')) => {
                    let namespace_url = match namespaces.prefix_map.find(&value) {
                        None => return InvalidQualifiedName,  // Undeclared namespace prefix
                        Some(ref url) => url.to_owned(),
                    };
                    explicit_namespace(iter, allow_universal, Some(namespace_url))
                },
                _ => default_namespace(namespaces, Some(value)),
            }
        },
        Some(&Delim('*')) => {
            iter.next();  // Consume '*'
            match iter.peek() {
                Some(&Delim('|')) => explicit_namespace(iter, allow_universal, None),
                _ => {
                    if allow_universal { default_namespace(namespaces, None) }
                    else { InvalidQualifiedName }
                },
            }
        },
        Some(&Delim('|')) => explicit_namespace(iter, allow_universal, Some(~"")),
        _ => NotAQualifiedName,
    }
}


fn parse_attribute_selector(content: ~[ComponentValue], namespaces: &NamespaceMap)
                            -> Option<SimpleSelector> {
    let iter = &mut content.move_iter().peekable();
    let attr = match parse_qualified_name(iter, /* allow_universal = */ false, namespaces) {
        InvalidQualifiedName | NotAQualifiedName => return None,
        QualifiedName(_, None) => fail!("Implementation error, this should not happen."),
        QualifiedName(namespace, Some(local_name)) => AttrSelector {
            namespace: namespace,
            name: local_name,
        },
    };
    skip_whitespace(iter);
    // TODO: deal with empty value or value containing whitespace (see spec)
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
        Some(DashMatch) => {
            let value = get_value!();
            let dashing_value = value + "-";
            AttrDashMatch(attr, value, dashing_value)  // [foo|=bar]
        },
        Some(PrefixMatch) => AttrPrefixMatch(attr, get_value!()),  // [foo^=bar]
        Some(SubstringMatch) => AttrSubstringMatch(attr, get_value!()),  // [foo*=bar]
        Some(SuffixMatch) => AttrSuffixMatch(attr, get_value!()),  // [foo$=bar]
        _ => return None
    };
    skip_whitespace(iter);
    if iter.next().is_none() { Some(result) } else { None }
}


fn parse_simple_pseudo_class(name: &str) -> Option<SimpleSelector> {
    // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
    let name_lower = name.to_ascii_lower(); 
    match name_lower.as_slice() {
        "any-link" => Some(AnyLink),
        "link" => Some(Link),
        "visited" => Some(Visited),
        "first-child" => Some(FirstChild),
        "last-child"  => Some(LastChild),
        "only-child"  => Some(OnlyChild),
        "root" => Some(Root),
        "first-of-type" => Some(FirstOfType),
        "last-of-type"  => Some(LastOfType),
        "only-of-type"  => Some(OnlyOfType),
//        "empty" => Some(Empty),
        _ => None
    }
}


fn parse_functional_pseudo_class(name: ~str, arguments: ~[ComponentValue],
                                 namespaces: &NamespaceMap, inside_negation: bool)
                                 -> Option<SimpleSelector> {
    // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
    let name_lower = name.to_ascii_lower();
    match name_lower.as_slice() {
//        "lang" => parse_lang(arguments),
        "nth-child"        => parse_nth(arguments).map(|(a, b)| NthChild(a, b)),
        "nth-last-child"   => parse_nth(arguments).map(|(a, b)| NthLastChild(a, b)),
        "nth-of-type"      => parse_nth(arguments).map(|(a, b)| NthOfType(a, b)),
        "nth-last-of-type" => parse_nth(arguments).map(|(a, b)| NthLastOfType(a, b)),
        "not" => if inside_negation { None } else { parse_negation(arguments, namespaces) },
        _ => None
    }
}


fn parse_pseudo_element(name: ~str) -> Option<PseudoElement> {
    // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
    let name_lower = name.to_ascii_lower();
    match name_lower.as_slice() {
        // All supported pseudo-elements
        "before" => Some(Before),
        "after" => Some(After),
//        "first-line" => Some(FirstLine),
//        "first-letter" => Some(FirstLetter),
        _ => None
    }
}


//fn parse_lang(arguments: ~[ComponentValue]) -> Option<SimpleSelector> {
//    let mut iter = arguments.move_skip_whitespace();
//    match iter.next() {
//        Some(Ident(value)) => {
//            if "" == value || iter.next().is_some() { None }
//            else { Some(Lang(value)) }
//        },
//        _ => None,
//    }
//}


// Level 3: Parse ONE simple_selector
fn parse_negation(arguments: ~[ComponentValue], namespaces: &NamespaceMap)
                  -> Option<SimpleSelector> {
    let iter = &mut arguments.move_iter().peekable();
    Some(Negation(match parse_type_selector(iter, namespaces) {
        InvalidTypeSelector => return None,
        TypeSelector(s) => s,
        NotATypeSelector => {
            match parse_one_simple_selector(iter, namespaces, /* inside_negation = */ true) {
                SimpleSelectorResult(s) => ~[s],
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


#[cfg(test)]
mod tests {
    use extra::arc::Arc;
    use cssparser;
    use namespaces::NamespaceMap;
    use super::*;

    fn parse(input: &str) -> Option<~[Selector]> {
        parse_selector_list(
            cssparser::tokenize(input).map(|(v, _)| v).to_owned_vec(),
            &NamespaceMap::new())
    }

    fn specificity(a: u32, b: u32, c: u32) -> u32 {
        a << 20 | b << 10 | c
    }

    #[test]
    fn test_parsing() {
        assert_eq!(parse(""), None)
        assert_eq!(parse("e"), Some(~[Selector{
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: ~[LocalNameSelector(~"e")],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        }]))
        assert_eq!(parse(".foo"), Some(~[Selector{
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: ~[ClassSelector(~"foo")],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        }]))
        assert_eq!(parse("#bar"), Some(~[Selector{
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: ~[IDSelector(~"bar")],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        }]))
        assert_eq!(parse("e.foo#bar"), Some(~[Selector{
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: ~[LocalNameSelector(~"e"),
                                    ClassSelector(~"foo"),
                                    IDSelector(~"bar")],
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        }]))
        assert_eq!(parse("e.foo #bar"), Some(~[Selector{
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: ~[IDSelector(~"bar")],
                next: Some((~CompoundSelector {
                    simple_selectors: ~[LocalNameSelector(~"e"),
                                        ClassSelector(~"foo")],
                    next: None,
                }, Descendant)),
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        }]))
    }
}
