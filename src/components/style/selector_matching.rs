/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use extra::arc::Arc;
use extra::sort::tim_sort;
use std::ascii::StrAsciiExt;
use std::hashmap::HashMap;
use std::str;

use media_queries::{Device, Screen};
use node::{TElement, TNode};
use properties::{PropertyDeclaration, PropertyDeclarationBlock};
use selectors::*;
use stylesheets::{Stylesheet, iter_style_rules};

pub enum StylesheetOrigin {
    UserAgentOrigin,
    AuthorOrigin,
    UserOrigin,
}

/// The definition of whitespace per CSS Selectors Level 3 § 4.
static SELECTOR_WHITESPACE: &'static [char] = &'static [' ', '\t', '\n', '\r', '\x0C'];

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
struct SelectorMap {
    // TODO: Tune the initial capacity of the HashMap
    // FIXME: Use interned strings
    id_hash: HashMap<~str, ~[Rule]>,
    class_hash: HashMap<~str, ~[Rule]>,
    element_hash: HashMap<~str, ~[Rule]>,
    // For Rules that don't have ID, class, or element selectors.
    universal_rules: ~[Rule],
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
    fn get_all_matching_rules<E:TElement,
                              N:TNode<E>>(
                              &self,
                              node: &N,
                              matching_rules_list: &mut ~[Rule]) {
        // At the end, we're going to sort the rules that we added, so remember where we began.
        let init_len = matching_rules_list.len();
        node.with_element(|element: &E| {
            match element.get_attr(None, "id") {
                Some(id) => {
                    SelectorMap::get_matching_rules_from_hash(node,
                                                              &self.id_hash,
                                                              id,
                                                              matching_rules_list)
                }
                None => {}
            }

            match element.get_attr(None, "class") {
                Some(ref class_attr) => {
                    for class in class_attr.split_iter(SELECTOR_WHITESPACE) {
                        SelectorMap::get_matching_rules_from_hash(node,
                                                                  &self.class_hash,
                                                                  class,
                                                                  matching_rules_list)
                    }
                }
                None => {}
            }

            // HTML elements in HTML documents must be matched case-insensitively.
            // TODO(pradeep): Case-sensitivity depends on the document type.
            SelectorMap::get_matching_rules_from_hash(node,
                                                      &self.element_hash,
                                                      element.get_local_name().to_ascii_lower(),
                                                      matching_rules_list);
            SelectorMap::get_matching_rules(node,
                                            self.universal_rules,
                                            matching_rules_list);
        });

        // Sort only the rules we just added.
        tim_sort(matching_rules_list.mut_slice_from(init_len));
    }

    fn get_matching_rules_from_hash<E:TElement,
                                    N:TNode<E>>(
                                    node: &N,
                                    hash: &HashMap<~str,~[Rule]>,
                                    key: &str,
                                    matching_rules: &mut ~[Rule]) {
        match hash.find(&key.to_str()) {
            Some(rules) => {
                SelectorMap::get_matching_rules(node, *rules, matching_rules)
            }
            None => {}
        }
    }

    /// Adds rules in `rules` that match `node` to the `matching_rules` list.
    fn get_matching_rules<E:TElement,
                          N:TNode<E>>(
                          node: &N,
                          rules: &[Rule],
                          matching_rules: &mut ~[Rule]) {
        for rule in rules.iter() {
            if matches_compound_selector(rule.selector.get(), node) {
                // TODO(pradeep): Is the cloning inefficient?
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
        let simple_selector_sequence = &rule.selector.get().simple_selectors;
        for ss in simple_selector_sequence.iter() {
            match *ss {
                // TODO(pradeep): Implement case-sensitivity based on the document type and quirks
                // mode.
                IDSelector(ref id) => return Some(id.clone()),
                _ => {}
            }
        }
        return None
    }

    /// Retrieve the FIRST class name in Rule, or None otherwise.
    fn get_class_name(rule: &Rule) -> Option<~str> {
        let simple_selector_sequence = &rule.selector.get().simple_selectors;
        for ss in simple_selector_sequence.iter() {
            match *ss {
                // TODO(pradeep): Implement case-sensitivity based on the document type and quirks
                // mode.
                ClassSelector(ref class) => return Some(class.clone()),
                _ => {}
            }
        }
        return None
    }

    /// Retrieve the name if it is a type selector, or None otherwise.
    fn get_element_name(rule: &Rule) -> Option<~str> {
        let simple_selector_sequence = &rule.selector.get().simple_selectors;
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
    priv element_map: PerPseudoElementSelectorMap,
    priv before_map: PerPseudoElementSelectorMap,
    priv after_map: PerPseudoElementSelectorMap,
    priv rules_source_order: uint,
}

impl Stylist {
    #[inline]
    pub fn new() -> Stylist {
        Stylist {
            element_map: PerPseudoElementSelectorMap::new(),
            before_map: PerPseudoElementSelectorMap::new(),
            after_map: PerPseudoElementSelectorMap::new(),
            rules_source_order: 0u,
        }
    }

    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet, origin: StylesheetOrigin) {
        let (mut element_map, mut before_map, mut after_map) = match origin {
            UserAgentOrigin => (
                &mut self.element_map.user_agent,
                &mut self.before_map.user_agent,
                &mut self.after_map.user_agent,
            ),
            AuthorOrigin => (
                &mut self.element_map.author,
                &mut self.before_map.author,
                &mut self.after_map.author,
            ),
            UserOrigin => (
                &mut self.element_map.user,
                &mut self.before_map.user,
                &mut self.after_map.user,
            ),
        };

        // Take apart the StyleRule into individual Rules and insert
        // them into the SelectorMap of that priority.
        macro_rules! append(
            ($priority: ident) => {
                if style_rule.declarations.$priority.get().len() > 0 {
                    for selector in style_rule.selectors.iter() {
                        let map = match selector.pseudo_element {
                            None => &mut element_map,
                            Some(Before) => &mut before_map,
                            Some(After) => &mut after_map,
                        };
                        map.$priority.insert(Rule {
                                selector: selector.compound_selectors.clone(),
                                specificity: selector.specificity,
                                declarations: style_rule.declarations.$priority.clone(),
                                source_order: self.rules_source_order,
                        });
                    }
                }
            };
        );

        let device = &Device { media_type: Screen };  // TODO, use Print when printing
        do iter_style_rules(stylesheet.rules.as_slice(), device) |style_rule| {
            append!(normal);
            append!(important);
            self.rules_source_order += 1;
        }
    }

    /// Returns the applicable CSS declarations for the given element. This corresponds to
    /// `ElementRuleCollector` in WebKit.
    pub fn get_applicable_declarations<E:TElement,
                                       N:TNode<E>>(
                                       &self,
                                       element: &N,
                                       style_attribute: Option<&PropertyDeclarationBlock>,
                                       pseudo_element: Option<PseudoElement>)
                                       -> ~[Arc<~[PropertyDeclaration]>] {
        assert!(element.is_element());
        assert!(style_attribute.is_none() || pseudo_element.is_none(),
                "Style attributes do not apply to pseudo-elements");

        let map = match pseudo_element {
            None => &self.element_map,
            Some(Before) => &self.before_map,
            Some(After) => &self.after_map,
        };
        // In cascading order:
        let rule_map_list = [
            &map.user_agent.normal,
            &map.user.normal,
            &map.author.normal,
            &map.author.important,
            &map.user.important,
            &map.user_agent.important
        ];

        // We keep track of the indices of each of the rule maps in the list we're building so that
        // we have the indices straight at the end.
        let mut rule_map_indices = [ 0, ..6 ];

        // TODO(pcwalton): Small vector optimization.
        let mut matching_rules_list = ~[];

        for (i, rule_map) in rule_map_list.iter().enumerate() {
            rule_map_indices[i] = matching_rules_list.len();
            rule_map.get_all_matching_rules(element, &mut matching_rules_list);
        }

        let count = matching_rules_list.len();

        let mut declaration_iter = matching_rules_list.move_iter().map(|rule| {
            let Rule {
                declarations,
                _
            } = rule;
            declarations
        });

        // Gather up all rules.
        let mut applicable_declarations = ~[];
        let mut i = 0;

        // Step 1: Normal rules.
        while i < rule_map_indices[3] {
            applicable_declarations.push(declaration_iter.next().unwrap());
            i += 1
        }

        // Step 2: Normal style attributes.
        style_attribute.map(|sa| applicable_declarations.push(sa.normal.clone()));

        // Step 3: Author-supplied `!important` rules.
        while i < rule_map_indices[4] {
            applicable_declarations.push(declaration_iter.next().unwrap());
            i += 1
        }

        // Step 4: `!important` style attributes.
        style_attribute.map(|sa| applicable_declarations.push(sa.important.clone()));

        // Step 5: User and UA `!important` rules.
        while i < count {
            applicable_declarations.push(declaration_iter.next().unwrap());
            i += 1
        }

        applicable_declarations
    }
}

struct PerOriginSelectorMap {
    normal: SelectorMap,
    important: SelectorMap,
}

impl PerOriginSelectorMap {
    #[inline]
    fn new() -> PerOriginSelectorMap {
        PerOriginSelectorMap {
            normal: SelectorMap::new(),
            important: SelectorMap::new(),
        }
    }
}

struct PerPseudoElementSelectorMap {
    user_agent: PerOriginSelectorMap,
    author: PerOriginSelectorMap,
    user: PerOriginSelectorMap,
}

impl PerPseudoElementSelectorMap {
    #[inline]
    fn new() -> PerPseudoElementSelectorMap {
        PerPseudoElementSelectorMap {
            user_agent: PerOriginSelectorMap::new(),
            author: PerOriginSelectorMap::new(),
            user: PerOriginSelectorMap::new(),
        }
    }
}

#[deriving(Clone)]
struct Rule {
    // This is an Arc because Rule will essentially be cloned for every node
    // that it matches. Selector contains an owned vector (through
    // CompoundSelector) and we want to avoid the allocation.
    selector: Arc<CompoundSelector>,
    declarations: Arc<~[PropertyDeclaration]>,
    // Precedence among rules of equal specificity
    source_order: uint,
    specificity: u32,
}

impl Ord for Rule {
    #[inline]
    fn lt(&self, other: &Rule) -> bool {
        let this_rank = (self.specificity, self.source_order);
        let other_rank = (other.specificity, other.source_order);
        this_rank < other_rank
    }
}

fn matches_compound_selector<E:TElement,N:TNode<E>>(selector: &CompoundSelector, element: &N)
                             -> bool {
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
            let mut node = (*element).clone();
            loop {
                let next_node = if siblings {
                    node.prev_sibling()
                } else {
                    node.parent_node()
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
fn matches_simple_selector<E:TElement,N:TNode<E>>(selector: &SimpleSelector, element: &N) -> bool {
    match *selector {
        // TODO: case-sensitivity depends on the document type
        // TODO: intern element names
        LocalNameSelector(ref name) => {
            do element.with_element |element: &E| {
                element.get_local_name().eq_ignore_ascii_case(name.as_slice())
            }
        }
        NamespaceSelector(ref url) => {
            do element.with_element |element: &E| {
                element.get_namespace_url() == url.as_slice()
            }
        }
        // TODO: case-sensitivity depends on the document type and quirks mode
        // TODO: cache and intern IDs on elements.
        IDSelector(ref id) => {
            do element.with_element |element: &E| {
                match element.get_attr(None, "id") {
                    Some(attr) => str::eq_slice(attr, *id),
                    None => false
                }
            }
        }
        // TODO: cache and intern classe names on elements.
        ClassSelector(ref class) => {
            do element.with_element |element: &E| {
                match element.get_attr(None, "class") {
                    None => false,
                    // TODO: case-sensitivity depends on the document type and quirks mode
                    Some(ref class_attr)
                    => class_attr.split_iter(SELECTOR_WHITESPACE).any(|c| c == class.as_slice()),
                }
            }
        }

        AttrExists(ref attr) => match_attribute(attr, element, |_| true),
        AttrEqual(ref attr, ref value) => match_attribute(attr, element, |v| v == value.as_slice()),
        AttrIncludes(ref attr, ref value) => do match_attribute(attr, element) |attr_value| {
            attr_value.split_iter(SELECTOR_WHITESPACE).any(|v| v == value.as_slice())
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
            do element.with_element |element: &E| {
                element.get_link().is_some()
            }
        }
        Link => {
            do element.with_element |element: &E| {
                match element.get_link() {
                    Some(url) => !url_is_visited(url),
                    None => false,
                }
            }
        }
        Visited => {
            do element.with_element |element: &E| {
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
fn matches_generic_nth_child<E:TElement,
                             N:TNode<E>>(
                             element: &N,
                             a: i32,
                             b: i32,
                             is_of_type: bool,
                             is_from_end: bool)
                             -> bool {
    let mut node = element.clone();
    // fail if we can't find a parent or if the node is the root element
    // of the document (Cf. Selectors Level 3)
    match node.parent_node() {
        Some(parent) => if parent.is_document() {
            return false;
        },
        None => return false
    };

    let mut element_local_name = "";
    let mut element_namespace = "";
    if is_of_type {
        do element.with_element |element: &E| {
            element_local_name = element.get_local_name();
            element_namespace = element.get_namespace_url();
        }
    }

    let mut index = 1;
    loop {
        if is_from_end {
            match node.next_sibling() {
                None => break,
                Some(next_sibling) => node = next_sibling
            }
        } else {
            match node.prev_sibling() {
                None => break,
                Some(prev_sibling) => node = prev_sibling
            }
        }

        if node.is_element() {
            if is_of_type {
                do node.with_element |node: &E| {
                    if element_local_name == node.get_local_name() &&
                       element_namespace == node.get_namespace_url() {
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
fn matches_root<E:TElement,N:TNode<E>>(element: &N) -> bool {
    match element.parent_node() {
        Some(parent) => parent.is_document(),
        None => false
    }
}

#[inline]
fn matches_first_child<E:TElement,N:TNode<E>>(element: &N) -> bool {
    let mut node = element.clone();
    loop {
        match node.prev_sibling() {
            Some(prev_sibling) => {
                node = prev_sibling;
                if node.is_element() {
                    return false
                }
            },
            None => match node.parent_node() {
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
fn matches_last_child<E:TElement,N:TNode<E>>(element: &N) -> bool {
    let mut node = element.clone();
    loop {
        match node.next_sibling() {
            Some(next_sibling) => {
                node = next_sibling;
                if node.is_element() {
                    return false
                }
            },
            None => match node.parent_node() {
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
fn match_attribute<E:TElement,
                   N:TNode<E>>(
                   attr: &AttrSelector,
                   element: &N,
                   f: &fn(&str) -> bool)
                   -> bool {
    do element.with_element |element: &E| {
        // FIXME: avoid .clone() here? See #1367
        match element.get_attr(attr.namespace.clone(), attr.name) {
            None => false,
            Some(value) => f(value)
        }
    }
}


#[cfg(test)]
mod tests {
    use extra::arc::Arc;
    use super::{Rule, SelectorMap};

    /// Helper method to get some Rules from selector strings.
    /// Each sublist of the result contains the Rules for one StyleRule.
    fn get_mock_rules(css_selectors: &[&str]) -> ~[~[Rule]] {
        use namespaces::NamespaceMap;
        use selectors::parse_selector_list;
        use cssparser::tokenize;

        let namespaces = NamespaceMap::new();
        css_selectors.iter().enumerate().map(|(i, selectors)| {
            parse_selector_list(tokenize(*selectors).map(|(c, _)| c).to_owned_vec(), &namespaces)
            .unwrap().move_iter().map(|s| {
                Rule {
                    specificity: s.specificity,
                    selector: s.compound_selectors,
                    declarations: Arc::new(~[]),
                    source_order: i,
                }
            }).to_owned_vec()
        }).to_owned_vec()
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
        assert_eq!(1, selector_map.id_hash.find(&~"top").unwrap()[0].source_order);
        selector_map.insert(rules_list[0][0].clone());
        assert_eq!(0, selector_map.class_hash.find(&~"intro").unwrap()[0].source_order);
        assert!(selector_map.class_hash.find(&~"foo").is_none());
    }
}
