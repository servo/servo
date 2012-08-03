#[doc="Constructs a list of css style rules from a token stream"]

// TODO: fail according to the css spec instead of failing when things
// are not as expected

import dom::style;
import style::{DisInline, DisBlock, DisNone, Display, TextColor, BackgroundColor, FontSize,
               Height, Width, StyleDeclaration, Selector};
import parser::css_lexer::{Token, StartDescription, EndDescription,
                           Descendant, Child, Sibling,
                           Comma, Element, Attr, Description,
                           Eof};
import comm::recv;
import option::{map, is_none};
import vec::push;
import parser::parser_util::{parse_display_type, parse_font_size, parse_size};
import util::color::parsing::parse_color;
import vec::push;

type TokenReader = {stream : pipes::port<Token>, mut lookahead : option<Token>};

trait util_methods {
    fn get() -> Token;
    fn unget(-tok : Token);
}

impl util_methods of util_methods for TokenReader {
    fn get() -> Token {
        alt copy self.lookahead {
          some(tok) => { self.lookahead = none; copy tok }
          none => { self.stream.recv() }
        }
    }

    fn unget(-tok : Token) {
        assert is_none(self.lookahead);
        self.lookahead = some(tok);
    }
}

trait parser_methods {
    fn parse_element() -> option<~style::Selector>;
    fn parse_selector() -> option<~[~Selector]>;
    fn parse_description() -> option<~[StyleDeclaration]>;
    fn parse_rule() -> option<~style::Rule>;
}

impl parser_methods of parser_methods for TokenReader {
    fn parse_element() -> option<~style::Selector> {
        // Get the current element type
         let elmt_name = alt self.get() {
           Element(tag) => { copy tag }
           Eof => { return none; }
           _ => { fail ~"Expected an element" }
         };

         let mut attr_list = ~[];

         // Get the attributes associated with that element
         loop {
             let tok = self.get();
             alt tok {
               Attr(attr) => { push(attr_list, copy attr); }
               StartDescription | Descendant | Child | Sibling | Comma => {
                 self.unget(tok); 
                 break;
               }
               Eof => { return none; }
               Element(_) => fail ~"Unexpected second element without relation to first element",
               EndDescription => fail ~"Unexpected '}'",
               Description(_, _) => fail ~"Unexpected description"
             }
         }
        return some(~style::Element(elmt_name, attr_list));
    }

    fn parse_selector() -> option<~[~Selector]> {
        let mut sel_list = ~[];

        // Collect all the selectors that this rule applies to
        loop {
            let mut cur_sel;

            alt self.parse_element() {
              some(elmt) => { cur_sel = copy elmt; }
              none => { return none; } // we hit an eof in the middle of a rule
            }

            loop {
                let tok = self.get();
                let built_sel <- cur_sel;
                
                alt tok {
                  Descendant => {
                    alt self.parse_element() {
                      some(elmt) => { 
                        let new_sel = copy elmt;
                        cur_sel <- ~style::Descendant(built_sel, new_sel)
                      }
                      none => { return none; }
                    }
                  }
                  Child => {
                    alt self.parse_element() {
                      some(elmt) => { 
                        let new_sel = copy elmt;
                        cur_sel <- ~style::Child(built_sel, new_sel)
                      }
                      none => { return none; }
                    }
                  }
                  Sibling => {
                    alt self.parse_element() {
                      some(elmt) => { 
                        let new_sel = copy elmt;
                        cur_sel <- ~style::Sibling(built_sel, new_sel)
                      }
                      none => { return none; }
                    }
                  }
                  StartDescription => {
                    push(sel_list, built_sel);
                    self.unget(StartDescription);
                    break;
                  }
                  Comma => {
                    push(sel_list, built_sel);
                    self.unget(Comma);
                    break;
                  }
                  Attr(_) | EndDescription | Element(_) | Description(_, _) => {
                    fail #fmt["Unexpected token %? in elements", tok];
                  }
                  Eof => { return none; }
                }
            }

            // check if we should break out of the nesting loop as well
            // TODO: fix this when rust gets labelled loops
            let tok = self.get();
            alt tok {
              StartDescription => { break; }
              Comma => { }
              _ => { self.unget(tok); }
            }
        }
        
        return some(sel_list);
    }

    fn parse_description() -> option<~[StyleDeclaration]> {
        let mut desc_list : ~[StyleDeclaration]= ~[];
        
        // Get the description to be applied to the selector
        loop {
            let tok = self.get();
            alt tok {
              EndDescription => { break; }
              Description(prop, val) => {
                let desc = alt prop {
                  // TODO: have color parsing return an option instead of a real value
                  ~"background-color" => parse_color(val).map(|res| BackgroundColor(res)),
                  ~"color" => parse_color(val).map(|res| TextColor(res)),
                  ~"display" => parse_display_type(val).map(|res| Display(res)),
                  ~"font-size" => parse_font_size(val).map(|res| FontSize(res)),
                  ~"height" => parse_size(val).map(|res| Height(res)),
                  ~"width" => parse_size(val).map(|res| Width(res)),
                  _ => { #debug["Recieved unknown style property '%s'", val]; none }
                };
                desc.map(|res| push(desc_list, res));
              }
              Eof => { return none; }
              StartDescription | Descendant | Child | Sibling | Comma | Element(_) | Attr(_) => {
                fail #fmt["Unexpected token %? in description", tok]; 
              }
            }
        }
        
        return some(desc_list);
    }

    fn parse_rule() -> option<~style::Rule> {
        // TODO: get rid of copies once alt move works
        let sel_list = alt self.parse_selector() {
          some(list) => { copy list }
          none => { return none; }
        };

        #debug("sel_list: %?", sel_list);
        
        // Get the description to be applied to the selector
        let desc_list = alt self.parse_description() {
          some(list) => { copy list }
          none => { return none; }
        };

        #debug("desc_list: %?", desc_list);
        
        return some(~(sel_list, desc_list));
    }
}

fn build_stylesheet(+stream : pipes::port<Token>) -> ~[~style::Rule] {
    let mut rule_list = ~[];
    let reader = {stream : stream, mut lookahead : none};

    loop {
        alt reader.parse_rule() {
          some(rule) => { push(rule_list, copy rule); }
          none => { break; }
        }
    }

    return rule_list;
}
