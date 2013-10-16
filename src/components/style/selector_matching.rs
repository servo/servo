/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::StrAsciiExt;
use extra::sort::tim_sort;

use selectors::*;
use stylesheets::parse_stylesheet;
use media_queries::{Device, Screen};
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use script::dom::node::{AbstractNode, ScriptView};
use script::dom::element::Element;
use servo_util::tree::{TreeNodeRef, ElementLike};


pub enum StylesheetOrigin {
    UserAgentOrigin,
    AuthorOrigin,
    UserOrigin,
}


pub struct Stylist {
    priv ua_rules: PerOriginRules,
    priv author_rules: PerOriginRules,
    priv user_rules: PerOriginRules,
}


impl Stylist {
    #[inline]
    pub fn new() -> Stylist {
        Stylist {
            ua_rules: PerOriginRules::new(),
            author_rules: PerOriginRules::new(),
            user_rules: PerOriginRules::new(),
        }
    }

    pub fn add_stylesheet(&mut self, css_source: &str, origin: StylesheetOrigin) {
        let stylesheet = parse_stylesheet(css_source);
        let rules = match origin {
            UserAgentOrigin => &mut self.ua_rules,
            AuthorOrigin => &mut self.author_rules,
            UserOrigin => &mut self.user_rules,
        };
        let mut added_normal_declarations = false;
        let mut added_important_declarations = false;

        macro_rules! append(
            ($priority: ident, $flag: ident) => {
                if style_rule.declarations.$priority.len() > 0 {
                    $flag = true;
                    for selector in style_rule.selectors.iter() {
                        rules.$priority.push(Rule {
                            selector: *selector,
                            declarations: style_rule.declarations.$priority,
                        })
                    }
                }
            };
        )

        let device = &Device { media_type: Screen };  // TODO, use Print when printing
        for style_rule in stylesheet.iter_style_rules(device) {
            append!(normal, added_normal_declarations);
            append!(important, added_important_declarations);
        }

        // These sorts need to be stable
        // Do not sort already-sorted unchanged vectors
        if added_normal_declarations {
            tim_sort(rules.normal)
        }
        if added_important_declarations {
            tim_sort(rules.important)
        }
    }

    pub fn get_applicable_declarations(&self, element: AbstractNode<ScriptView>,
                                       style_attribute: Option<&PropertyDeclarationBlock>,
                                       pseudo_element: Option<PseudoElement>)
                                       -> ~[@[PropertyDeclaration]] {
        assert!(element.is_element())
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements")
        let mut applicable_declarations = ~[];  // TODO: use an iterator?

        macro_rules! append(
            ($rules: expr) => {
                for rule in $rules.iter() {
                    if matches_selector(rule.selector, element, pseudo_element) {
                        applicable_declarations.push(rule.declarations)
                    }
                }
            };
        );

        // In cascading order
        append!(self.ua_rules.normal);
        append!(self.user_rules.normal);

        // Style attributes have author origin but higher specificity than style rules.
        append!(self.author_rules.normal);
        style_attribute.map(|sa| applicable_declarations.push(sa.normal));

        append!(self.author_rules.important);
        style_attribute.map(|sa| applicable_declarations.push(sa.important));

        append!(self.user_rules.important);
        append!(self.ua_rules.important);

        applicable_declarations
    }
}


struct PerOriginRules {
    normal: ~[Rule],
    important: ~[Rule],
}

impl PerOriginRules {
    #[inline]
    fn new() -> PerOriginRules {
        PerOriginRules { normal: ~[], important: ~[] }
    }
}

#[deriving(Clone)]
struct Rule {
    selector: @Selector,
    declarations: @[PropertyDeclaration],
}


impl Ord for Rule {
    #[inline]
    fn lt(&self, other: &Rule) -> bool {
        self.selector.specificity < other.selector.specificity
    }
}


#[inline]
fn matches_selector(selector: &Selector, element: AbstractNode<ScriptView>,
                    pseudo_element: Option<PseudoElement>) -> bool {
    selector.pseudo_element == pseudo_element &&
    matches_compound_selector(&selector.compound_selectors, element)
}


fn matches_compound_selector(selector: &CompoundSelector,
                             element: AbstractNode<ScriptView>) -> bool {
    if do element.with_imm_element |element| {
        !do selector.simple_selectors.iter().all |simple_selector| {
            matches_simple_selector(simple_selector, element)
        }
    } {
        return false
    }
    match selector.next {
        None => true,
        Some((ref next_selector, combinator)) => {
            let (siblings, just_one) = match combinator {
                Child => (false, true),
                Descendant => (false, false),
                NextSibling => (true, true),
                LaterSibling => (true, false),
            };
            let mut node = element;
            loop {
                match if siblings { node.prev_sibling() } else { node.parent_node() } {
                    None => return false,
                    Some(next_node) => node = next_node,
                }
                if node.is_element() {
                    if matches_compound_selector(&**next_selector, node) {
                        return true
                    } else if just_one {
                        return false
                    }
                }
            }
        }
    }
}

#[inline]
fn matches_simple_selector(selector: &SimpleSelector, element: &Element) -> bool {
    static WHITESPACE: &'static [char] = &'static [' ', '\t', '\n', '\r', '\x0C'];

    match *selector {
        // TODO: case-sensitivity depends on the document type
        // TODO: intern element names
        LocalNameSelector(ref name) => element.tag_name.eq_ignore_ascii_case(name.as_slice()),
        NamespaceSelector(_) => false,  // TODO, when the DOM supports namespaces on elements.
        // TODO: case-sensitivity depends on the document type and quirks mode
        // TODO: cache and intern IDs on elements.
        IDSelector(ref id) => element.get_attr("id") == Some(id.as_slice()),
        // TODO: cache and intern classe names on elements.
        ClassSelector(ref class) => match element.get_attr("class") {
            None => false,
            // TODO: case-sensitivity depends on the document type and quirks mode
            Some(ref class_attr)
            => class_attr.split_iter(WHITESPACE).any(|c| c == class.as_slice()),
        },

        AttrExists(ref attr) => match_attribute(attr, element, |_| true),
        AttrEqual(ref attr, ref value) => match_attribute(attr, element, |v| v == value.as_slice()),
        AttrIncludes(ref attr, ref value) => do match_attribute(attr, element) |attr_value| {
            attr_value.split_iter(WHITESPACE).any(|v| v == value.as_slice())
        },
        AttrDashMatch(ref attr, ref value, ref dashing_value)
        => do match_attribute(attr, element) |attr_value| {
            attr_value == value.as_slice() || attr_value.starts_with(dashing_value.as_slice())
        },
        AttrPrefixMatch(ref attr, ref value) => do match_attribute(attr, element) |attr_value| {
            attr_value.starts_with(value.as_slice())
        },
        AttrSubstringMatch(ref attr, ref value) => do match_attribute(attr, element) |attr_value| {
            attr_value.contains(value.as_slice())
        },
        AttrSuffixMatch(ref attr, ref value) => do match_attribute(attr, element) |attr_value| {
            attr_value.ends_with(value.as_slice())
        },

        Negation(ref negated) => {
            !negated.iter().all(|s| matches_simple_selector(s, element))
        },
    }
}


#[inline]
fn match_attribute(attr: &AttrSelector, element: &Element, f: &fn(&str)-> bool) -> bool {
    match attr.namespace {
        Some(_) => false,  // TODO, when the DOM supports namespaces on attributes
        None => match element.get_attr(attr.name) {
            None => false,
            Some(ref value) => f(value.as_slice())
        }
    }
}
