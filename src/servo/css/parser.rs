/**
Constructs a list of css style rules from a token stream
*/

// TODO: fail according to the css spec instead of failing when things
// are not as expected

use newcss::values::*;
// Disambiguate parsed Selector, Rule values from tokens
use css = newcss::values;
use tok = lexer;
use lexer::Token;
use comm::recv;
use option::{map, is_none};
use vec::push;
use parser_util::*;
use newcss::color::parsing::parse_color;
use vec::push;

type TokenReader = {stream : pipes::Port<Token>, mut lookahead : Option<Token>};

trait TokenReaderMethods {
    fn get() -> Token;
    fn unget(+tok : Token);
}

impl TokenReader : TokenReaderMethods {
    fn get() -> Token {
        match copy self.lookahead {
          Some(tok) => { self.lookahead = None; copy tok }
          None => { self.stream.recv() }
        }
    }

    fn unget(tok : Token) {
        assert is_none(&self.lookahead);
        self.lookahead = Some(move tok);
    }
}

trait ParserMethods {
    fn parse_element() -> Option<~css::Selector>;
    fn parse_selector() -> Option<~[~css::Selector]>;
    fn parse_description() -> Option<~[StyleDeclaration]>;
    fn parse_rule() -> Option<~css::Rule>;
}

impl TokenReader : ParserMethods {
    fn parse_element() -> Option<~css::Selector> {
        // Get the current element type
         let elmt_name = match self.get() {
           lexer::Element(tag) => { copy tag }
           lexer::Eof => { return None; }
           _ => { fail ~"Expected an element" }
         };

         let mut attr_list = ~[];

         // Get the attributes associated with that element
         loop {
             let token = self.get();
             match token {
               lexer::Attr(attr) => { push(&mut attr_list, copy attr); }
               tok::StartDescription | tok::Descendant | tok::Child | tok::Sibling | tok::Comma => {
                 self.unget(move token); 
                 break;
               }
               tok::Eof => { return None; }
               tok::Element(_) => fail ~"Unexpected second element without relation to first element",
               tok::EndDescription => fail ~"Unexpected '}'",
               tok::Description(_, _) => fail ~"Unexpected description"
             }
         }
        return Some(~css::Element(move elmt_name, move attr_list));
    }

    fn parse_selector() -> Option<~[~css::Selector]> {
        let mut sel_list = ~[];

        // Collect all the selectors that this rule applies to
        loop {
            let mut cur_sel;

            match self.parse_element() {
              Some(elmt) => { cur_sel = copy elmt; }
              None => { return None; } // we hit an eof in the middle of a rule
            }

            loop {
                let tok = self.get();
                let built_sel = move cur_sel;
                
                match tok {
                  tok::Descendant => {
                    match self.parse_element() {
                      Some(elmt) => { 
                        let new_sel = copy elmt;
                        cur_sel = ~css::Descendant(move built_sel, move new_sel)
                      }
                      None => { return None; }
                    }
                  }
                  tok::Child => {
                    match self.parse_element() {
                      Some(elmt) => { 
                        let new_sel = copy elmt;
                        cur_sel = ~css::Child(move built_sel, move new_sel)
                      }
                      None => { return None; }
                    }
                  }
                  tok::Sibling => {
                    match self.parse_element() {
                      Some(elmt) => { 
                        let new_sel = copy elmt;
                        cur_sel = ~css::Sibling(move built_sel, move new_sel)
                      }
                      None => { return None; }
                    }
                  }
                  tok::StartDescription => {
                    push(&mut sel_list, move built_sel);
                    self.unget(tok::StartDescription);
                    break;
                  }
                  tok::Comma => {
                    push(&mut sel_list, move built_sel);
                    self.unget(tok::Comma);
                    break;
                  }
                  tok::Attr(_) | tok::EndDescription | tok::Element(_) | tok::Description(_, _) => {
                    fail #fmt["Unexpected token %? in elements", tok];
                  }
                  tok::Eof => { return None; }
                }
            }

            // check if we should break out of the nesting loop as well
            // TODO: fix this when rust gets labelled loops
            let tok = self.get();
            match tok {
              tok::StartDescription => { break; }
              tok::Comma => { }
              _ => { self.unget(move tok); }
            }
        }
        
        return Some(move sel_list);
    }

    fn parse_description() -> Option<~[StyleDeclaration]> {
        let mut desc_list : ~[StyleDeclaration]= ~[];
        
        // Get the description to be applied to the selector
        loop {
            let tok = self.get();
            match tok {
              tok::EndDescription => { break; }
              tok::Description(prop, val) => {
                let desc : Option<StyleDeclaration> = match prop {
                  // TODO: have color parsing return a ParseResult instead of a real value
                  ~"background-color" => parse_color(val).map(|res| BackgroundColor(Specified(BgColor(*res)))),
                  ~"color" => parse_color(val).map(|res| Color(Specified(TextColor(*res)))),
                  ~"display" => parse_display_type(val).extract(|res| Display(res)),
                  ~"font-size" => parse_font_size(val).extract(|res| FontSize(res)),
                  ~"height" => parse_box_sizing(val).extract(|res| Height(res)),
                  ~"width" => parse_box_sizing(val).extract(|res| Width(res)),
                    ~"border-width" => parse_length(val).map(|res| BorderWidth(Specified(*res))),
                    ~"border-color" => parse_color(val).map(|res| BorderColor(Specified(BdrColor(*res)))),
                    ~"position" => parse_position(val).extract(|res| Position(res)),
                    ~"top" => parse_length(val).map(|res| Top(Specified(*res))),
                    ~"right" => parse_length(val).map(|res| Right(Specified(*res))),
                    ~"bottom" => parse_length(val).map(|res| Bottom(Specified(*res))),
                    ~"left" => parse_length(val).map(|res| Left(Specified(*res))),
                  _ => { #debug["Recieved unknown style property '%s'", val]; None }
                };
                match desc {
                  Some(d) => push(&mut desc_list, d),
                  None => { #debug["Couldn't parse value '%s' for property '%s'", val, prop] }
                }
              }
              tok::Eof => { return None; }
              tok::StartDescription | tok::Descendant |  tok::Child | tok::Sibling 
              | tok::Comma | tok::Element(_) | tok::Attr(_) => {
                fail #fmt["Unexpected token %? in description", tok]; 
              }
            }
        }
        
        return Some(move desc_list);
    }

    fn parse_rule() -> Option<~css::Rule> {
        // TODO: get rid of copies once match move works
        let sel_list = match self.parse_selector() {
          Some(list) => { copy list }
          None => { return None; }
        };

        #debug("sel_list: %?", sel_list);
        
        // Get the description to be applied to the selector
        let desc_list = match self.parse_description() {
          Some(list) => { copy list }
          None => { return None; }
        };

        #debug("desc_list: %?", desc_list);
        
        return Some(~(move sel_list, move desc_list));
    }
}

pub fn build_stylesheet(stream : pipes::Port<Token>) -> ~[~css::Rule] {
    let mut rule_list = ~[];
    let reader = {stream : move stream, mut lookahead : None};

    loop {
        match reader.parse_rule() {
          Some(rule) => { push(&mut rule_list, copy rule); }
          None => { break; }
        }
    }

    return move rule_list;
}
