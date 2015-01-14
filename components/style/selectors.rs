/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{cmp, iter};
use std::ascii::{AsciiExt, OwnedAsciiExt};
use std::sync::Arc;

use cssparser::ast::*;
use cssparser::ast::ComponentValue::*;
use cssparser::{tokenize, parse_nth};

use selector_matching::StylesheetOrigin;
use string_cache::{Atom, Namespace};

use namespaces::NamespaceMap;

/// Ambient data used by the parser.
#[deriving(Copy)]
pub struct ParserContext {
    /// The origin of this stylesheet.
    pub origin: StylesheetOrigin,
}

#[deriving(PartialEq, Clone)]
pub struct Selector {
    pub compound_selectors: Arc<CompoundSelector>,
    pub pseudo_element: Option<PseudoElement>,
    pub specificity: u32,
}

#[deriving(Eq, PartialEq, Clone, Hash, Copy)]
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

#[deriving(PartialEq, Clone, Copy)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum SimpleSelector {
    ID(Atom),
    Class(Atom),
    LocalName(LocalName),
    Namespace(Namespace),

    // Attribute selectors
    AttrExists(AttrSelector),  // [foo]
    AttrEqual(AttrSelector, String, CaseSensitivity),  // [foo=bar]
    AttrIncludes(AttrSelector, String),  // [foo~=bar]
    AttrDashMatch(AttrSelector, String, String), // [foo|=bar]  Second string is the first + "-"
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
    Checked,
    Indeterminate,
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
    OnlyOfType,
    ServoNonzeroBorder,
    // ...
}


#[deriving(Eq, PartialEq, Clone, Hash, Copy)]
pub enum CaseSensitivity {
    CaseSensitive,  // Selectors spec says language-defined, but HTML says sensitive.
    CaseInsensitive,
}


#[deriving(Eq, PartialEq, Clone, Hash)]
pub struct LocalName {
    pub name: Atom,
    pub lower_name: Atom,
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub struct AttrSelector {
    pub name: Atom,
    pub lower_name: Atom,
    pub namespace: NamespaceConstraint,
}

#[deriving(Eq, PartialEq, Clone, Hash)]
pub enum NamespaceConstraint {
    Any,
    Specific(Namespace),
}


/// Re-exported to script, but opaque.
pub struct SelectorList {
    selectors: Vec<Selector>
}

/// Public to the style crate, but not re-exported to script
pub fn get_selector_list_selectors<'a>(selector_list: &'a SelectorList) -> &'a [Selector] {
    selector_list.selectors.as_slice()
}


type Iter<I> = iter::Peekable<ComponentValue, I>;


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
                &SimpleSelector::LocalName(..) =>
                    specificity.element_selectors += 1,
                &SimpleSelector::ID(..) =>
                    specificity.id_selectors += 1,
                &SimpleSelector::Class(..) |
                &SimpleSelector::AttrExists(..) |
                &SimpleSelector::AttrEqual(..) |
                &SimpleSelector::AttrIncludes(..) |
                &SimpleSelector::AttrDashMatch(..) |
                &SimpleSelector::AttrPrefixMatch(..) |
                &SimpleSelector::AttrSubstringMatch(..) |
                &SimpleSelector::AttrSuffixMatch(..) |
                &SimpleSelector::AnyLink | &SimpleSelector::Link |
                &SimpleSelector::Visited | &SimpleSelector::Hover |
                &SimpleSelector::Disabled | &SimpleSelector::Enabled |
                &SimpleSelector::FirstChild | &SimpleSelector::LastChild |
                &SimpleSelector::OnlyChild | &SimpleSelector::Root |
                &SimpleSelector::Checked |
                &SimpleSelector::Indeterminate |
//                &SimpleSelector::Empty | &SimpleSelector::Lang(*) |
                &SimpleSelector::NthChild(..) |
                &SimpleSelector::NthLastChild(..) |
                &SimpleSelector::NthOfType(..) |
                &SimpleSelector::NthLastOfType(..) |
                &SimpleSelector::FirstOfType | &SimpleSelector::LastOfType |
                &SimpleSelector::OnlyOfType |
                &SimpleSelector::ServoNonzeroBorder =>
                    specificity.class_like_selectors += 1,
                &SimpleSelector::Namespace(..) => (),
                &SimpleSelector::Negation(ref negated) =>
                    simple_selectors_specificity(negated.as_slice(), specificity),
            }
        }
    }

    static MAX_10BIT: u32 = (1u32 << 10) - 1;
    cmp::min(specificity.id_selectors, MAX_10BIT) << 20
    | cmp::min(specificity.class_like_selectors, MAX_10BIT) << 10
    | cmp::min(specificity.element_selectors, MAX_10BIT)
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
                NamespaceConstraint::Specific(ns) => {
                    simple_selectors.push(SimpleSelector::Namespace(ns))
                },
                NamespaceConstraint::Any => (),
            }
            match local_name {
                Some(name) => {
                    simple_selectors.push(SimpleSelector::LocalName(LocalName {
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
    SimpleSelector(SimpleSelector),
    PseudoElement(PseudoElement),
}


/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `iter` was not consumed.
/// * `Ok(Some((namespace, local_name)))`: `None` for the local name means a `*` universal selector
fn parse_qualified_name<I: Iterator<ComponentValue>>(
                        iter: &mut Iter<I>, in_attr_selector: bool, namespaces: &NamespaceMap)
                        -> Result<Option<(NamespaceConstraint, Option<String>)>, ()> {
    let default_namespace = |local_name| {
        let namespace = match namespaces.default {
            Some(ref ns) => NamespaceConstraint::Specific(ns.clone()),
            None => NamespaceConstraint::Any,
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
                    let namespace = match namespaces.prefix_map.get(&value) {
                        None => return Err(()),  // Undeclared namespace prefix
                        Some(ref ns) => (*ns).clone(),
                    };
                    explicit_namespace(iter, NamespaceConstraint::Specific(namespace))
                },
                _ if in_attr_selector => Ok(Some(
                    (NamespaceConstraint::Specific(ns!("")), Some(value)))),
                _ => default_namespace(Some(value)),
            }
        },
        Some(&Delim('*')) => {
            iter.next();  // Consume '*'
            match iter.peek() {
                Some(&Delim('|')) => explicit_namespace(iter, NamespaceConstraint::Any),
                _ => {
                    if !in_attr_selector { default_namespace(None) }
                    else { Err(()) }
                },
            }
        },
        Some(&Delim('|')) => explicit_namespace(iter, NamespaceConstraint::Specific(ns!(""))),
        _ => Ok(None),
    }
}


fn parse_attribute_selector(content: Vec<ComponentValue>, namespaces: &NamespaceMap)
                            -> Result<SimpleSelector, ()> {
    let iter = &mut content.into_iter().peekable();
    let attr = match try!(parse_qualified_name(iter, /* in_attr_selector = */ true, namespaces)) {
        None => return Err(()),
        Some((_, None)) => panic!("Implementation error, this should not happen."),
        Some((namespace, Some(local_name))) => AttrSelector {
            namespace: namespace,
            lower_name: Atom::from_slice(local_name.as_slice().to_ascii_lower().as_slice()),
            name: Atom::from_slice(local_name.as_slice()),
        },
    };
    skip_whitespace(iter);
    // TODO: deal with empty value or value containing whitespace (see spec)
    let result = match iter.next() {
        // [foo]
        None => SimpleSelector::AttrExists(attr),

        // [foo=bar]
        Some(Delim('=')) =>
            SimpleSelector::AttrEqual(attr, try!(parse_attribute_value(iter)),
                                      try!(parse_attribute_flags(iter))),

        // [foo~=bar]
        Some(IncludeMatch) =>
            SimpleSelector::AttrIncludes(attr, try!(parse_attribute_value(iter))),

        // [foo|=bar]
        Some(DashMatch) => {
            let value = try!(parse_attribute_value(iter));
            let dashing_value = format!("{}-", value);
            SimpleSelector::AttrDashMatch(attr, value, dashing_value)
        },

        // [foo^=bar]
        Some(PrefixMatch) =>
            SimpleSelector::AttrPrefixMatch(attr, try!(parse_attribute_value(iter))),

        // [foo*=bar]
        Some(SubstringMatch) =>
            SimpleSelector::AttrSubstringMatch(attr, try!(parse_attribute_value(iter))),

        // [foo$=bar]
        Some(SuffixMatch) =>
            SimpleSelector::AttrSuffixMatch(attr, try!(parse_attribute_value(iter))),

        _ => return Err(())
    };
    skip_whitespace(iter);
    if iter.next().is_none() { Ok(result) } else { Err(()) }
}


fn parse_attribute_value<I: Iterator<ComponentValue>>(iter: &mut Iter<I>) -> Result<String, ()> {
    skip_whitespace(iter);
    match iter.next() {
        Some(Ident(value)) | Some(QuotedString(value)) => Ok(value),
        _ => Err(())
    }
}


fn parse_attribute_flags<I: Iterator<ComponentValue>>(iter: &mut Iter<I>)
                         -> Result<CaseSensitivity, ()> {
    skip_whitespace(iter);
    match iter.next() {
        None => Ok(CaseSensitivity::CaseSensitive),
        Some(Ident(ref value)) if value.as_slice().eq_ignore_ascii_case("i")
        => Ok(CaseSensitivity::CaseInsensitive),
        _ => Err(())
    }
}

pub fn parse_selector_list_from_str(context: &ParserContext, input: &str)
                                    -> Result<SelectorList,()> {
    let namespaces = NamespaceMap::new();
    let iter = tokenize(input).map(|(token, _)| token);
    parse_selector_list(context, iter, &namespaces).map(|s| SelectorList { selectors: s })
}

/// Parse a comma-separated list of Selectors.
/// aka Selector Group in http://www.w3.org/TR/css3-selectors/#grouping
///
/// Return the Selectors or None if there is an invalid selector.
pub fn parse_selector_list<I>(context: &ParserContext, iter: I, namespaces: &NamespaceMap)
                              -> Result<Vec<Selector>,()>
                              where I: Iterator<ComponentValue> {
    let iter = &mut iter.peekable();
    let mut results = vec![try!(parse_selector(context, iter, namespaces))];

    loop {
        skip_whitespace(iter);
        match iter.peek() {
            None => break,  // EOF
            Some(&Comma) => {
                iter.next();
            }
            _ => return Err(()),
        }
        results.push(try!(parse_selector(context, iter, namespaces)));
    }
    Ok(results)
}
/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// `Err` means invalid selector.
fn parse_selector<I>(context: &ParserContext, iter: &mut Iter<I>, namespaces: &NamespaceMap)
                     -> Result<Selector,()>
                     where I: Iterator<ComponentValue> {
    let (first, mut pseudo_element) = try!(parse_simple_selectors(context, iter, namespaces));
    let mut compound = CompoundSelector{ simple_selectors: first, next: None };

    while pseudo_element.is_none() {
        let any_whitespace = skip_whitespace(iter);
        let combinator = match iter.peek() {
            None => break,  // EOF
            Some(&Comma) => break,
            Some(&Delim('>')) => { iter.next(); Combinator::Child },
            Some(&Delim('+')) => { iter.next(); Combinator::NextSibling },
            Some(&Delim('~')) => { iter.next(); Combinator::LaterSibling },
            Some(_) => {
                if any_whitespace { Combinator::Descendant }
                else { return Err(()) }
            }
        };
        let (simple_selectors, pseudo) = try!(parse_simple_selectors(context, iter, namespaces));
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

/// Level 3: Parse **one** simple_selector
fn parse_negation(context: &ParserContext,
                  arguments: Vec<ComponentValue>,
                  namespaces: &NamespaceMap)
                  -> Result<SimpleSelector,()> {
    let iter = &mut arguments.into_iter().peekable();
    match try!(parse_type_selector(iter, namespaces)) {
        Some(type_selector) => Ok(SimpleSelector::Negation(type_selector)),
        None => {
            match try!(parse_one_simple_selector(context,
                                                 iter,
                                                 namespaces,
                                                 /* inside_negation = */ true)) {
                Some(SimpleSelectorParseResult::SimpleSelector(simple_selector)) => {
                    Ok(SimpleSelector::Negation(vec![simple_selector]))
                }
                _ => Err(())
            }
        },
    }
}

/// simple_selector_sequence
/// : [ type_selector | universal ] [ HASH | class | attrib | pseudo | negation ]*
/// | [ HASH | class | attrib | pseudo | negation ]+
///
/// `Err(())` means invalid selector
fn parse_simple_selectors<I>(context: &ParserContext,
                             iter: &mut Iter<I>,
                             namespaces: &NamespaceMap)
                             -> Result<(Vec<SimpleSelector>, Option<PseudoElement>),()>
                             where I: Iterator<ComponentValue> {
    let mut empty = true;
    let mut simple_selectors = match try!(parse_type_selector(iter, namespaces)) {
        None => vec![],
        Some(s) => { empty = false; s }
    };

    let mut pseudo_element = None;
    loop {
        match try!(parse_one_simple_selector(context,
                                             iter,
                                             namespaces,
                                             /* inside_negation = */ false)) {
            None => break,
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => { simple_selectors.push(s); empty = false },
            Some(SimpleSelectorParseResult::PseudoElement(p)) => { pseudo_element = Some(p); empty = false; break },
        }
    }
    if empty {
        // An empty selector is invalid.
        Err(())
    } else {
        Ok((simple_selectors, pseudo_element))
    }
}

fn parse_functional_pseudo_class(context: &ParserContext,
                                 name: String,
                                 arguments: Vec<ComponentValue>,
                                 namespaces: &NamespaceMap,
                                 inside_negation: bool)
                                 -> Result<SimpleSelector,()> {
    match name.as_slice().to_ascii_lower().as_slice() {
//        "lang" => parse_lang(arguments),
        "nth-child"        => parse_nth(arguments.as_slice()).map(|(a, b)| SimpleSelector::NthChild(a, b)),
        "nth-last-child"   => parse_nth(arguments.as_slice()).map(|(a, b)| SimpleSelector::NthLastChild(a, b)),
        "nth-of-type"      => parse_nth(arguments.as_slice()).map(|(a, b)| SimpleSelector::NthOfType(a, b)),
        "nth-last-of-type" => parse_nth(arguments.as_slice()).map(|(a, b)| SimpleSelector::NthLastOfType(a, b)),
        "not" => {
            if inside_negation {
                Err(())
            } else {
                parse_negation(context, arguments, namespaces)
            }
        }
        _ => Err(())
    }
}

/// Parse a simple selector other than a type selector.
///
/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `iter` was not consumed.
/// * `Ok(Some(_))`: Parsed a simple selector or pseudo-element
fn parse_one_simple_selector<I>(context: &ParserContext,
                                iter: &mut Iter<I>,
                                namespaces: &NamespaceMap,
                                inside_negation: bool)
                                -> Result<Option<SimpleSelectorParseResult>,()>
                                where I: Iterator<ComponentValue> {
    match iter.peek() {
        Some(&IDHash(_)) => match iter.next() {
            Some(IDHash(id)) => Ok(Some(SimpleSelectorParseResult::SimpleSelector(
                SimpleSelector::ID(Atom::from_slice(id.as_slice()))))),
            _ => panic!("Implementation error, this should not happen."),
        },
        Some(&Delim('.')) => {
            iter.next();
            match iter.next() {
                Some(Ident(class)) => Ok(Some(SimpleSelectorParseResult::SimpleSelector(
                    SimpleSelector::Class(Atom::from_slice(class.as_slice()))))),
                _ => Err(()),
            }
        }
        Some(&SquareBracketBlock(_)) => match iter.next() {
            Some(SquareBracketBlock(content))
            => Ok(Some(SimpleSelectorParseResult::SimpleSelector(try!(parse_attribute_selector(content, namespaces))))),
            _ => panic!("Implementation error, this should not happen."),
        },
        Some(&Colon) => {
            iter.next();
            match iter.next() {
                Some(Ident(name)) => match parse_simple_pseudo_class(context, name.as_slice()) {
                    Err(()) => {
                        match name.as_slice().to_ascii_lower().as_slice() {
                            // Supported CSS 2.1 pseudo-elements only.
                            // ** Do not add to this list! **
                            "before" => Ok(Some(SimpleSelectorParseResult::PseudoElement(PseudoElement::Before))),
                            "after" => Ok(Some(SimpleSelectorParseResult::PseudoElement(PseudoElement::After))),
//                            "first-line" => SimpleSelectorParseResult::PseudoElement(FirstLine),
//                            "first-letter" => SimpleSelectorParseResult::PseudoElement(FirstLetter),
                            _ => Err(())
                        }
                    },
                    Ok(result) => Ok(Some(SimpleSelectorParseResult::SimpleSelector(result))),
                },
                Some(Function(name, arguments))
                => {
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(try!(parse_functional_pseudo_class(
                                        context,
                                        name,
                                        arguments,
                                        namespaces,
                                        inside_negation)))))
                }
                Some(Colon) => {
                    match iter.next() {
                        Some(Ident(name))
                        => Ok(Some(SimpleSelectorParseResult::PseudoElement(try!(parse_pseudo_element(name))))),
                        _ => Err(()),
                    }
                }
                _ => Err(()),
            }
        }
        _ => Ok(None),
    }
}

fn parse_simple_pseudo_class(context: &ParserContext, name: &str) -> Result<SimpleSelector,()> {
    match name.to_ascii_lower().as_slice() {
        "any-link" => Ok(SimpleSelector::AnyLink),
        "link" => Ok(SimpleSelector::Link),
        "visited" => Ok(SimpleSelector::Visited),
        "hover" => Ok(SimpleSelector::Hover),
        "disabled" => Ok(SimpleSelector::Disabled),
        "enabled" => Ok(SimpleSelector::Enabled),
        "checked" => Ok(SimpleSelector::Checked),
        "indeterminate" => Ok(SimpleSelector::Indeterminate),
        "first-child" => Ok(SimpleSelector::FirstChild),
        "last-child"  => Ok(SimpleSelector::LastChild),
        "only-child"  => Ok(SimpleSelector::OnlyChild),
        "root" => Ok(SimpleSelector::Root),
        "first-of-type" => Ok(SimpleSelector::FirstOfType),
        "last-of-type"  => Ok(SimpleSelector::LastOfType),
        "only-of-type"  => Ok(SimpleSelector::OnlyOfType),
        "-servo-nonzero-border" if context.origin == StylesheetOrigin::UserAgent => Ok(SimpleSelector::ServoNonzeroBorder),
//        "empty" => Ok(Empty),
        _ => Err(())
    }
}

fn parse_pseudo_element(name: String) -> Result<PseudoElement, ()> {
    match name.as_slice().to_ascii_lower().as_slice() {
        // All supported pseudo-elements
        "before" => Ok(PseudoElement::Before),
        "after" => Ok(PseudoElement::After),
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



/// Assuming the next token is an ident, consume it and return its value
#[inline]
fn get_next_ident<I: Iterator<ComponentValue>>(iter: &mut Iter<I>) -> String {
    match iter.next() {
        Some(Ident(value)) => value,
        _ => panic!("Implementation error, this should not happen."),
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
    use std::sync::Arc;
    use cssparser;
    use namespaces::NamespaceMap;
    use selector_matching::StylesheetOrigin;
    use string_cache::Atom;
    use super::*;

    fn parse(input: &str) -> Result<Vec<Selector>, ()> {
        parse_ns(input, &NamespaceMap::new())
    }

    fn parse_ns(input: &str, namespaces: &NamespaceMap) -> Result<Vec<Selector>, ()> {
        let context = ParserContext {
            origin: StylesheetOrigin::Author,
        };
        parse_selector_list(&context, cssparser::tokenize(input).map(|(v, _)| v), namespaces)
    }

    fn specificity(a: u32, b: u32, c: u32) -> u32 {
        a << 20 | b << 10 | c
    }

    #[test]
    fn test_parsing() {
        assert!(parse("") == Err(()))
        assert!(parse("EeÉ") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::LocalName(LocalName {
                    name: Atom::from_slice("EeÉ"),
                    lower_name: Atom::from_slice("eeÉ") })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        })))
        assert!(parse(".foo") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::Class(Atom::from_slice("foo"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        })))
        assert!(parse("#bar") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::ID(Atom::from_slice("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        })))
        assert!(parse("e.foo#bar") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::LocalName(LocalName {
                                            name: Atom::from_slice("e"),
                                            lower_name: Atom::from_slice("e") }),
                                       SimpleSelector::Class(Atom::from_slice("foo")),
                                       SimpleSelector::ID(Atom::from_slice("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        })))
        assert!(parse("e.foo #bar") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::ID(Atom::from_slice("bar"))),
                next: Some((box CompoundSelector {
                    simple_selectors: vec!(SimpleSelector::LocalName(LocalName {
                                                name: Atom::from_slice("e"),
                                                lower_name: Atom::from_slice("e") }),
                                           SimpleSelector::Class(Atom::from_slice("foo"))),
                    next: None,
                }, Combinator::Descendant)),
            }),
            pseudo_element: None,
            specificity: specificity(1, 1, 1),
        })))
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let mut namespaces = NamespaceMap::new();
        assert!(parse_ns("[Foo]", &namespaces) == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::AttrExists(AttrSelector {
                    name: Atom::from_slice("Foo"),
                    lower_name: Atom::from_slice("foo"),
                    namespace: NamespaceConstraint::Specific(ns!("")),
                })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        })))
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        namespaces.default = Some(ns!(MathML));
        assert!(parse_ns("[Foo]", &namespaces) == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::AttrExists(AttrSelector {
                    name: Atom::from_slice("Foo"),
                    lower_name: Atom::from_slice("foo"),
                    namespace: NamespaceConstraint::Specific(ns!("")),
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
                    SimpleSelector::Namespace(ns!(MathML)),
                    SimpleSelector::LocalName(LocalName {
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
            pseudo_element: Some(PseudoElement::Before),
            specificity: specificity(0, 0, 1),
        })))
        assert!(parse("div :after") == Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(),
                next: Some((box CompoundSelector {
                    simple_selectors: vec!(SimpleSelector::LocalName(LocalName {
                        name: atom!("div"),
                        lower_name: atom!("div") })),
                    next: None,
                }, Combinator::Descendant)),
            }),
            pseudo_element: Some(PseudoElement::After),
            specificity: specificity(0, 0, 2),
        })))
    }
}
