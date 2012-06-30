#[doc="Performs CSS selector matching."]

import base::{LayoutData};
import dom::base;
import base::{ElementData, Node, Text};
import dom::style::{Selector, StyleDeclaration, FontSize, Display, TextColor, BackgroundColor,
                    Stylesheet, Element, Child, Descendant, Sibling, Attr, Exact, Exists, Includes,
                    StartsWith};
import dom::rcu::ReaderMethods;
import style::{computed_style, default_style_for_node_kind};

export matching_methods;

#[doc="Check if a CSS attribute matches the attribute of an HTML element."]
fn attrs_match(attr: Attr, elmt: ElementData) -> bool {
    alt attr {
      Exists(name) {
        alt elmt.get_attr(name) {
          some(_) { ret true; }
          none    { ret false; }
        }
      }
      Exact(name, val) {
        alt elmt.get_attr(name) {
          some(value) { ret value == val; }
          none        { ret false; }
        }
      }
      Includes(name, val) {
        // Comply with css spec, if the specified attribute is empty
        // it cannot match.
        if val == "" { ret false; }

        alt elmt.get_attr(name) {
          some(value) { ret value.split_char(' ').contains(val); }
          none        { ret false; }
        }
      }
      StartsWith(name, val) {
        alt elmt.get_attr(name) {
          some(value) { 
            //check that there is only one attribute value and it
            //starts with the perscribed value
            if !value.starts_with(val) || value.contains(" ") { ret false; }

            // We match on either the exact value or value-foo
            if value.len() == val.len() { ret true; }
            else { ret value.starts_with(val + "-"); }
          }
          none {
            ret false;
          }
        }
      }
    }
}

impl priv_matching_methods for Node {
    #[doc="
        Checks if the given CSS selector, which must describe a single element with no relational
        information, describes the given HTML element.
    "]
    fn matches_element(sel: ~Selector) -> bool {
        alt *sel {
          Child(_, _) | Descendant(_, _) | Sibling(_, _) { ret false; }
          Element(tag, attrs) {
            alt self.read { |n| copy *n.kind } {
                base::Element(elmt) {
                    if !(tag == "*" || tag == elmt.tag_name) {
                        ret false;
                    }
                    
                    let mut i = 0u;
                    while i < attrs.len() {
                        if !attrs_match(attrs[i], elmt) { ret false; }
                        i += 1u;
                    }

                    ret true;
                }
                Text(str)   { /*fall through, currently unsupported*/ }
            }
          }
        }

        ret false; //If we got this far it was because something was
                   //unsupported.
    }

    #[doc = "Checks if a generic CSS selector matches a given HTML element"]
    fn matches_selector(sel : ~Selector) -> bool {
        alt *sel {
          Element(str, atts) { ret self.matches_element(sel); }
          Child(sel1, sel2) {
            alt self.read { |n| n.tree.parent } {
              some(parent) { 
                ret self.matches_element(sel2) &&
                    parent.matches_selector(sel1);
              }
              none         { ret false; }
            }
          }
          Descendant(sel1, sel2) {
            if !self.matches_element(sel2) {
                ret false;
            }

            //loop over all ancestors to check if they are the person
            //we should be descended from.
            let mut cur_parent = alt self.read { |n| n.tree.parent } {
                some(parent) { parent }
                none         { ret false; }
            };

            loop {
                if cur_parent.matches_selector(sel1) { ret true; }

                cur_parent = alt cur_parent.read { |n| n.tree.parent } {
                    some(parent) { parent }
                    none         { ret false; }
                };
            }
          }
          Sibling(sel1, sel2) {
            if !self.matches_element(sel2) { ret false; }

            // Loop over this node's previous siblings to see if they match.
            alt self.read { |n| n.tree.prev_sibling } {
                some(sib) {
                    let mut cur_sib = sib;
                    loop {
                        if cur_sib.matches_selector(sel1) { ret true; }
                
                        cur_sib = alt cur_sib.read { |n| n.tree.prev_sibling } {
                            some(sib) { sib }
                            none      { break; }
                        };
                    }
                }
                none { }
            }

            // check the rest of the siblings
            alt self.read { |n| n.tree.next_sibling } {
                some(sib) {
                    let mut cur_sib = sib;
                    loop {
                        if cur_sib.matches_selector(sel1) { ret true; }
                
                        cur_sib = alt cur_sib.read { |n| n.tree.next_sibling } {
                            some(sib) { sib }
                            none      { break; }
                        };
                    }
                }
                none { }
            }

            ret false;
          }
        }
    }
}

impl priv_style_methods for Node {
    #[doc="Update the computed style of an HTML element with a style specified by CSS."]
    fn update_style(decl : StyleDeclaration) {
        self.aux() { |layout|
            alt decl {
              Display(dis)           { layout.computed_style.display = dis; }
              BackgroundColor(col)  { layout.computed_style.back_color = col; }
              TextColor(*) | FontSize(*)   { /* not supported yet */ } 
            }
        }
    }
}

impl matching_methods for Node {
    #[doc="Compare an html element to a list of css rules and update its
           style according to the rules matching it."]
    fn match_css_style(styles : Stylesheet) {
        // Loop over each rule, see if our node matches what is described in the rule. If it
        // matches, update its style. As we don't currently have priorities of style information,
        // the latest rule takes precedence over the others. So we just overwrite style
        // information as we go.

        for styles.each { |sty|
            let (selectors, decls) = copy *sty;
            for selectors.each { |sel|
                if self.matches_selector(sel) {
                    for decls.each { |decl| 
                        self.update_style(decl);
                    }
                }
            }
        }
        
        self.aux() { |a| #debug["Changed the style to: %?", copy *a.computed_style]; }
    }
}

#[cfg(test)]
mod test {
    import dom::base::{Attr, HTMLDivElement, HTMLHeadElement, HTMLImageElement};
    import dom::base::{NodeScope, TreeReadMethods, TreeWriteMethods, UnknownElement};
    import dvec::{dvec, extensions};
    import io::println;

    #[warn(no_non_implicitly_copyable_typarams)]
    fn new_node_from_attr(scope: NodeScope, -name: str, -val: str) -> Node {
        let elmt = ElementData("div", ~HTMLDivElement);
        let attr = ~Attr(name, val);
        elmt.attrs.push(attr);
        ret scope.new_node(base::Element(elmt));
    }

    #[test]
    fn test_match_pipe1() {
        let scope = NodeScope();
        let node = new_node_from_attr(scope, "lang", "en-us");

        let sel = Element("*", [StartsWith("lang", "en")]);

        assert node.matches_selector(~sel);
    }

    #[test]
    fn test_match_pipe2() {
        let scope = NodeScope();
        let node = new_node_from_attr(scope, "lang", "en");

        let sel = Element("*", [StartsWith("lang", "en")]);

        assert node.matches_selector(~sel);
    }
    
    #[test] 
    fn test_not_match_pipe() {
        let scope = NodeScope();
        let node = new_node_from_attr(scope, "lang", "english");

        let sel = Element("*", [StartsWith("lang", "en")]);

        assert !node.matches_selector(~sel);
    }

    #[test]
    fn test_match_includes() {
        let scope = NodeScope();
        let node = new_node_from_attr(scope, "mad", "hatter cobler cooper");

        let sel = Element("div", [Includes("mad", "hatter")]);

        assert node.matches_selector(~sel);
    }

    #[test]
    fn test_match_exists() {
        let scope = NodeScope();
        let node = new_node_from_attr(scope, "mad", "hatter cobler cooper");

        let sel1 = Element("div", [Exists("mad")]);
        let sel2 = Element("div", [Exists("hatter")]);

        assert node.matches_selector(~sel1);
        assert !node.matches_selector(~sel2);
    }

    #[test]
    fn test_match_exact() {
        let scope = NodeScope();
        let node1 = new_node_from_attr(scope, "mad", "hatter cobler cooper");
        let node2 = new_node_from_attr(scope, "mad", "hatter");

        let sel = Element("div", [Exact("mad", "hatter")]);

        assert !node1.matches_selector(~copy sel);
        assert node2.matches_selector(~sel);
    }

    #[test]
    fn match_tree() {
        let scope = NodeScope();

        let root = new_node_from_attr(scope, "class", "blue");
        let child1 = new_node_from_attr(scope, "id", "green");
        let child2 = new_node_from_attr(scope, "flag", "black");
        let gchild = new_node_from_attr(scope, "flag", "grey");
        let ggchild = new_node_from_attr(scope, "flag", "white");
        let gggchild = new_node_from_attr(scope, "flag", "purple");

        scope.add_child(root, child1);
        scope.add_child(root, child2);
        scope.add_child(child2, gchild);
        scope.add_child(gchild, ggchild);
        scope.add_child(ggchild, gggchild);

        let sel1 = Descendant(~Element("*", [Exact("class", "blue")]),
                              ~Element("*", []));

        assert !root.matches_selector(~copy sel1);
        assert child1.matches_selector(~copy sel1);
        assert child2.matches_selector(~copy sel1);
        assert gchild.matches_selector(~copy sel1);
        assert ggchild.matches_selector(~copy sel1);
        assert gggchild.matches_selector(~sel1);

        let sel2 = Descendant(~Child(~Element("*", [Exact("class", "blue")]),
                                     ~Element("*", [])),
                              ~Element("div", [Exists("flag")]));

        assert !root.matches_selector(~copy sel2);
        assert !child1.matches_selector(~copy sel2);
        assert !child2.matches_selector(~copy sel2);
        assert gchild.matches_selector(~copy sel2);
        assert ggchild.matches_selector(~copy sel2);
        assert gggchild.matches_selector(~sel2);

        let sel3 = Sibling(~Element("*", []), ~Element("*", []));

        assert !root.matches_selector(~copy sel3);
        assert child1.matches_selector(~copy sel3);
        assert child2.matches_selector(~copy sel3);
        assert !gchild.matches_selector(~copy sel3);
        assert !ggchild.matches_selector(~copy sel3);
        assert !gggchild.matches_selector(~sel3);

        let sel4 = Descendant(~Child(~Element("*", [Exists("class")]),
                                    ~Element("*", [])),
                              ~Element("*", []));

        assert !root.matches_selector(~copy sel4);
        assert !child1.matches_selector(~copy sel4);
        assert !child2.matches_selector(~copy sel4);
        assert gchild.matches_selector(~copy sel4);
        assert ggchild.matches_selector(~copy sel4);
        assert gggchild.matches_selector(~sel4);
    }
}
