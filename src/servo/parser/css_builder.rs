#[doc="Constructs a list of style rules from a token stream"]

// TODO: fail according to the css spec instead of failing when things
// are not as expected

import dom::style::*;
import parser::lexer::css::{token, to_start_desc, to_end_desc,
                            to_descendant, to_child, to_sibling,
                            to_comma, to_elmt, to_attr, to_desc,
                            to_eof};
import comm::recv;
import option::is_none;

type token_reader = {stream : port<token>, mut lookahead : option<token>};

impl methods for token_reader {
    fn get() -> token {
        alt copy self.lookahead {
          some(tok)  { self.lookahead = none; tok }
          none       { recv(self.stream) }
        }
    }

    fn unget(tok : token) {
        assert is_none(self.lookahead);
        self.lookahead = some(tok);
    }
}

fn parse_element(reader : token_reader) -> option<~selector> {
    // Get the current element type
    let elmt_name = alt reader.get() {
      to_elmt(tag)  { tag }
      to_eof        { ret none; }
      _             { fail "Expected an element" }
    };

    let mut attr_list = [];

    // Get the attributes associated with that element
    loop {
        let tok = reader.get();
        alt tok {
          to_attr(attr)       { attr_list += [attr]; }
          to_start_desc | to_descendant | to_child | to_sibling 
          | to_comma {
            reader.unget(tok); 
            break;
          }
          to_eof              { ret none; }          
          to_elmt(_)          { fail "Unexpected second element without " +
                                   "relation to first element"; }
          to_end_desc         { fail "Unexpected '}'"; }
          to_desc(_, _)       { fail "Unexpected description"; }
        }
    }
        
    ret some(~element(elmt_name, attr_list));
}

// Currently colors are supported in rgb(a,b,c) form and also by
// keywords for several common colors.
// TODO: extend this
fn parse_color(color : str) -> uint {
    let blue_unit = 1u;
    let green_unit = 256u;
    let red_unit = 256u * 256u;
    
    let result_color = if color.starts_with("rgb(") {
        let color_vec = str::bytes(color);
        let mut i = 4u;
        let mut red_vec = [];
        let mut green_vec = [];
        let mut blue_vec = [];

        while i < color_vec.len() && color_vec[i] != ',' as u8 {
            red_vec += [color_vec[i]];
            i += 1u;
        }

        i += 1u;

        while i < color_vec.len() && color_vec[i] != ',' as u8 {
            green_vec += [color_vec[i]];
            i += 1u;
        }

        i += 1u;
         
        while i < color_vec.len() && color_vec[i] != ')' as u8 {
            blue_vec += [color_vec[i]];
            i += 1u;
        }

        // TODO, fail by ignoring the rule instead of setting the
        // color to black

        let blue_intense = alt uint::from_str(str::from_bytes(blue_vec)) {
          some(c)  { c }
          none     { 0u }
        };

        let green_intense = alt uint::from_str(str::from_bytes(green_vec)) { 
          some(c)  { c }
          none     { 0u }
        };

        let red_intense = alt uint::from_str(str::from_bytes(red_vec)) { 
          some(c)  { c }
          none     { 0u }
        };


        blue_unit * blue_intense + green_intense * green_unit
            + red_intense * red_unit
    } else {
        alt color {
          "red"   { red_unit * 255u }
          "blue"  { blue_unit * 255u }
          "green" { green_unit * 255u}
          "white" { red_unit * 256u - 1u }
          "black" { 0u }
          // TODO, fail by ignoring the rule instead of setting the
          // color to black
          _       { #debug["Unrecognized color %s", color]; 0u }
        }
    };

    ret result_color;
}

fn parse_rule(reader : token_reader) -> option<~rule> {
    let mut sel_list = [];
    let mut desc_list = [];

    // Collect all the selectors that this rule applies to
    loop {
        let mut cur_sel;

        alt parse_element(reader) {
          some(elmt)  { cur_sel <- elmt; }
          none        { ret none; } // we hit an eof in the middle of a rule
        }

        loop {
            let tok = reader.get();
            alt tok {
              to_descendant {
                alt parse_element(reader) {
                  some(elmt)   { 
                    let built_sel <- cur_sel;
                    let new_sel <- elmt;
                    cur_sel <- ~descendant(built_sel, new_sel)
                  }
                  none         { ret none; }
                }
              }
              to_child {
                alt parse_element(reader) {
                  some(elmt)   { 
                    let built_sel <- cur_sel;
                    let new_sel <- elmt;
                    cur_sel <- ~child(built_sel, new_sel)
                  }
                  none         { ret none; }
                }
              }
              to_sibling {
                alt parse_element(reader) {
                  some(elmt)   { 
                    let built_sel <- cur_sel;
                    let new_sel <- elmt;
                    cur_sel <- ~sibling(built_sel, new_sel)
                  }
                  none         { ret none; }
                }
              }
              to_start_desc {
                let built_sel <- cur_sel; 
                sel_list += [built_sel];
                reader.unget(to_start_desc);
                break;
              }
              to_comma      {
                let built_sel <- cur_sel;
                sel_list += [built_sel];
                reader.unget(to_comma);
                break;
              }
              to_attr(_) | to_end_desc | to_elmt(_) | to_desc(_, _) {
                fail #fmt["Unexpected token %? in elements", tok];
              }
              to_eof        { ret none; }
            }
        }

        // check if we should break out of the nesting loop as well
        let tok = reader.get();
        alt tok {
          to_start_desc { break; }
          to_comma      { }
          _             { reader.unget(tok); }
        }
    }
    
    // Get the description to be applied to the selector
    loop {
        let tok = reader.get();
        alt tok {
          to_end_desc   { break; }
          to_desc(prop, val) {
            alt prop {
              "font-size" {
                // TODO, support more ways to declare a font size than # pt
                assert val.ends_with("pt");
                let num = val.substr(0u, val.len() - 2u);
                
                alt uint::from_str(num) {
                  some(n)    { desc_list += [font_size(n)]; }
                  none       { fail "Nonnumber provided as font size"; }
                }
              }
              "display" {
                alt val {
                  "inline"   { desc_list += [display(di_inline)]; }
                  "block"    { desc_list += [display(di_block)]; }
                  "none"     { desc_list += [display(di_none)]; }
                  _          { #debug["Recieved unknown display value '%s'",
                                      val]; }
                }
              }
              "color" {
                desc_list += [text_color(parse_color(val))];
              }
              "background-color" {
                desc_list += [background_color(parse_color(val))];
              }
              _          { #debug["Recieved unknown style property '%s'",
                                  val]; }
            }
          }
          to_eof        { ret none; }
          to_start_desc | to_descendant | to_child | to_sibling
          | to_comma | to_elmt(_) | to_attr(_)  {
            fail #fmt["Unexpected token %? in description", tok]; 
          }
        }
    }

    ret some(~(sel_list, desc_list));
}

fn build_stylesheet(stream : port<token>) -> [~rule] {
    let mut rule_list = [];
    let reader = {stream : stream, mut lookahead : none};

    loop {
        alt parse_rule(reader) {
          some(rule)   { let r <- rule; rule_list += [r]; }
          none         { break; }
        }
    }

    ret rule_list;
}
