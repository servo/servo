/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::StrAsciiExt;
use extra::arc::Arc;
use extra::sort::tim_sort;

use selectors::*;
use stylesheets::{Stylesheet, iter_style_rules};
use media_queries::{Device, Screen};
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use servo_util::tree::{TreeNodeRefAsElement, TreeNode, ElementLike};

use std::str;

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

    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet, origin: StylesheetOrigin) {
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
                        // TODO: avoid copying?
                        rules.$priority.push(Rule {
                            selector: selector.clone(),
                            declarations: Arc::new(style_rule.declarations.$priority.clone()),
                        })
                    }
                }
            };
        )

        let device = &Device { media_type: Screen };  // TODO, use Print when printing
        do iter_style_rules(stylesheet.rules.as_slice(), device) |style_rule| {
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

    pub fn get_applicable_declarations<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
            &self, element: &T, style_attribute: Option<&PropertyDeclarationBlock>,
            pseudo_element: Option<PseudoElement>) -> ~[Arc<~[PropertyDeclaration]>] {
        assert!(element.is_element())
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements")
        let mut applicable_declarations = ~[];  // TODO: use an iterator?

        macro_rules! append(
            ($rules: expr) => {
                for rule in $rules.iter() {
                    if matches_selector::<N, T, E>(&rule.selector, element, pseudo_element) {
                        applicable_declarations.push(rule.declarations.clone())
                    }
                }
            };
        );

        // In cascading order
        append!(self.ua_rules.normal);
        append!(self.user_rules.normal);

        // Style attributes have author origin but higher specificity than style rules.
        append!(self.author_rules.normal);
        // TODO: avoid copying?
        style_attribute.map(|sa| applicable_declarations.push(Arc::new(sa.normal.clone())));

        append!(self.author_rules.important);
        // TODO: avoid copying?
        style_attribute.map(|sa| applicable_declarations.push(Arc::new(sa.important.clone())));

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
    selector: Selector,
    declarations: Arc<~[PropertyDeclaration]>,
}


impl Ord for Rule {
    #[inline]
    fn lt(&self, other: &Rule) -> bool {
        self.selector.specificity < other.selector.specificity
    }
}


#[inline]
fn matches_selector<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        selector: &Selector, element: &T, pseudo_element: Option<PseudoElement>) -> bool {
    selector.pseudo_element == pseudo_element &&
    matches_compound_selector::<N, T, E>(&selector.compound_selectors, element)
}

fn matches_compound_selector<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        selector: &CompoundSelector, element: &T) -> bool {
    if !do selector.simple_selectors.iter().all |simple_selector| {
            matches_simple_selector(simple_selector, element)
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
            let mut node = element.clone();
            loop {
                let next_node = if siblings {
                    node.node().prev_sibling()
                } else {
                    node.node().parent_node()
                };
                match next_node {
                    None => return false,
                    Some(next_node) => node = next_node,
                }
                if node.is_element() {
                    if matches_compound_selector(&**next_selector, &node) {
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
fn matches_simple_selector<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        selector: &SimpleSelector, element: &T) -> bool {
    static WHITESPACE: &'static [char] = &'static [' ', '\t', '\n', '\r', '\x0C'];

    match *selector {
        // TODO: case-sensitivity depends on the document type
        // TODO: intern element names
        LocalNameSelector(ref name) => {
            do element.with_imm_element_like |element: &E| {
                element.get_local_name().eq_ignore_ascii_case(name.as_slice())
            }
        }
        NamespaceSelector(_) => false,  // TODO, when the DOM supports namespaces on elements.
        // TODO: case-sensitivity depends on the document type and quirks mode
        // TODO: cache and intern IDs on elements.
        IDSelector(ref id) => {
            do element.with_imm_element_like |element: &E| {
                match element.get_attr("id") {
                    Some(attr) => str::eq_slice(attr, *id),
                    None => false
                }
            }
        }
        // TODO: cache and intern classe names on elements.
        ClassSelector(ref class) => {
            do element.with_imm_element_like |element: &E| {
                match element.get_attr("class") {
                    None => false,
                    // TODO: case-sensitivity depends on the document type and quirks mode
                    Some(ref class_attr)
                    => class_attr.split_iter(WHITESPACE).any(|c| c == class.as_slice()),
                }
            }
        }

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


        AnyLink => {
            do element.with_imm_element_like |element: &E| {
                element.get_link().is_some()
            }
        }
        Link => {
            do element.with_imm_element_like |element: &E| {
                match element.get_link() {
                    Some(url) => !url_is_visited(url),
                    None => false,
                }
            }
        }
        Visited => {
            do element.with_imm_element_like |element: &E| {
                match element.get_link() {
                    Some(url) => url_is_visited(url),
                    None => false,
                }
            }
        }
        FirstChild => matches_first_child(element),

        Negation(ref negated) => {
            !negated.iter().all(|s| matches_simple_selector(s, element))
        },
    }
}

fn url_is_visited(_url: &str) -> bool {
    // FIXME: implement this.
    // This function will probably need to take a "session"
    // or something containing browsing history as an additional parameter.
    false
}

#[inline]
fn matches_first_child<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        element: &T) -> bool {
    let mut node = element.clone();
    loop {
        match node.node().prev_sibling() {
            Some(prev_sibling) => {
                node = prev_sibling;
                if node.is_element() {
                    return false
                }
            }
            None => return !element.is_root(),
        }
    }
}

#[inline]
fn match_attribute<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        attr: &AttrSelector, element: &T, f: &fn(&str)-> bool) -> bool {
    do element.with_imm_element_like |element: &E| {
        match attr.namespace {
            Some(_) => false,  // TODO, when the DOM supports namespaces on attributes
            None => match element.get_attr(attr.name) {
                None => false,
                Some(ref value) => f(value.as_slice())
            }
        }
    }
}
