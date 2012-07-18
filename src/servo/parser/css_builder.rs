#[doc="Constructs a list of style rules from a token stream"]

// TODO: fail according to the css spec instead of failing when things
// are not as expected

import dom::style;
import style::{DisInline, DisBlock, DisNone, Display, TextColor, BackgroundColor, FontSize};
import parser::css_lexer::{Token, StartDescription, EndDescription,
                           Descendant, Child, Sibling,
                           Comma, Element, Attr, Description,
                           Eof};
import comm::recv;
import option::is_none;
import util::color::parsing::parse_color;
import vec::push;

type TokenReader = {stream : port<Token>, mut lookahead : option<Token>};

impl methods for TokenReader {
    fn get() -> Token {
        alt copy self.lookahead {
          some(tok)  { self.lookahead = none; copy tok }
          none       { recv(self.stream) }
        }
    }

    fn unget(-tok : Token) {
        assert is_none(self.lookahead);
        self.lookahead = some(tok);
    }
}

fn parse_element(reader : TokenReader) -> option<~style::Selector> {
    // Get the current element type
    let elmt_name = alt reader.get() {
      Element(tag)  { copy tag }
      Eof  { ret none; }
      _  { fail ~"Expected an element" }
    };

    let mut attr_list = ~[];

    // Get the attributes associated with that element
    loop {
        let tok = reader.get();
        alt tok {
          Attr(attr)       { push(attr_list, copy attr); }
          StartDescription | Descendant | Child | Sibling | Comma {
            reader.unget(tok); 
            break;
          }
          Eof              { ret none; }          
          Element(_)          { fail ~"Unexpected second element without "
                                   + ~"relation to first element"; }
          EndDescription         { fail ~"Unexpected '}'"; }
          Description(_, _)       { fail ~"Unexpected description"; }
        }
    }
        
    ret some(~style::Element(elmt_name, attr_list));
}

fn parse_rule(reader : TokenReader) -> option<~style::Rule> {
    let mut sel_list = ~[];
    let mut desc_list = ~[];

    // Collect all the selectors that this rule applies to
    loop {
        let mut cur_sel;

        alt parse_element(reader) {
          some(elmt)  { cur_sel = copy elmt; }
          none        { ret none; } // we hit an eof in the middle of a rule
        }

        loop {
            let tok = reader.get();
            alt tok {
              Descendant {
                alt parse_element(reader) {
                  some(elmt)   { 
                    let built_sel <- cur_sel;
                    let new_sel = copy elmt;
                    cur_sel <- ~style::Descendant(built_sel, new_sel)
                  }
                  none         { ret none; }
                }
              }
              Child {
                alt parse_element(reader) {
                  some(elmt)   { 
                    let built_sel <- cur_sel;
                    let new_sel = copy elmt;
                    cur_sel <- ~style::Child(built_sel, new_sel)
                  }
                  none         { ret none; }
                }
              }
              Sibling {
                alt parse_element(reader) {
                  some(elmt)   { 
                    let built_sel <- cur_sel;
                    let new_sel = copy elmt;
                    cur_sel <- ~style::Sibling(built_sel, new_sel)
                  }
                  none         { ret none; }
                }
              }
              StartDescription {
                let built_sel <- cur_sel; 
                push(sel_list, built_sel);
                reader.unget(StartDescription);
                break;
              }
              Comma      {
                let built_sel <- cur_sel;
                push(sel_list, built_sel);
                reader.unget(Comma);
                break;
              }
              Attr(_) | EndDescription | Element(_) | Description(_, _) {
                fail #fmt["Unexpected token %? in elements", tok];
              }
              Eof        { ret none; }
            }
        }

        // check if we should break out of the nesting loop as well
        let tok = reader.get();
        alt tok {
          StartDescription { break; }
          Comma      { }
          _             { reader.unget(tok); }
        }
    }
    
    // Get the description to be applied to the selector
    loop {
        let tok = reader.get();
        alt tok {
          EndDescription   { break; }
          Description(prop, val) {
            alt prop {
              ~"font-size" {
                // TODO, support more ways to declare a font size than # pt
                assert val.ends_with(~"pt");
                let num = val.substr(0u, val.len() - 2u);
                
                alt uint::from_str(num) {
                  some(n)    { push(desc_list, FontSize(n)); }
                  none       { fail ~"Nonnumber provided as font size"; }
                }
              }
              ~"display" {
                alt val {
                  ~"inline"   { push(desc_list, Display(DisInline)); }
                  ~"block"    { push(desc_list, Display(DisBlock)); }
                  ~"none"     { push(desc_list, Display(DisNone)); }
                  _          { #debug["Recieved unknown display value '%s'", val]; }
                }
              }
              ~"color" {
                push(desc_list, TextColor(parse_color(val)));
              }
              ~"background-color" {
                push(desc_list, BackgroundColor(parse_color(val)));
              }
              _ { #debug["Recieved unknown style property '%s'", val]; }
            }
          }
          Eof        { ret none; }
          StartDescription | Descendant | Child | Sibling
          | Comma | Element(_) | Attr(_)  {
            fail #fmt["Unexpected token %? in description", tok]; 
          }
        }
    }

    ret some(~(sel_list, desc_list));
}

fn build_stylesheet(stream : port<Token>) -> ~[~style::Rule] {
    let mut rule_list = ~[];
    let reader = {stream : stream, mut lookahead : none};

    loop {
        alt parse_rule(reader) {
          some(rule)   { push(rule_list, copy rule); }
          none         { break; }
        }
    }

    ret rule_list;
}
