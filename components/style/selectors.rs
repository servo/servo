/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{cmp, iter};
use std::ascii::{StrAsciiExt, OwnedStrAsciiExt};
use sync::Arc;

use cssparser::ast::*;
use cssparser::{tokenize, parse_nth};

use servo_util::atom::Atom;
use servo_util::namespace::Namespace;
use servo_util::namespace;

use namespaces::NamespaceMap;


// Only used in tests
impl PartialEq for Arc<CompoundSelector> {
    fn eq(&self, other: &Arc<CompoundSelector>) -> bool {
        **self == **other
    }
}


#[deriving(PartialEq, Clone)]
pub struct Selector {
    pub compound_selectors: Arc<CompoundSelector>,
    pub pseudo_element: Option<PseudoElement>,
    pub specificity: u32,
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum PseudoElement {
    Before,
    After,
//    FirstLine,
//    FirstLetter,
}


#[deriving(PartialEq, Clone)]
pub struct CompoundSelector {
    pub simple_selectors: Vec<SimpleSelector>,
    pub next: Option<(Box<CompoundSelector>, Combinator)>,  // c.next is left of c
}

#[deriving(PartialEq, Clone)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum SimpleSelector {
    IDSelector(Atom),
    ClassSelector(Atom),
    LocalNameSelector(LocalName),
    NamespaceSelector(Namespace),

    // Attribute selectors
    AttrExists(AttrSelector),  // [foo]
    AttrEqual(AttrSelector, String),  // [foo=bar]
    AttrIncludes(AttrSelector, String),  // [foo~=bar]
    AttrDashMatch(AttrSelector, String, String),  // [foo|=bar]  Second string is the first + "-"
    AttrPrefixMatch(AttrSelector, String),  // [foo^=bar]
    AttrSubstringMatch(AttrSelector, String),  // [foo*=bar]
    AttrSuffixMatch(AttrSelector, String),  // [foo$=bar]

    // Pseudo-classes
    Negation(Vec<SimpleSelector>),
    AnyLink,
    Link,
    Visited,
    Hover,
    Disabled,
    Enabled,
    FirstChild, LastChild, OnlyChild,
//    Empty,
    Root,
//    Lang(String),
    NthChild(i32, i32),
    NthLastChild(i32, i32),
    NthOfType(i32, i32),
    NthLastOfType(i32, i32),
    FirstOfType,
    LastOfType,
    OnlyOfType
    // ...
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub struct LocalName {
    pub name: Atom,
    pub lower_name: Atom,
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub struct AttrSelector {
    pub name: String,
    pub lower_name: String,
    pub namespace: NamespaceConstraint,
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum NamespaceConstraint {
    AnyNamespace,
    SpecificNamespace(Namespace),
}


pub fn parse_selector_list_from_str(input: &str) -> Result<SelectorList, ()> {
    let namespaces = NamespaceMap::new();
    let iter = tokenize(input).map(|(token, _)| token);
    parse_selector_list(iter, &namespaces).map(|s| SelectorList { selectors: s })
}

/// Re-exported to script, but opaque.
pub struct SelectorList {
    selectors: Vec<Selector>
}

/// Public to the style crate, but not re-exported to script
pub fn get_selector_list_selectors<'a>(selector_list: &'a SelectorList) -> &'a [Selector] {
    selector_list.selectors.as_slice()
}

/// Parse a comma-separated list of Selectors.
/// aka Selector Group in http://www.w3.org/TR/css3-selectors/#grouping
///
/// Return the Selectors or None if there is an invalid selector.
pub fn parse_selector_list<I: Iterator<ComponentValue>>(
                           iter: I, namespaces: &NamespaceMap)
                           -> Result<Vec<Selector>, ()> {
    let iter = &mut iter.peekable();
    let mut results = vec![try!(parse_selector(iter, namespaces))];

    loop {
        skip_whitespace(iter);
        match iter.peek() {
            None => break,  // EOF
            Some(&Comma) => {
                iter.next();
            }
            _ => return Err(()),
        }
        results.push(try!(parse_selector(iter, namespaces)));
    }
    Ok(results)
}


type Iter<I> = iter::Peekable<ComponentValue, I>;

/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// `Err` means invalid selector.
fn parse_selector<I: Iterator<ComponentValue>>(
                  iter: &mut Iter<I>, namespaces: &NamespaceMap)
                  -> Result<Selector, ()> {
    let (first, mut pseudo_element) = try!(parse_simple_selectors(iter, namespaces));
    let mut compound = CompoundSelector{ simple_selectors: first, next: None };

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
                else { return Err(()) }
            }
        };
        let (simple_selectors, pseudo) = try!(parse_simple_selectors(iter, namespaces));
        compound = CompoundSelector {
            simple_selectors: simple_selectors,
            next: Some((box compound, combinator))
        };
        pseudo_element = pseudo;
    }
    Ok(Selector {
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

    simple_selectors_specificity(selector.simple_selectors.as_slice(), &mut specificity);
    loop {
        match selector.next {
            None => break,
            Some((ref next_selector, _)) => {
                selector = &**next_selector;
                simple_selectors_specificity(selector.simple_selectors.as_slice(), &mut specificity)
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
                | &AnyLink | &Link | &Visited | &Hover | &Disabled | &Enabled
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
    cmp::min(specificity.id_selectors, MAX_10BIT) << 20
    | cmp::min(specificity.class_like_selectors, MAX_10BIT) << 10
    | cmp::min(specificity.element_selectors, MAX_10BIT)
}


/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
///
/// `Err(())` means invalid selector
fn parse_simple_selectors<I: Iterator<ComponentValue>>(
                          iter: &mut Iter<I>, namespaces: &NamespaceMap)
                          -> Result<(Vec<SimpleSelector>, Option<PseudoElement>), ()> {
    let mut empty = true;
    let mut simple_selectors = match try!(parse_type_selector(iter, namespaces)) {
        None => vec![],
        Some(s) => { empty = false; s }
    };

    let mut pseudo_element = None;
    loop {
        match try!(parse_one_simple_selector(iter, namespaces, /* inside_negation = */ false)) {
            None => break,
            Some(SimpleSelectorResult(s)) => { simple_selectors.push(s); empty = false },
            Some(PseudoElementResult(p)) => { pseudo_element = Some(p); empty = false; break },
        }
    }
    if empty { Err(()) }  // An empty selector is invalid
    else { Ok((simple_selectors, pseudo_element)) }
}


/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a type selector, could be something else. `iter` was not consumed.
/// * `Ok(Some(vec))`: Length 0 (`*|*`), 1 (`*|E` or `ns|*`) or 2 (`|E` or `ns|E`)
fn parse_type_selector<I: Iterator<ComponentValue>>(
                       iter: &mut Iter<I>, namespaces: &NamespaceMap)
                       -> Result<Option<Vec<SimpleSelector>>, ()> {
    skip_whitespace(iter);
    match try!(parse_qualified_name(iter, /* in_attr_selector = */ false, namespaces)) {
        None => Ok(None),
        Some((namespace, local_name)) => {
            let mut simple_selectors = vec!();
            match namespace {
                SpecificNamespace(ns) => simple_selectors.push(NamespaceSelector(ns)),
                AnyNamespace => (),
            }
            match local_name {
                Some(name) => {
                    simple_selectors.push(LocalNameSelector(LocalName {
                        name: Atom::from_slice(name.as_slice()),
                        lower_name: Atom::from_slice(name.into_ascii_lower().as_slice())
                    }))
                }
                None => (),
            }
            Ok(Some(simple_selectors))
        }
    }
}


enum SimpleSelectorParseResult {
    SimpleSelectorResult(SimpleSelector),
    PseudoElementResult(PseudoElement),
}

/// Parse a simple selector other than a type selector.
///
/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `iter` was not consumed.
/// * `Ok(Some(_))`: Parsed a simple selector or pseudo-element
fn parse_one_simple_selector<I: Iterator<ComponentValue>>(
                             iter: &mut Iter<I>, namespaces: &NamespaceMap, inside_negation: bool)
                             -> Result<Option<SimpleSelectorParseResult>, ()> {
    match iter.peek() {
        Some(&IDHash(_)) => match iter.next() {
            Some(IDHash(id)) => Ok(Some(SimpleSelectorResult(
                IDSelector(Atom::from_slice(id.as_slice()))))),
            _ => fail!("Implementation error, this should not happen."),
        },
        Some(&Delim('.')) => {
            iter.next();
            match iter.next() {
                Some(Ident(class)) => Ok(Some(SimpleSelectorResult(
                    ClassSelector(Atom::from_slice(class.as_slice()))))),
                _ => Err(()),
            }
        }
        Some(&SquareBracketBlock(_)) => match iter.next() {
            Some(SquareBracketBlock(content))
            => Ok(Some(SimpleSelectorResult(try!(parse_attribute_selector(content, namespaces))))),
            _ => fail!("Implementation error, this should not happen."),
        },
        Some(&Colon) => {
            iter.next();
            match iter.next() {
                Some(Ident(name)) => match parse_simple_pseudo_class(name.as_slice()) {
                    Err(()) => {
                        match name.as_slice().to_ascii_lower().as_slice() {
                            // Supported CSS 2.1 pseudo-elements only.
                            // ** Do not add to this list! **
                            "before" => Ok(Some(PseudoElementResult(Before))),
                            "after" => Ok(Some(PseudoElementResult(After))),
//                            "first-line" => PseudoElementResult(FirstLine),
//                            "first-letter" => PseudoElementResult(FirstLetter),
                            _ => Err(())
                        }
                    },
                    Ok(result) => Ok(Some(SimpleSelectorResult(result))),
                },
                Some(Function(name, arguments))
                => Ok(Some(SimpleSelectorResult(try!(parse_functional_pseudo_class(
                        name, arguments, namespaces, inside_negation))))),
                Some(Colon) => {
                    match iter.next() {
                        Some(Ident(name))
                        => Ok(Some(PseudoElementResult(try!(parse_pseudo_element(name))))),
                        _ => Err(()),
                    }
                }
                _ => Err(()),
            }
        }
        _ => Ok(None),
    }
}


/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `iter` was not consumed.
/// * `Ok(Some((namespace, local_name)))`: `None` for the local name means a `*` universal selector
fn parse_qualified_name<I: Iterator<ComponentValue>>(
                        iter: &mut Iter<I>, in_attr_selector: bool, namespaces: &NamespaceMap)
                        -> Result<Option<(NamespaceConstraint, Option<String>)>, ()> {
    let default_namespace = |local_name| {
        let namespace = match namespaces.default {
            Some(ref ns) => SpecificNamespace(ns.clone()),
            None => AnyNamespace,
        };
        Ok(Some((namespace, local_name)))
    };

    let explicit_namespace = |iter: &mut Iter<I>, namespace| {
        assert!(iter.next() == Some(Delim('|')),
                "Implementation error, this should not happen.");
        match iter.peek() {
            Some(&Delim('*')) if !in_attr_selector => {
                iter.next();
                Ok(Some((namespace, None)))
            },
            Some(&Ident(_)) => {
                let local_name = get_next_ident(iter);
                Ok(Some((namespace, Some(local_name))))
            },
            _ => Err(()),
        }
    };

    match iter.peek() {
        Some(&Ident(_)) => {
            let value = get_next_ident(iter);
            match iter.peek() {
                Some(&Delim('|')) => {
                    let namespace = match namespaces.prefix_map.find(&value) {
                        None => return Err(()),  // Undeclared namespace prefix
                        Some(ref ns) => (*ns).clone(),
                    };
                    explicit_namespace(iter, SpecificNamespace(namespace))
                },
                _ if in_attr_selector => Ok(Some(
                    (SpecificNamespace(namespace::Null), Some(value)))),
                _ => default_namespace(Some(value)),
            }
        },
        Some(&Delim('*')) => {
            iter.next();  // Consume '*'
            match iter.peek() {
                Some(&Delim('|')) => explicit_namespace(iter, AnyNamespace),
                _ => {
                    if !in_attr_selector { default_namespace(None) }
                    else { Err(()) }
                },
            }
        },
        Some(&Delim('|')) => explicit_namespace(iter, SpecificNamespace(namespace::Null)),
        _ => Ok(None),
    }
}


fn parse_attribute_selector(content: Vec<ComponentValue>, namespaces: &NamespaceMap)
                            -> Result<SimpleSelector, ()> {
    let iter = &mut content.into_iter().peekable();
    let attr = match try!(parse_qualified_name(iter, /* in_attr_selector = */ true, namespaces)) {
        None => return Err(()),
        Some((_, None)) => fail!("Implementation error, this should not happen."),
        Some((namespace, Some(local_name))) => AttrSelector {
            namespace: namespace,
            lower_name: local_name.as_slice().to_ascii_lower(),
            name: local_name,
        },
    };
    skip_whitespace(iter);
    // TODO: deal with empty value or value containing whitespace (see spec)
    macro_rules! get_value( () => {{
        skip_whitespace(iter);
        match iter.next() {
            Some(Ident(value)) | Some(QuotedString(value)) => value,
            _ => return Err(())
        }
    }};)
    let result = match iter.next() {
        None => AttrExists(attr),  // [foo]
        Some(Delim('=')) => AttrEqual(attr, (get_value!())),  // [foo=bar]
        Some(IncludeMatch) => AttrIncludes(attr, (get_value!())),  // [foo~=bar]
        Some(DashMatch) => {
            let value = get_value!();
            let dashing_value = format!("{}-", value);
            AttrDashMatch(attr, value, dashing_value)  // [foo|=bar]
        },
        Some(PrefixMatch) => AttrPrefixMatch(attr, (get_value!())),  // [foo^=bar]
        Some(SubstringMatch) => AttrSubstringMatch(attr, (get_value!())),  // [foo*=bar]
        Some(SuffixMatch) => AttrSuffixMatch(attr, (get_value!())),  // [foo$=bar]
        _ => return Err(())
    };
    skip_whitespace(iter);
    if iter.next().is_none() { Ok(result) } else { Err(()) }
}


fn parse_simple_pseudo_class(name: &str) -> Result<SimpleSelector, ()> {
    match name.to_ascii_lower().as_slice() {
        "any-link" => Ok(AnyLink),
        "link" => Ok(Link),
        "visited" => Ok(Visited),
        "hover" => Ok(Hover),
        "disabled" => Ok(Disabled),
        "enabled" => Ok(Enabled),
        "first-child" => Ok(FirstChild),
        "last-child"  => Ok(LastChild),
        "only-child"  => Ok(OnlyChild),
        "root" => Ok(Root),
        "first-of-type" => Ok(FirstOfType),
        "last-of-type"  => Ok(LastOfType),
        "only-of-type"  => Ok(OnlyOfType),
//        "empty" => Ok(Empty),
        _ => Err(())
    }
}


fn parse_functional_pseudo_class(name: String, arguments: Vec<ComponentValue>,
                                 namespaces: &NamespaceMap, inside_negation: bool)
                                 -> Result<SimpleSelector, ()> {
    match name.as_slice().to_ascii_lower().as_slice() {
//        "lang" => parse_lang(arguments),
        "nth-child"        => parse_nth(arguments.as_slice()).map(|(a, b)| NthChild(a, b)),
        "nth-last-child"   => parse_nth(arguments.as_slice()).map(|(a, b)| NthLastChild(a, b)),
        "nth-of-type"      => parse_nth(arguments.as_slice()).map(|(a, b)| NthOfType(a, b)),
        "nth-last-of-type" => parse_nth(arguments.as_slice()).map(|(a, b)| NthLastOfType(a, b)),
        "not" => if inside_negation { Err(()) } else { parse_negation(arguments, namespaces) },
        _ => Err(())
    }
}


fn parse_pseudo_element(name: String) -> Result<PseudoElement, ()> {
    match name.as_slice().to_ascii_lower().as_slice() {
        // All supported pseudo-elements
        "before" => Ok(Before),
        "after" => Ok(After),
//        "first-line" => Some(FirstLine),
//        "first-letter" => Some(FirstLetter),
        _ => Err(())
    }
}


//fn parse_lang(arguments: vec!(ComponentValue)) -> Result<SimpleSelector, ()> {
//    let mut iter = arguments.move_skip_whitespace();
//    match iter.next() {
//        Some(Ident(value)) => {
//            if "" == value || iter.next().is_some() { None }
//            else { Ok(Lang(value)) }
//        },
//        _ => Err(()),
//    }
//}


/// Level 3: Parse **one** simple_selector
fn parse_negation(arguments: Vec<ComponentValue>, namespaces: &NamespaceMap)
                  -> Result<SimpleSelector, ()> {
    let iter = &mut arguments.into_iter().peekable();
    match try!(parse_type_selector(iter, namespaces)) {
        Some(type_selector) => Ok(Negation(type_selector)),
        None => {
            match try!(parse_one_simple_selector(iter, namespaces, /* inside_negation = */ true)) {
                Some(SimpleSelectorResult(simple_selector)) => Ok(Negation(vec![simple_selector])),
                _ => Err(())
            }
        },
    }
}


/// Assuming the next token is an ident, consume it and return its value
#[inline]
fn get_next_ident<I: Iterator<ComponentValue>>(iter: &mut Iter<I>) -> String {
    match iter.next() {
        Some(Ident(value)) => value,
        _ => fail!("Implementation error, this should not happen."),
    }
}


#[inline]
fn skip_whitespace<I: Iterator<ComponentValue>>(iter: &mut Iter<I>) -> bool {
    let mut any_whitespace = false;
    loop {
        if iter.peek() != Some(&WhiteSpace) { return any_whitespace }
        any_whitespace = true;
        iter.next();
    }
}


#[cfg(test)]
mod tests {
    use sync::Arc;
    use cssparser;
    use servo_util::atom::Atom;
    use servo_util::namespace;
    use namespaces::NamespaceMap;
    use super::*;

    fn parse(input: &str) -> Result<Vec<Selector>, ()> {
        parse_ns(input, &NamespaceMap::new())
    }

    fn parse_ns(input: &str, namespaces: &NamespaceMap) -> Result<Vec<Selector>, ()> {
        parse_selector_list(cssparser::tokenize(input).map(|(v, _)| v), namespaces)
    }

    fn specificity(a: u32, b: u32, c: u32) -> u32 {
        a << 20 | b << 10 | c
    }

    #[test]
    fn test_parsing() {
        assert!(parse("") == Err(()))
        assert!(parse("EeÉ") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(LocalNameSelector(LocalName {
                    name: Atom::from_slice("EeÉ"),
                    lower_name: Atom::from_slice("eeÉ") })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        })))
        assert!(parse(".foo") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(ClassSelector(Atom::from_slice("foo"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        })))
        assert!(parse("#bar") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(IDSelector(Atom::from_slice("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        })))
        assert!(parse("e.foo#bar") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(LocalNameSelector(LocalName {
                                            name: Atom::from_slice("e"),
                                            lower_name: Atom::from_slice("e") }),
                                       ClassSelector(Atom::from_slice("foo")),
                                       IDSelector(Atom::from_slice("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        })))
        assert!(parse("e.foo #bar") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(IDSelector(Atom::from_slice("bar"))),
                next: Some((box CompoundSelector {
                    simple_selectors: vec!(LocalNameSelector(LocalName {
                                                name: Atom::from_slice("e"),
                                                lower_name: Atom::from_slice("e") }),
                                           ClassSelector(Atom::from_slice("foo"))),
                    next: None,
                }, Descendant)),
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        })))
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let mut namespaces = NamespaceMap::new();
        assert!(parse_ns("[Foo]", &namespaces) == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(AttrExists(AttrSelector {
                    name: Atom::from_slice("Foo"),
                    lower_name: Atom::from_slice("foo"),
                    namespace: SpecificNamespace(namespace::Null),
                })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        })))
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        namespaces.default = Some(namespace::MathML);
        assert!(parse_ns("[Foo]", &namespaces) == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(AttrExists(AttrSelector {
                    name: Atom::from_slice("Foo"),
                    lower_name: Atom::from_slice("foo"),
                    namespace: SpecificNamespace(namespace::Null),
                })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        })))
        // Default namespace does apply to type selectors
        assert!(parse_ns("e", &namespaces) == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(
                    NamespaceSelector(namespace::MathML),
                    LocalNameSelector(LocalName {
                        name: Atom::from_slice("e"),
                        lower_name: Atom::from_slice("e") }),
                ),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        })))
        // https://github.com/mozilla/servo/issues/1723
        assert!(parse("::before") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(),
                next: None,
            }),
            pseudo_element: Some(Before),
            specificity: specificity(0, 0, 1),
        })))
        assert!(parse("div :after") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(),
                next: Some((box CompoundSelector {
                    simple_selectors: vec!(LocalNameSelector(LocalName {
                        name: Atom::from_slice("div"),
                        lower_name: Atom::from_slice("div") })),
                    next: None,
                }, Descendant)),
            }),
            pseudo_element: Some(After),
            specificity: specificity(0, 0, 2),
        })))
    }
}
