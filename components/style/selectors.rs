/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cmp;
use std::ascii::{AsciiExt, OwnedAsciiExt};
use std::sync::Arc;
use std::string::CowString;

use cssparser::{Token, Parser, parse_nth};
use string_cache::{Atom, Namespace};
use url::Url;

use parser::ParserContext;
use stylesheets::Origin;


#[derive(PartialEq, Clone, Debug)]
pub struct Selector {
    pub compound_selectors: Arc<CompoundSelector>,
    pub pseudo_element: Option<PseudoElement>,
    pub specificity: u32,
}

#[derive(Eq, PartialEq, Clone, Hash, Copy, Debug)]
pub enum PseudoElement {
    Before,
    After,
    // ...
}


#[derive(PartialEq, Clone, Debug)]
pub struct CompoundSelector {
    pub simple_selectors: Vec<SimpleSelector>,
    pub next: Option<(Box<CompoundSelector>, Combinator)>,  // c.next is left of c
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
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
    Root,
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


#[derive(Eq, PartialEq, Clone, Hash, Copy, Debug)]
pub enum CaseSensitivity {
    CaseSensitive,  // Selectors spec says language-defined, but HTML says sensitive.
    CaseInsensitive,
}


#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct LocalName {
    pub name: Atom,
    pub lower_name: Atom,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct AttrSelector {
    pub name: Atom,
    pub lower_name: Atom,
    pub namespace: NamespaceConstraint,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum NamespaceConstraint {
    Any,
    Specific(Namespace),
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

    simple_selectors_specificity(&selector.simple_selectors, &mut specificity);
    loop {
        match selector.next {
            None => break,
            Some((ref next_selector, _)) => {
                selector = &**next_selector;
                simple_selectors_specificity(&selector.simple_selectors, &mut specificity)
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
                    simple_selectors_specificity(negated, specificity),
            }
        }
    }

    static MAX_10BIT: u32 = (1u32 << 10) - 1;
    cmp::min(specificity.id_selectors, MAX_10BIT) << 20
    | cmp::min(specificity.class_like_selectors, MAX_10BIT) << 10
    | cmp::min(specificity.element_selectors, MAX_10BIT)
}



pub fn parse_author_origin_selector_list_from_str(input: &str) -> Result<Vec<Selector>, ()> {
    let url = Url::parse("about:blank").unwrap();
    let context = ParserContext::new(Origin::Author, &url);
    parse_selector_list(&context, &mut Parser::new(input))
}

/// Parse a comma-separated list of Selectors.
/// aka Selector Group in http://www.w3.org/TR/css3-selectors/#grouping
///
/// Return the Selectors or None if there is an invalid selector.
pub fn parse_selector_list(context: &ParserContext, input: &mut Parser)
                           -> Result<Vec<Selector>,()> {
    input.parse_comma_separated(|input| parse_selector(context, input))
}


/// Build up a Selector.
/// selector : simple_selector_sequence [ combinator simple_selector_sequence ]* ;
///
/// `Err` means invalid selector.
fn parse_selector(context: &ParserContext, input: &mut Parser) -> Result<Selector,()> {
    let (first, mut pseudo_element) = try!(parse_simple_selectors(context, input));
    let mut compound = CompoundSelector{ simple_selectors: first, next: None };

    'outer_loop: while pseudo_element.is_none() {
        let combinator;
        let mut any_whitespace = false;
        loop {
            let position = input.position();
            match input.next_including_whitespace() {
                Err(()) => break 'outer_loop,
                Ok(Token::WhiteSpace(_)) => any_whitespace = true,
                Ok(Token::Delim('>')) => {
                    combinator = Combinator::Child;
                    break
                }
                Ok(Token::Delim('+')) => {
                    combinator = Combinator::NextSibling;
                    break
                }
                Ok(Token::Delim('~')) => {
                    combinator = Combinator::LaterSibling;
                    break
                }
                Ok(_) => {
                    input.reset(position);
                    if any_whitespace {
                        combinator = Combinator::Descendant;
                        break
                    } else {
                        break 'outer_loop
                    }
                }
            }
        }
        let (simple_selectors, pseudo) = try!(parse_simple_selectors(context, input));
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


/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a type selector, could be something else. `input` was not consumed.
/// * `Ok(Some(vec))`: Length 0 (`*|*`), 1 (`*|E` or `ns|*`) or 2 (`|E` or `ns|E`)
fn parse_type_selector(context: &ParserContext, input: &mut Parser)
                       -> Result<Option<Vec<SimpleSelector>>, ()> {
    match try!(parse_qualified_name(context, input, /* in_attr_selector = */ false)) {
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
                        name: Atom::from_slice(&name),
                        lower_name: Atom::from_slice(&name.into_owned().into_ascii_lowercase())
                    }))
                }
                None => (),
            }
            Ok(Some(simple_selectors))
        }
    }
}


#[derive(Debug)]
enum SimpleSelectorParseResult {
    SimpleSelector(SimpleSelector),
    PseudoElement(PseudoElement),
}


/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some((namespace, local_name)))`: `None` for the local name means a `*` universal selector
fn parse_qualified_name<'i, 't>
                       (context: &ParserContext, input: &mut Parser<'i, 't>,
                        in_attr_selector: bool)
                        -> Result<Option<(NamespaceConstraint, Option<CowString<'i>>)>, ()> {
    let default_namespace = |local_name| {
        let namespace = match context.namespaces.default {
            Some(ref ns) => NamespaceConstraint::Specific(ns.clone()),
            None => NamespaceConstraint::Any,
        };
        Ok(Some((namespace, local_name)))
    };

    let explicit_namespace = |input: &mut Parser<'i, 't>, namespace| {
        match input.next_including_whitespace() {
            Ok(Token::Delim('*')) if !in_attr_selector => {
                Ok(Some((namespace, None)))
            },
            Ok(Token::Ident(local_name)) => {
                Ok(Some((namespace, Some(local_name))))
            },
            _ => Err(()),
        }
    };

    let position = input.position();
    match input.next_including_whitespace() {
        Ok(Token::Ident(value)) => {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => {
                    let result = context.namespaces.prefix_map.get(&*value);
                    let namespace = try!(result.ok_or(()));
                    explicit_namespace(input, NamespaceConstraint::Specific(namespace.clone()))
                },
                _ => {
                    input.reset(position);
                    if in_attr_selector {
                        Ok(Some((NamespaceConstraint::Specific(ns!("")), Some(value))))
                    } else {
                        default_namespace(Some(value))
                    }
                }
            }
        },
        Ok(Token::Delim('*')) => {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::Delim('|')) => explicit_namespace(input, NamespaceConstraint::Any),
                _ => {
                    input.reset(position);
                    if in_attr_selector {
                        Err(())
                    } else {
                        default_namespace(None)
                    }
                },
            }
        },
        Ok(Token::Delim('|')) => explicit_namespace(input, NamespaceConstraint::Specific(ns!(""))),
        _ => {
            input.reset(position);
            Ok(None)
        }
    }
}


fn parse_attribute_selector(context: &ParserContext, input: &mut Parser)
                            -> Result<SimpleSelector, ()> {
    let attr = match try!(parse_qualified_name(context, input, /* in_attr_selector = */ true)) {
        None => return Err(()),
        Some((_, None)) => unreachable!(),
        Some((namespace, Some(local_name))) => AttrSelector {
            namespace: namespace,
            lower_name: Atom::from_slice(&local_name.to_ascii_lowercase()),
            name: Atom::from_slice(&local_name),
        },
    };

    fn parse_value(input: &mut Parser) -> Result<String, ()> {
        Ok((try!(input.expect_ident_or_string())).into_owned())
    }
    // TODO: deal with empty value or value containing whitespace (see spec)
    match input.next() {
        // [foo]
        Err(()) => Ok(SimpleSelector::AttrExists(attr)),

        // [foo=bar]
        Ok(Token::Delim('=')) => {
            Ok(SimpleSelector::AttrEqual(attr, try!(parse_value(input)),
                                         try!(parse_attribute_flags(input))))
        }
        // [foo~=bar]
        Ok(Token::IncludeMatch) => {
            Ok(SimpleSelector::AttrIncludes(attr, try!(parse_value(input))))
        }
        // [foo|=bar]
        Ok(Token::DashMatch) => {
            let value = try!(parse_value(input));
            let dashing_value = format!("{:?}-", value);
            Ok(SimpleSelector::AttrDashMatch(attr, value, dashing_value))
        }
        // [foo^=bar]
        Ok(Token::PrefixMatch) => {
            Ok(SimpleSelector::AttrPrefixMatch(attr, try!(parse_value(input))))
        }
        // [foo*=bar]
        Ok(Token::SubstringMatch) => {
            Ok(SimpleSelector::AttrSubstringMatch(attr, try!(parse_value(input))))
        }
        // [foo$=bar]
        Ok(Token::SuffixMatch) => {
            Ok(SimpleSelector::AttrSuffixMatch(attr, try!(parse_value(input))))
        }
        _ => Err(())
    }
}


fn parse_attribute_flags(input: &mut Parser) -> Result<CaseSensitivity, ()> {
    match input.next() {
        Err(()) => Ok(CaseSensitivity::CaseSensitive),
        Ok(Token::Ident(ref value)) if value.eq_ignore_ascii_case("i") => {
            Ok(CaseSensitivity::CaseInsensitive)
        }
        _ => Err(())
    }
}


/// Level 3: Parse **one** simple_selector
fn parse_negation(context: &ParserContext, input: &mut Parser) -> Result<SimpleSelector,()> {
    match try!(parse_type_selector(context, input)) {
        Some(type_selector) => Ok(SimpleSelector::Negation(type_selector)),
        None => {
            match try!(parse_one_simple_selector(context,
                                                 input,
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
fn parse_simple_selectors(context: &ParserContext, input: &mut Parser)
                          -> Result<(Vec<SimpleSelector>, Option<PseudoElement>),()> {
    // Consume any leading whitespace.
    loop {
        let position = input.position();
        if !matches!(input.next_including_whitespace(), Ok(Token::WhiteSpace(_))) {
            input.reset(position);
            break
        }
    }
    let mut empty = true;
    let mut simple_selectors = match try!(parse_type_selector(context, input)) {
        None => vec![],
        Some(s) => { empty = false; s }
    };

    let mut pseudo_element = None;
    loop {
        match try!(parse_one_simple_selector(context,
                                             input,
                                             /* inside_negation = */ false)) {
            None => break,
            Some(SimpleSelectorParseResult::SimpleSelector(s)) => {
                simple_selectors.push(s);
                empty = false
            }
            Some(SimpleSelectorParseResult::PseudoElement(p)) => {
                pseudo_element = Some(p);
                empty = false;
                break
            }
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
                                 input: &mut Parser,
                                 name: &str,
                                 inside_negation: bool)
                                 -> Result<SimpleSelector,()> {
    match_ignore_ascii_case! { name,
        "nth-child" => parse_nth_pseudo_class(input, SimpleSelector::NthChild),
        "nth-of-type" => parse_nth_pseudo_class(input, SimpleSelector::NthOfType),
        "nth-last-child" => parse_nth_pseudo_class(input, SimpleSelector::NthLastChild),
        "nth-last-of-type" => parse_nth_pseudo_class(input, SimpleSelector::NthLastOfType),
        "not" => {
            if inside_negation {
                Err(())
            } else {
                parse_negation(context, input)
            }
        }
        _ => Err(())
    }
}


fn parse_nth_pseudo_class<F>(input: &mut Parser, selector: F) -> Result<SimpleSelector, ()>
where F: FnOnce(i32, i32) -> SimpleSelector {
    let (a, b) = try!(parse_nth(input));
    Ok(selector(a, b))
}


/// Parse a simple selector other than a type selector.
///
/// * `Err(())`: Invalid selector, abort
/// * `Ok(None)`: Not a simple selector, could be something else. `input` was not consumed.
/// * `Ok(Some(_))`: Parsed a simple selector or pseudo-element
fn parse_one_simple_selector(context: &ParserContext,
                             input: &mut Parser,
                             inside_negation: bool)
                             -> Result<Option<SimpleSelectorParseResult>,()> {
    let start_position = input.position();
    match input.next_including_whitespace() {
        Ok(Token::IDHash(id)) => {
            let id = SimpleSelector::ID(Atom::from_slice(&id));
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(id)))
        }
        Ok(Token::Delim('.')) => {
            match input.next_including_whitespace() {
                Ok(Token::Ident(class)) => {
                    let class = SimpleSelector::Class(Atom::from_slice(&class));
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(class)))
                }
                _ => Err(()),
            }
        }
        Ok(Token::SquareBracketBlock) => {
            let attr = try!(input.parse_nested_block(|input| {
                parse_attribute_selector(context, input)
            }));
            Ok(Some(SimpleSelectorParseResult::SimpleSelector(attr)))
        }
        Ok(Token::Colon) => {
            match input.next_including_whitespace() {
                Ok(Token::Ident(name)) => {
                    match parse_simple_pseudo_class(context, &name) {
                        Err(()) => {
                            let pseudo_element = match_ignore_ascii_case! { name,
                                // Supported CSS 2.1 pseudo-elements only.
                                // ** Do not add to this list! **
                                "before" => PseudoElement::Before,
                                "after" => PseudoElement::After,
                                "first-line" => return Err(()),
                                "first-letter" => return Err(())
                                _ => return Err(())
                            };
                            Ok(Some(SimpleSelectorParseResult::PseudoElement(pseudo_element)))
                        },
                        Ok(result) => Ok(Some(SimpleSelectorParseResult::SimpleSelector(result))),
                    }
                }
                Ok(Token::Function(name)) => {
                    let pseudo = try!(input.parse_nested_block(|input| {
                        parse_functional_pseudo_class(context, input, &name, inside_negation)
                    }));
                    Ok(Some(SimpleSelectorParseResult::SimpleSelector(pseudo)))
                }
                Ok(Token::Colon) => {
                    match input.next() {
                        Ok(Token::Ident(name)) => {
                            let pseudo = try!(parse_pseudo_element(&name));
                            Ok(Some(SimpleSelectorParseResult::PseudoElement(pseudo)))
                        }
                        _ => Err(())
                    }
                }
                _ => Err(())
            }
        }
        _ => {
            input.reset(start_position);
            Ok(None)
        }
    }
}

fn parse_simple_pseudo_class(context: &ParserContext, name: &str) -> Result<SimpleSelector,()> {
    match_ignore_ascii_case! { name,
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
        "-servo-nonzero-border" => {
            if context.in_user_agent_stylesheet() {
                Ok(SimpleSelector::ServoNonzeroBorder)
            } else {
                Err(())
            }
        }
        _ => Err(())
    }
}

fn parse_pseudo_element(name: &str) -> Result<PseudoElement, ()> {
    match_ignore_ascii_case! { name,
        "before" => Ok(PseudoElement::Before),
        "after" => Ok(PseudoElement::After)
        _ => Err(())
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use cssparser::Parser;
    use stylesheets::Origin;
    use string_cache::Atom;
    use parser::ParserContext;
    use url::Url;
    use super::*;

    fn parse(input: &str) -> Result<Vec<Selector>, ()> {
        parse_ns(input, &ParserContext::new(Origin::Author, &Url::parse("about:blank").unwrap()))
    }

    fn parse_ns(input: &str, context: &ParserContext) -> Result<Vec<Selector>, ()> {
        parse_selector_list(context, &mut Parser::new(input))
    }

    fn specificity(a: u32, b: u32, c: u32) -> u32 {
        a << 20 | b << 10 | c
    }

    #[test]
    fn test_parsing() {
        assert_eq!(parse(""), Err(())) ;
        assert_eq!(parse("EeÉ"), Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::LocalName(LocalName {
                    name: Atom::from_slice("EeÉ"),
                    lower_name: Atom::from_slice("eeÉ") })),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 0, 1),
        })));
        assert_eq!(parse(".foo"), Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::Class(Atom::from_slice("foo"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(0, 1, 0),
        })));
        assert_eq!(parse("#bar"), Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(SimpleSelector::ID(Atom::from_slice("bar"))),
                next: None,
            }),
            pseudo_element: None,
            specificity: specificity(1, 0, 0),
        })));
        assert_eq!(parse("e.foo#bar"), Ok(vec!(Selector {
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
        })));
        assert_eq!(parse("e.foo #bar"), Ok(vec!(Selector {
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
        })));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        let url = Url::parse("about:blank").unwrap();
        let mut context = ParserContext::new(Origin::Author, &url);
        assert_eq!(parse_ns("[Foo]", &context), Ok(vec!(Selector {
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
        })));
        // Default namespace does not apply to attribute selectors
        // https://github.com/mozilla/servo/pull/1652
        context.namespaces.default = Some(ns!(MathML));
        assert_eq!(parse_ns("[Foo]", &context), Ok(vec!(Selector {
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
        })));
        // Default namespace does apply to type selectors
        assert_eq!(parse_ns("e", &context), Ok(vec!(Selector {
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
        })));
        // https://github.com/mozilla/servo/issues/1723
        assert_eq!(parse("::before"), Ok(vec!(Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec!(),
                next: None,
            }),
            pseudo_element: Some(PseudoElement::Before),
            specificity: specificity(0, 0, 1),
        })));
        assert_eq!(parse("div :after"), Ok(vec!(Selector {
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
        })));
        assert_eq!(parse("#d1 > .ok"), Ok(vec![Selector {
            compound_selectors: Arc::new(CompoundSelector {
                simple_selectors: vec![
                    SimpleSelector::Class(Atom::from_slice("ok")),
                ],
                next: Some((box CompoundSelector {
                    simple_selectors: vec![
                        SimpleSelector::ID(Atom::from_slice("d1")),
                    ],
                    next: None,
                }, Combinator::Child)),
            }),
            pseudo_element: None,
            specificity: (1 << 20) + (1 << 10) + (0 << 0),
        }]))
    }
}
