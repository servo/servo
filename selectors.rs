use cssparser::*;


pub struct Selector {
    compound_selectors: CompoundSelector,
    pseudo_element: PseudoElement,
    specificity: u32,
}

pub enum PseudoElement {
    Element, // No pseudo-element
    Before,
    After,
    FirstLine,
    FirstLetter,
}


pub struct CompoundSelector {
    simple_selectors: ~[SimpleSelector],
    next: Option<(~CompoundSelector, Combinator)>,  // c.next is left of c
}

pub enum Combinator {
    Child,  //  >
    Descendant,  // space
    NextSibling,  // +
    LaterSibling,  // ~
}

pub enum SimpleSelector {
    IDSelector(~str),
    ClassSelector(~str),
    LocalNameSelector{lowercase_name: ~str, cased_name: ~str},
//    NamespaceSelector(Namespace)

    // Attribute selectors
    AttrExists(AttrSelector),  // [foo]
    AttrEqual(AttrSelector, ~str),  // [foo=bar]
    AttrIncludes(AttrSelector, ~str),  // [foo~=bar]
    AttrDashMatch(AttrSelector, ~str),  // [foo|=bar]
    AttrPrefixMatch(AttrSelector, ~str),  // [foo^=bar]
    AttrSubstringMatch(AttrSelector, ~str),  // [foo*=bar]
    AttrSuffixMatch(AttrSelector, ~str),  // [foo$=bar]

    // Pseudo-classes
//    Empty,
//    Root,
//    Lang(~str),
//    NthChild(u32, u32),
//    NthLastChild(u32, u32),
//    NthOfType(u32, u32),
//    NthLastOfType(u32, u32),
//    Lang(~str),
//    Negation(~Selector),
    // ...
}

pub struct AttrSelector {
    lowercase_name: ~str,
    cased_name: ~str,
//    namespace: Option<~str>,
}


pub fn parse_selector_list(input: &[ComponentValue]) -> Option<~[Selector]> {
    let len = input.len();
    let (first, pos) = match parse_selector(input, 0) {
        None => return None,
        Some(result) => result
    };
    let mut results = ~[first];
    let mut pos = pos;

    loop {
        pos = skip_whitespace(input, pos);
        if pos >= len { break }  // EOF
        if input[pos] != Comma { return None }
        pos = skip_whitespace(input, pos);
        match parse_selector(input, pos) {
            None => return None,
            Some((selector, next_pos)) => {
                results.push(selector);
                pos = next_pos;
            }
        }
    }
    Some(results)
}


fn parse_selector(input: &[ComponentValue], pos: uint) -> Option<(Selector, uint)> {
    let len = input.len();
    let (first, pos) = match parse_simple_selectors(input, pos) {
        None => return None,
        Some(result) => result
    };
    let mut compound = CompoundSelector{ simple_selectors: first, next: None };
    let mut pos = pos;

    loop {
        let pre_whitespace_pos = pos;
        pos = skip_whitespace(input, pos);
        if pos >= len { break }  // EOF
        let combinator = match input[pos] {
            Delim('>') => { pos += 1; Child },
            Delim('+') => { pos += 1; NextSibling },
            Delim('~') => { pos += 1; LaterSibling },
            _ => {
                if pos > pre_whitespace_pos { Descendant }
                else { return None }
            }
        };
        pos = skip_whitespace(input, pos);
        match parse_simple_selectors(input, pos) {
            None => return None,
            Some((simple_selectors, next_pos)) => {
                compound = CompoundSelector {
                    simple_selectors: simple_selectors,
                    next: Some((~compound, combinator))
                };
                pos = next_pos;
            }
        }
    }
    let selector = Selector{
        compound_selectors: compound,
        pseudo_element: Element,
        specificity: 0, // TODO
    };
    Some((selector, pos))
}


fn parse_simple_selectors(input: &[ComponentValue], pos: uint)
                           -> Option<(~[SimpleSelector], uint)> {
    let _ = input;
    let _ = pos;
    None  // TODO
}


#[inline]
fn skip_whitespace(input: &[ComponentValue], mut pos: uint) -> uint {
    let len = input.len();
    while pos < len {
        if input[pos] == WhiteSpace { break }
        pos += 1;
    }
    pos
}
