/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::StrAsciiExt;
use std::hashmap::HashMap;
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


/// Map node attributes to Rules whose last simple selector starts with them.
///
/// e.g.,
/// "p > img" would go into the set of Rules corresponding to the
/// element "img"
/// "a .foo .bar.baz" would go into the set of Rules corresponding to
/// the class "bar"
///
/// Because we match Rules right-to-left (i.e., moving up the tree
/// from a node), we need to compare the last simple selector in the
/// Rule with the node.
///
/// So, if a node has ID "id1" and classes "foo" and "bar", then all
/// the rules it matches will have their last simple selector starting
/// either with "#id1" or with ".foo" or with ".bar".
///
/// Hence, the union of the rules keyed on each of node's classes, ID,
/// element name, etc. will contain the Rules that actually match that
/// node.
pub struct SelectorMap {
    // TODO: Tune the initial capacity of the HashMap
    // FIXME: Use interned strings
    priv id_hash: HashMap<~str, ~[Rule]>,
    priv class_hash: HashMap<~str, ~[Rule]>,
    priv element_hash: HashMap<~str, ~[Rule]>,
    // For Rules that don't have ID, class, or element selectors.
    priv universal_rules: ~[Rule],
}

impl SelectorMap {
    fn new() -> SelectorMap {
        SelectorMap {
            id_hash: HashMap::new(),
            class_hash: HashMap::new(),
            element_hash: HashMap::new(),
            universal_rules: ~[],
        }
    }

    /// Append to `rule_list` all Rules in `self` that match node.
    ///
    /// Extract matching rules as per node's ID, classes, tag name, etc..
    /// Sort the Rules at the end to maintain cascading order.
    fn get_all_matching_rules<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        &self, node: &T,
        pseudo_element: Option<PseudoElement>,
        matching_rules_list: &mut [~[Rule]],
        list_index: uint) {

        let init_len = matching_rules_list[list_index].len();
        static WHITESPACE: &'static [char] = &'static [' ', '\t', '\n', '\r', '\x0C'];
        do node.with_imm_element_like |element: &E| {
            match element.get_attr("id") {
                Some(id) => SelectorMap::get_matching_rules_from_hash(
                    node, pseudo_element, &self.id_hash, id, &mut matching_rules_list[list_index]),
                None => {}
            }

            match element.get_attr("class") {
                Some(ref class_attr) => {
                    for class in class_attr.split_iter(WHITESPACE) {
                        SelectorMap::get_matching_rules_from_hash(
                            node, pseudo_element, &self.class_hash, class, &mut matching_rules_list[list_index]);
                    }
                }
                None => {}
            }

            SelectorMap::get_matching_rules_from_hash(
                node, pseudo_element, &self.element_hash,
                // HTML elements in HTML documents must be matched case-insensitively
                // TODO: case-sensitivity depends on the document type
                element.get_local_name().to_ascii_lower(),
                &mut matching_rules_list[list_index]);
            SelectorMap::get_matching_rules(
                node, pseudo_element, self.universal_rules, &mut matching_rules_list[list_index]);
        }

        // Sort only the rules we just added.
        tim_sort(matching_rules_list[list_index].mut_slice_from(init_len));
    }

    fn get_matching_rules_from_hash<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        node: &T,
        pseudo_element: Option<PseudoElement>,
        hash: &HashMap<~str, ~[Rule]>, 
        key: &str,
        matching_rules: &mut ~[Rule]) {
        match hash.find(&key.to_str()) {
            Some(rules) => SelectorMap::get_matching_rules(node, pseudo_element, *rules,
                                                           matching_rules),
            None => {}
        };
    }
    
    /// Return rules in `rules` that match `node`.
    fn get_matching_rules<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        node: &T,
        pseudo_element: Option<PseudoElement>,
        rules: &[Rule],
        matching_rules: &mut ~[Rule]) {
        for rule in rules.iter() {
            if matches_selector(rule.selector.get(), node, pseudo_element) {
                // TODO: Is the cloning inefficient?
                matching_rules.push(rule.clone());
            }
        }
    }

    /// Insert rule into the correct hash.
    /// Order in which to try: id_hash, class_hash, element_hash, universal_rules.
    fn insert(&mut self, rule: Rule) {
        match SelectorMap::get_id_name(&rule) {
            Some(id_name) => {
                match self.id_hash.find_mut(&id_name) {
                    Some(rules) => {
                        rules.push(rule);
                        return;
                    }
                    None => {}
                }
                self.id_hash.insert(id_name, ~[rule]);
                return;
            }
            None => {}
        }
        match SelectorMap::get_class_name(&rule) {
            Some(class_name) => {
                match self.class_hash.find_mut(&class_name) {
                    Some(rules) => {
                        rules.push(rule);
                        return;
                    }
                    None => {}
                }
                self.class_hash.insert(class_name, ~[rule]);
                return;
            }
            None => {}
        }

        match SelectorMap::get_element_name(&rule) {
            Some(element_name) => {
                match self.element_hash.find_mut(&element_name) {
                    Some(rules) => {
                        rules.push(rule);
                        return;
                    }
                    None => {}
                }
                self.element_hash.insert(element_name, ~[rule]);
                return;
            }
            None => {}
        }

        self.universal_rules.push(rule);
    }

    /// Retrieve the first ID name in Rule, or None otherwise.
    fn get_id_name(rule: &Rule) -> Option<~str> {
        let simple_selector_sequence = &rule.selector.get().compound_selectors.simple_selectors;
        for ss in simple_selector_sequence.iter() {
            match *ss {
                // TODO: Implement case-sensitivity based on the document type and quirks mode
                IDSelector(ref id) => return Some(id.clone()),
                _ => {}
            }
        }
        return None
    }

    /// Retrieve the FIRST class name in Rule, or None otherwise.
    fn get_class_name(rule: &Rule) -> Option<~str> {
        let simple_selector_sequence = &rule.selector.get().compound_selectors.simple_selectors;
        for ss in simple_selector_sequence.iter() {
            match *ss {
                // TODO: Implement case-sensitivity based on the document type and quirks mode
                ClassSelector(ref class) => return Some(class.clone()),
                _ => {}
            }
        }
        return None
    }

    /// Retrieve the name if it is a type selector, or None otherwise.
    fn get_element_name(rule: &Rule) -> Option<~str> {
        let simple_selector_sequence = &rule.selector.get().compound_selectors.simple_selectors;
        for ss in simple_selector_sequence.iter() {
            match *ss {
                // HTML elements in HTML documents must be matched case-insensitively
                // TODO: case-sensitivity depends on the document type
                LocalNameSelector(ref name) => return Some(name.to_ascii_lower()),
                _ => {}
            }
        }
        return None
    }
}

pub struct Stylist {
    priv ua_rule_map: PerOriginSelectorMap,
    priv author_rule_map: PerOriginSelectorMap,
    priv user_rule_map: PerOriginSelectorMap,
    priv stylesheet_index: uint,
}

impl Stylist {
    #[inline]
    pub fn new() -> Stylist {
        Stylist {
            ua_rule_map: PerOriginSelectorMap::new(),
            author_rule_map: PerOriginSelectorMap::new(),
            user_rule_map: PerOriginSelectorMap::new(),
            stylesheet_index: 0u,
        }
    }

    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet, origin: StylesheetOrigin) {
        let rule_map = match origin {
            UserAgentOrigin => &mut self.ua_rule_map,
            AuthorOrigin => &mut self.author_rule_map,
            UserOrigin => &mut self.user_rule_map,
        };
        let mut added_normal_declarations = false;
        let mut added_important_declarations = false;
        let mut style_rule_index = 0u;

        // Take apart the StyleRule into individual Rules and insert
        // them into the SelectorMap of that priority.
        macro_rules! append(
            ($priority: ident, $flag: ident) => {
                if style_rule.declarations.$priority.get().len() > 0 {
                    $flag = true;
                    for selector in style_rule.selectors.iter() {
                        // TODO: avoid copying?
                        rule_map.$priority.insert(Rule {
                                selector: Arc::new(selector.clone()),
                                declarations: style_rule.declarations.$priority.clone(),
                                index: style_rule_index,
                                stylesheet_index: self.stylesheet_index,
                            });
                    }
                }
            };
        );

        let device = &Device { media_type: Screen };  // TODO, use Print when printing
        do iter_style_rules(stylesheet.rules.as_slice(), device) |style_rule| {
            append!(normal, added_normal_declarations);
            append!(important, added_important_declarations);
            style_rule_index += 1u;
        }
        self.stylesheet_index += 1;
    }

    pub fn get_applicable_declarations<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
            &self, element: &T, style_attribute: Option<&PropertyDeclarationBlock>,
            pseudo_element: Option<PseudoElement>) -> ~[Arc<~[PropertyDeclaration]>] {
        assert!(element.is_element());
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements");
            
        // In cascading order
        let rule_map_list = [&self.ua_rule_map.normal,
                             &self.user_rule_map.normal,
                             &self.author_rule_map.normal,
                             &self.author_rule_map.important,
                             &self.user_rule_map.important,
                             &self.ua_rule_map.important];

        let mut matching_rules_list: [~[Rule], ..6] = [~[], ~[], ~[], ~[], ~[], ~[]];
        for (i, rule_map) in rule_map_list.iter().enumerate() {
            rule_map.get_all_matching_rules(element, pseudo_element, matching_rules_list, i);
        }
        
        // Keeping this as a separate step because we will need it for further
        // optimizations regarding grouping of Rules having the same Selector.
        let declarations_list: ~[~[Arc<~[PropertyDeclaration]>]] = matching_rules_list.iter().map(
            |rules| rules.iter().map(|r| r.declarations.clone()).collect()).collect();

        let mut applicable_declarations = ~[];
        applicable_declarations.push_all_move(declarations_list.slice(0, 3).concat_vec());
        // Style attributes have author origin but higher specificity than style rules.
        // TODO: avoid copying?
        style_attribute.map(|sa| applicable_declarations.push(sa.normal.clone()));
        applicable_declarations.push_all_move(declarations_list.slice(3, 4).concat_vec());
        style_attribute.map(|sa| applicable_declarations.push(sa.important.clone()));
        applicable_declarations.push_all_move(declarations_list.slice(4, 6).concat_vec());

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

struct PerOriginSelectorMap {
    normal: SelectorMap,
    important: SelectorMap,
}

impl PerOriginSelectorMap {
    #[inline]
    fn new() -> PerOriginSelectorMap {
        PerOriginSelectorMap { normal: SelectorMap::new(), important:SelectorMap::new() }
    }
}

#[deriving(Clone)]
struct Rule {
    // This is an Arc because Rule will essentially be cloned for every node
    // that it matches. Selector contains an owned vector (through
    // CompoundSelector) and we want to avoid the allocation.
    selector: Arc<Selector>,
    declarations: Arc<~[PropertyDeclaration]>,
    // Index of the parent StyleRule in the parent Stylesheet (useful for
    // breaking ties while cascading).
    index: uint,
    // Index of the parent stylesheet among all the stylesheets
    stylesheet_index: uint,
}


impl Ord for Rule {
    #[inline]
    fn lt(&self, other: &Rule) -> bool {
        let this_rank = (self.selector.get().specificity, self.stylesheet_index, self.index);
        let other_rank = (other.selector.get().specificity, other.stylesheet_index, other.index);
        this_rank < other_rank
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
        LastChild  => matches_last_child(element),
        OnlyChild  => matches_first_child(element) &&
                      matches_last_child(element),

        Root => matches_root(element),

        NthChild(a, b)      => matches_generic_nth_child(element, a, b, false, false),
        NthLastChild(a, b)  => matches_generic_nth_child(element, a, b, false, true),
        NthOfType(a, b)     => matches_generic_nth_child(element, a, b, true, false),
        NthLastOfType(a, b) => matches_generic_nth_child(element, a, b, true, true),

        FirstOfType => matches_generic_nth_child(element, 0, 1, true, false),
        LastOfType  => matches_generic_nth_child(element, 0, 1, true, true),
        OnlyOfType  => matches_generic_nth_child(element, 0, 1, true, false) &&
                       matches_generic_nth_child(element, 0, 1, true, true),

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
fn matches_generic_nth_child<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        element: &T, a: i32, b: i32, is_of_type: bool, is_from_end: bool) -> bool {
    let mut node = element.clone();
    // fail if we can't find a parent or if the node is the root element
    // of the document (Cf. Selectors Level 3)
    match node.node().parent_node() {
        Some(parent) => if parent.is_document() {
            return false;
        },
        None => return false
    };

    let mut local_name = "";
    if is_of_type {
        // FIXME this is wrong
        // TODO when the DOM supports namespaces on elements
        do element.with_imm_element_like |element: &E| {
            local_name = element.get_local_name();
        }
    }

    let mut index = 1;
    loop {
        if is_from_end {
            match node.node().next_sibling() {
                None => break,
                Some(next_sibling) => node = next_sibling
            }
        } else {
            match node.node().prev_sibling() {
                None => break,
                Some(prev_sibling) => node = prev_sibling
            }
        }

        if node.is_element() {
            if is_of_type {
                // FIXME this is wrong
                // TODO when the DOM supports namespaces on elements
                do node.with_imm_element_like |node: &E| {
                    if local_name == node.get_local_name() {
                        index += 1;
                    }
                }
            } else {
              index += 1;
            }
        }

    }

    if a == 0 {
        return b == index;
    }

    let (n, r) = (index - b).div_rem(&a);
    n >= 0 && r == 0
}

#[inline]
fn matches_root<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        element: &T) -> bool {
    match element.node().parent_node() {
        Some(parent) => parent.is_document(),
        None => false
    }
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
            },
            None => match node.node().parent_node() {
                // Selectors level 3 says :first-child does not match the
                // root of the document; Warning, level 4 says, for the time
                // being, the contrary...
                Some(parent) => return !parent.is_document(),
                None => return false
            }
        }
    }
}

#[inline]
fn matches_last_child<N: TreeNode<T>, T: TreeNodeRefAsElement<N, E>, E: ElementLike>(
        element: &T) -> bool {
    let mut node = element.clone();
    loop {
        match node.node().next_sibling() {
            Some(next_sibling) => {
                node = next_sibling;
                if node.is_element() {
                    return false
                }
            },
            None => match node.node().parent_node() {
                // Selectors level 3 says :last-child does not match the
                // root of the document; Warning, level 4 says, for the time
                // being, the contrary...
                Some(parent) => return !parent.is_document(),
                None => return false
            }
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
fn get_rules(css_string: &str) -> ~[~[Rule]] {
    let device = &Device { media_type: Screen };
    let sheet = Stylesheet::from_str(css_string);
    let mut index = 0u;
    let mut results = ~[];
    do iter_style_rules(sheet.rules.as_slice(), device) |style_rule| {
        results.push(style_rule.selectors.iter().map(|s| Rule {
                    selector: Arc::new(s.clone()),
                    declarations: style_rule.declarations.normal.clone(),
                    index: index,
                    stylesheet_index: 0u,
                }).collect());
        index += 1u;
    }
    results
}

/// Helper method to get some Rules from selector strings.
/// Each sublist of the result contains the Rules for one StyleRule.
fn get_mock_rules(css_selectors: &[&str]) -> ~[~[Rule]] {
    let css_string = css_selectors.map(|s| s + " { color: red; } ").concat();
    get_rules(css_string)
}

#[test]
fn test_rule_ordering_same_specificity(){
    let rules_list = get_mock_rules(["a.intro", "img.sidebar"]);
    let rule1 = rules_list[0][0].clone();
    let rule2 = rules_list[1][0].clone();
    assert!(rule1 < rule2, "The rule that comes later should win.");
}

#[test]
fn test_get_id_name(){
    let rules_list = get_mock_rules([".intro", "#top"]);
    assert_eq!(SelectorMap::get_id_name(&rules_list[0][0]), None);
    assert_eq!(SelectorMap::get_id_name(&rules_list[1][0]), Some(~"top"));
}

#[test]
fn test_get_class_name(){
    let rules_list = get_mock_rules([".intro.foo", "#top"]);
    assert_eq!(SelectorMap::get_class_name(&rules_list[0][0]), Some(~"intro"));
    assert_eq!(SelectorMap::get_class_name(&rules_list[1][0]), None);
}

#[test]
fn test_get_element_name(){
    let rules_list = get_mock_rules(["img.foo", "#top", "IMG", "ImG"]);
    assert_eq!(SelectorMap::get_element_name(&rules_list[0][0]), Some(~"img"));
    assert_eq!(SelectorMap::get_element_name(&rules_list[1][0]), None);
    assert_eq!(SelectorMap::get_element_name(&rules_list[2][0]), Some(~"img"));
    assert_eq!(SelectorMap::get_element_name(&rules_list[3][0]), Some(~"img"));
}

#[test]
fn test_insert(){
    let rules_list = get_mock_rules([".intro.foo", "#top"]);
    let mut selector_map = SelectorMap::new();
    selector_map.insert(rules_list[1][0].clone());
    assert_eq!(1, selector_map.id_hash.find(&~"top").unwrap()[0].index);
    selector_map.insert(rules_list[0][0].clone());
    assert_eq!(0, selector_map.class_hash.find(&~"intro").unwrap()[0].index);
    assert!(selector_map.class_hash.find(&~"foo").is_none());
}

