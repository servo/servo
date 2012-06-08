#[doc="Perform css selector matching"]

import dom::base::{node, nk_element, nk_text};
import dom::style::{selector, style_decl, font_size, display, text_color,
                    background_color, stylesheet, element, child, descendant,
                    sibling, attr, exact, exists, includes, starts_with};
import dom::rcu::{reader_methods};
import style::{computed_style, default_style_for_node_kind};
import base::{layout_data};

export matching_methods;

#[doc="Update the computed style of an html element with a style specified
       by css."]
fn update_style(style : @computed_style, decl : style_decl) {
    alt decl {
      display(dis)           { (*style).display = dis; }
      background_color(col)  { (*style).back_color = col; }
      text_color(*) | font_size(*)   { /* not supported yet */ } 
    }
}

#[doc="Check if a css attribute matches the attribute of an html element."]
fn attrs_match(attr : attr, elmt : dom::base::element) -> bool {
    alt attr {
      exists(name) {
        alt elmt.get_attr(name) {
          some(_) { ret true; }
          none    { ret false; }
        }
      }
      exact(name, val) {
        alt elmt.get_attr(name) {
          some(value) { ret value == val; }
          none        { ret false; }
        }
      }
      includes(name, val) {
        // Comply with css spec, if the specified attribute is empty
        // it cannot match.
        if val == "" { ret false; }

        alt elmt.get_attr(name) {
          some(value) { ret value.split_char(' ').contains(val); }
          none        { ret false; }
        }
      }
      starts_with(name, val) {
        alt elmt.get_attr(name) {
          some(value) { 
            //check that there is only one attribute value and it
            //starts with the perscribed value
            if !value.starts_with(val) || value.contains(" ") { ret false; }

            // We match on either the exact value or value-foo
            if value.len() == val.len() { ret true; }
            else { ret value.starts_with(val + "-"); }
          }
          none       { ret false; }
        }
      }
    }
}

impl priv_matching_methods for node {
    #[doc="Checks if the given css selector, which must describe a single
           element with no relational information, describes the given
           html element."]
    fn matches_element(sel : ~selector) -> bool {
        alt *sel {
          child(_, _) | descendant(_, _) | sibling(_, _) { ret false; }
          element(tag, attrs) {
            alt self.rd { |n| copy *n.kind } {
                nk_element(elmt) {
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
                nk_text(str)   { /*fall through, currently unsupported*/ }
            }
          }
        }

        ret false; //If we got this far it was because something was
                   //unsupported.
    }

    #[doc = "Checks if a generic css selector matches a given html element"]
    fn matches_selector(sel : ~selector) -> bool {
        alt *sel {
          element(str, atts) { ret self.matches_element(sel); }
          child(sel1, sel2) {
            alt self.rd { |n| n.tree.parent } {
              some(parent) { 
                ret self.matches_element(sel2) &&
                    parent.matches_selector(sel1);
              }
              none         { ret false; }
            }
          }
          descendant(sel1, sel2) {
            if !self.matches_element(sel2) {
                ret false;
            }

            //loop over all ancestors to check if they are the person
            //we should be descended from.
            let mut cur_parent = alt self.rd { |n| n.tree.parent } {
                some(parent) { parent }
                none         { ret false; }
            };

            loop {
                if cur_parent.matches_selector(sel1) { ret true; }

                cur_parent = alt cur_parent.rd { |n| n.tree.parent } {
                    some(parent) { parent }
                    none         { ret false; }
                };
            }
          }
          sibling(sel1, sel2) {
            if !self.matches_element(sel2) { ret false; }

            // loop over this node's previous siblings to see if they
            // match
            alt self.rd { |n| n.tree.prev_sibling } {
                some(sib) {
                    let mut cur_sib = sib;
                    loop {
                        if cur_sib.matches_selector(sel1) { ret true; }
                
                        cur_sib = alt cur_sib.rd { |n| n.tree.prev_sibling } {
                            some(sib) { sib }
                            none      { break; }
                        };
                    }
                }
                none { }
            }

            // check the rest of the siblings
            alt self.rd { |n| n.tree.next_sibling } {
                some(sib) {
                    let mut cur_sib = sib;
                    loop {
                        if cur_sib.matches_selector(sel1) { ret true; }
                
                        cur_sib = alt cur_sib.rd { |n| n.tree.next_sibling } {
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

impl matching_methods for node {
    #[doc="Compare an html element to a list of css rules and update its
           style according to the rules matching it."]
    fn match_css_style(styles : stylesheet) -> computed_style {
        let node_kind = self.rd { |n| copy *n.kind };
        let style = 
            @default_style_for_node_kind(node_kind);

        // Loop over each rule, see if our node matches what is
        // described in the rule.  If it matches, update its style.
        // As we don't currently have priorities of style information,
        // the latest rule takes precedence so we can just overwrite
        // style information.
        for styles.each { |sty|
            let (selectors, decls) <- *(copy sty);
            for selectors.each { |sel|
                if self.matches_selector(sel) {
                    #debug("Matched selector {%?} with node {%?}",
                           *sel, node_kind);
                    for decls.each { |decl| 
                        update_style(style, decl);
                    }
                }
            }
        }

        #debug["Changed the style to: %?", *style];

        ret copy *(style);
    }
}

mod test {
    import dom::base::{node_scope, methods, nk_element, attr, es_div,
                       es_img, es_unknown, es_head, wr_tree_ops};
    import dvec::{dvec, extensions};
    import io::println;

    fn new_node_from_attr(scope : node_scope, name : str, val : str) -> node {
        let elmt = dom::base::element("div", ~es_div);
        let attr = ~attr(name, val);
        elmt.attrs.push(copy attr);
        ret scope.new_node(nk_element(elmt));        
    }

    #[test]
    fn test_match_pipe1() {
        let scope = node_scope();
        let node = new_node_from_attr(scope, "lang", "en-us");

        let sel = element("*", [starts_with("lang", "en")]);

        assert node.matches_selector(~sel);        
    }

    #[test]
    fn test_match_pipe2() {
        let scope = node_scope();
        let node = new_node_from_attr(scope, "lang", "en");

        let sel = element("*", [starts_with("lang", "en")]);

        assert node.matches_selector(~sel);        
    }
    
    #[test] 
    fn test_not_match_pipe() {
        let scope = node_scope();
        let node = new_node_from_attr(scope, "lang", "english");

        let sel = element("*", [starts_with("lang", "en")]);

        assert !node.matches_selector(~sel);        
    }

    #[test]
    fn test_match_includes() {
        let scope = node_scope();
        let node = new_node_from_attr(scope, "mad", "hatter cobler cooper");

        let sel = element("div", [includes("mad", "hatter")]);

        assert node.matches_selector(~sel);
    }

    #[test]
    fn test_match_exists() {
        let scope = node_scope();
        let node = new_node_from_attr(scope, "mad", "hatter cobler cooper");

        let sel1 = element("div", [exists("mad")]);
        let sel2 = element("div", [exists("hatter")]);

        assert node.matches_selector(~sel1);
        assert !node.matches_selector(~sel2);
    }

    #[test]
    fn test_match_exact() {
        let scope = node_scope();
        let node1 = new_node_from_attr(scope, "mad", "hatter cobler cooper");
        let node2 = new_node_from_attr(scope, "mad", "hatter");

        let sel = element("div", [exact("mad", "hatter")]);

        assert !node1.matches_selector(~copy sel);
        assert node2.matches_selector(~sel);
    }

    #[test]
    fn match_tree() {
        let scope = node_scope();

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

        let sel1 = descendant(~element("*", [exact("class", "blue")]),
                              ~element("*", []));

        assert !root.matches_selector(~copy sel1);
        assert child1.matches_selector(~copy sel1);
        assert child2.matches_selector(~copy sel1);
        assert gchild.matches_selector(~copy sel1);
        assert ggchild.matches_selector(~copy sel1);
        assert gggchild.matches_selector(~sel1);

        let sel2 = descendant(~child(~element("*", [exact("class", "blue")]),
                                     ~element("*", [])),
                              ~element("div", [exists("flag")]));

        assert !root.matches_selector(~copy sel2);
        assert !child1.matches_selector(~copy sel2);
        assert !child2.matches_selector(~copy sel2);
        assert gchild.matches_selector(~copy sel2);
        assert ggchild.matches_selector(~copy sel2);
        assert gggchild.matches_selector(~sel2);

        let sel3 = sibling(~element("*", []), ~element("*", []));

        assert !root.matches_selector(~copy sel3);
        assert child1.matches_selector(~copy sel3);
        assert child2.matches_selector(~copy sel3);
        assert !gchild.matches_selector(~copy sel3);
        assert !ggchild.matches_selector(~copy sel3);
        assert !gggchild.matches_selector(~sel3);

        let sel4 = descendant(~child(~element("*", [exists("class")]),
                                    ~element("*", [])),
                              ~element("*", []));

        assert !root.matches_selector(~copy sel4);
        assert !child1.matches_selector(~copy sel4);
        assert !child2.matches_selector(~copy sel4);
        assert gchild.matches_selector(~copy sel4);
        assert ggchild.matches_selector(~copy sel4);
        assert gggchild.matches_selector(~sel4);
    }
}
