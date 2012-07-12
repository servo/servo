import comm::{port, chan};
import dom::style;
import option::is_none;
import str::from_bytes;
import vec::push;

import lexer_util::*;

enum ParserState {
    CssElement,
    CssRelation,
    CssDescription,
    CssAttribute
}

type CssLexer = {
    input_state: InputState,
    mut parser_state: ParserState
};

enum Token {
    StartDescription,
    EndDescription,
    Descendant,
    Child,
    Sibling,
    Comma,
    Element(str),
    Attr(style::Attr), 
    Description(str, str),
    Eof
}

impl css_methods for CssLexer {
    fn parse_css() -> Token {
        let mut ch: u8;
        alt self.input_state.get() {
            CoeChar(c) { ch = c; }
            CoeEof { ret Eof; }
        }

        let token = alt self.parser_state {
          CssDescription        { self.parse_css_description(ch) }
          CssAttribute   { self.parse_css_attribute(ch) }
          CssElement        { self.parse_css_element(ch) }
          CssRelation    { self.parse_css_relation(ch) }
        };

        #debug["token=%?", token];
        ret token;
    }

    fn parse_css_relation(c : u8) -> Token {
        self.parser_state = CssElement;

        let token = alt c {
          '{' as u8  { self.parser_state = CssDescription; StartDescription }
          '>' as u8  { Child }
          '+' as u8  { Sibling }
          ',' as u8  { Comma }
          _          { self.input_state.unget(c); Descendant }
        };

        self.input_state.eat_whitespace();
        
        ret token;
    }

    fn parse_css_element(c : u8) -> Token {
        assert is_none(self.input_state.lookahead);

        /* Check for special attributes with an implied element,
        or a wildcard which is not a alphabet character.*/
        if c == '.' as u8 || c == '#' as u8 {
            self.parser_state = CssAttribute;
            self.input_state.unget(c);
            ret Element("*");
        } else if c == '*' as u8 {
            self.parser_state = CssAttribute;
            ret Element("*");
        }

        self.input_state.unget(c);
        let element = self.input_state.parse_ident();

        self.parser_state = CssAttribute;
        
        ret Element(element);
    }

    fn parse_css_attribute(c : u8) -> Token {
        let mut ch = c;
        
        /* If we've reached the end of this list of attributes,
        look for the relation to the next element.*/
        if c.is_whitespace() {
            self.parser_state = CssRelation;
            self.input_state.eat_whitespace();

            alt self.input_state.get() {
              CoeChar(c)  { ch = c }
              CoeEof      { fail "File ended before description of style" }
            }

            ret self.parse_css_relation(ch);
        }
        
        alt ch {
          '.' as u8 { ret Attr(style::Includes("class", self.input_state.parse_ident())); }
          '#' as u8 { ret Attr(style::Includes("id", self.input_state.parse_ident())); }
          '[' as u8 {
            let attr_name = self.input_state.parse_ident();
            
            alt self.input_state.get() {
              CoeChar(c)    { ch = c; }
              CoeEof        { fail "File ended before description finished"; }
            }

            if ch == ']' as u8 {
                ret Attr(style::Exists(attr_name));
            } else if ch == '=' as u8 {
                let attr_val = self.input_state.parse_ident();
                self.input_state.expect(']' as u8);
                ret Attr(style::Exact(attr_name, attr_val));
            } else if ch == '~' as u8 {
                self.input_state.expect('=' as u8);
                let attr_val = self.input_state.parse_ident();
                self.input_state.expect(']' as u8);
                ret Attr(style::Includes(attr_name, attr_val));
            } else if ch == '|' as u8 {
                self.input_state.expect('=' as u8);
                let attr_val = self.input_state.parse_ident();
                self.input_state.expect(']' as u8);
                ret Attr(style::StartsWith(attr_name, attr_val));
            }
            
            fail #fmt("Unexpected symbol %c in attribute", ch as char);
          }
          _   { fail #fmt("Unexpected symbol %c in attribute", ch as char); }
        }
    }

    fn parse_css_description(c: u8) -> Token {
        let mut ch = c;

        if ch == '}' as u8 {
            self.parser_state = CssElement;
            self.input_state.eat_whitespace();
            ret EndDescription;
        } else if ch.is_whitespace() {
            self.input_state.eat_whitespace();

            alt self.input_state.get() {
              CoeChar(c)  { ch = c }
              CoeEof      { fail "Reached end of file in CSS description" }
            }
        }
        
        let mut desc_name = ~[];
        
        // Get the name of the descriptor
        loop {
            if ch.is_whitespace() {
                self.input_state.eat_whitespace();
            } else if ch == ':' as u8 {
                if desc_name.len() == 0u {
                    fail "Expected descriptor name";
                } else {
                    break;
                }
            } else {
                push(desc_name, ch);
            }

            alt self.input_state.get() {
              CoeChar(c)  { ch = c }
              CoeEof      { fail "Reached end of file in CSS description" }
            }
        }

        self.input_state.eat_whitespace();
        let mut desc_val = ~[];

        // Get the value of the descriptor
        loop {
            alt self.input_state.get() {
              CoeChar(c)  { ch = c }
              CoeEof      { fail "Reached end of file in CSS description" }
            }

            if ch.is_whitespace() {
                self.input_state.eat_whitespace();
            } else if ch == '}' as u8 {
                if desc_val.len() == 0u {
                    fail "Expected descriptor value";
                } else {
                    self.input_state.unget('}' as u8);
                    break;
                }
            } else if ch == ';' as u8 {
                if desc_val.len() == 0u {
                    fail "Expected descriptor value";
                } else {
                    break;
                }
            } else {
                push(desc_val, ch);
            }
        }

        ret Description(from_bytes(desc_name), from_bytes(desc_val));
    }
}

fn parser(reader: io::reader, state : ParserState) -> CssLexer {
    ret { input_state: {mut lookahead: none, reader: reader},
         mut parser_state: state };
}

#[warn(no_non_implicitly_copyable_typarams)]
fn spawn_css_lexer_task(-filename: ~str) -> port<Token> {
    let result_port = port();
    let result_chan = chan(result_port);

    task::spawn(|| {
        assert (*copy filename).ends_with(".css");
        let file_try = io::read_whole_file(*filename);

        // Check if the given css file existed, if it does, parse it,
        // otherwise just send an eof.  This is a hack to allow
        // guessing that if foo.html exists, foo.css is the
        // corresponding stylesheet.
        if file_try.is_ok() {
            #debug["Lexing css sheet %s", *copy filename];
            let file_data = file_try.get();
            let reader = io::bytes_reader(file_data);
        
            let lexer = parser(reader, CssElement);

            loop {
                let token = lexer.parse_css();
                let should_break = token == Eof;
                result_chan.send(token);
                if should_break { break; }
            }
        } else {
            #debug["Failed to open css sheet %s", *copy filename];
            result_chan.send(Eof);
        }
    });

    ret result_port;
}
