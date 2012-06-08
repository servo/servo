import io::println;


enum display_type{
    di_block,
    di_inline,
    di_none
}

enum style_decl{
    font_size(uint), // Currently assumes format '# pt'
    display(display_type),
    text_color(uint),
    background_color(uint)
}

enum attr{
    exists(str),
    exact(str, str),
    includes(str, str),
    starts_with(str, str)
}
    
enum selector{
    element(str, [attr]),
    child(~selector, ~selector),
    descendant(~selector, ~selector),
    sibling(~selector, ~selector)
}

type rule = ([~selector], [style_decl]);

type stylesheet = [~rule];

fn print_list<T>(list : [T], print : fn(T) -> str) -> str {
    let l = vec::len(list);
    if l == 0u { ret "" }

    let mut res = print(list[0]);
    let mut i = 1u;
    
    while i < l { 
        res += ", ";
        res += print(list[i]);
        i += 1u;
    }
    
    ret res;
} 

fn print_list_vert<T>(list : [T], print : fn(T) -> str) -> str {
    let l = vec::len(list);
    if l == 0u { ret "" }

    let mut res = "-";
    res += print(list[0]);
    let mut i = 1u;
    
    while i < l { 
        res += "\n-";
        res += print(list[i]);
        i += 1u;
    }
    
    ret res;
} 

fn print_display(dis_ty : display_type) -> str {
    alt dis_ty { 
      di_block  { "block" } 
      di_inline { "inline" }
      di_none   { "none" }
    }
}

fn print_style(decl : style_decl) -> str{
    alt decl {
      font_size(s) { #fmt("Font size = %u pt", s) }
      display(dis_ty) { #fmt("Display style = %s", print_display(dis_ty)) }
      text_color(c) { #fmt("Text color = 0x%06x", c) }
      background_color(c) { #fmt("Background color = 0x%06x", c) }
    }
}

fn print_attr(attribute : attr) -> str {
    alt attribute {
      exists(att) { #fmt("[%s]", att) }
      exact(att, val) { #fmt("[%s = %s]", att, val) }
      includes(att, val) { #fmt("[%s ~= %s]", att, val) }
      starts_with(att, val) { #fmt("[%s |= %s]", att, val) }
    }
}

fn print_selector(&&select : ~selector) -> str {
    alt *select {
      element(s, attrs) { #fmt("Element %s with attributes: %s", s, 
                               print_list(attrs, print_attr)) }
      child(sel1, sel2) { #fmt("(%s) > (%s)", print_selector(sel1),
                               print_selector(sel2)) }
      descendant(sel1, sel2) { #fmt("(%s) (%s)", print_selector(sel1),
                                    print_selector(sel2)) }
      sibling(sel1, sel2) { #fmt("(%s) + (%s)", print_selector(sel1),
                                    print_selector(sel2)) }
    }
}

fn print_rule(&&rule : ~rule) -> str {
    alt *rule {
      (sels, styles) {
        let sel_str = print_list(sels, print_selector);
        let sty_str = print_list(styles, print_style);        
        
        #fmt("Selectors: %s; Style: {%s}", sel_str, sty_str)
      }
    }
}

fn print_sheet(sheet : stylesheet) -> str {
    #fmt("CSS Rules:\n%s", print_list_vert(sheet, print_rule))
}

#[test]
fn test_pretty_print() {
    let test1 = [~([~element("p", [])], [font_size(32u)])];
    let actual1 = print_sheet(test1);
    let expected1 = "CSS Rules:\n-Selectors: Element p with attributes: ;" +
        " Style: {Font size = 32 pt}";

    assert(actual1 == expected1);

    let elmt1 = ~element("*", []);
    let elmt2 = ~element("body", [exact("class", "2")]);

    let test2 = [~([~descendant(elmt1, elmt2)],
                  [display(di_block), text_color(0u)])];

    let actual2 = print_sheet(test2);
    let expected2 =  "CSS Rules:\n-Selectors: (Element * with attributes: ) "
        + "(Element body with attributes: [class = 2]); " + 
        "Style: {Display style = block, Text color = 0x000000}";

    assert(actual2 == expected2);
}
