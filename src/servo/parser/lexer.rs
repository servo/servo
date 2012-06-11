import comm::{port, chan};
import html::html_methods;
import css::css_methods;
import dom::style;
import option::is_none;

enum parse_state {
    ps_html_normal,
    ps_html_tag,
    ps_css_elmt,
    ps_css_relation,
    ps_css_desc,
    ps_css_attribute
}

type parser = {
    mut lookahead: option<char_or_eof>,
    mut state: parse_state,
    reader: io::reader
};

enum char_or_eof {
    coe_char(u8),
    coe_eof
}

impl u8_methods for u8 {
    fn is_whitespace() -> bool {
        ret self == ' ' as u8 || self == '\n' as u8
            || self == '\t' as u8;
    }

    fn is_alpha() -> bool {
        ret (self >= ('A' as u8) && self <= ('Z' as u8)) ||
            (self >= ('a' as u8) && self <= ('z' as u8));
    }
}

impl u8_vec_methods for [u8] {
    fn to_str() -> str { ret str::from_bytes(self); }
    fn to_html_token() -> html::token { ret html::to_text(self.to_str()); }
    fn to_css_token() -> html::token { ret html::to_text(self.to_str()); }
}

impl util_methods for parser {
    fn get() -> char_or_eof {
        alt copy self.lookahead {
            some(coe) {
                let rv = coe;
                self.lookahead = none;
                ret rv;
            }
            none {
                /* fall through */
            }
        }

        if self.reader.eof() { ret coe_eof; }
        ret coe_char(self.reader.read_byte() as u8);
    }

    fn unget(ch: u8) {
        assert is_none(self.lookahead);
        self.lookahead = some(coe_char(ch));
    }

    fn parse_err(err: str) -> ! {
        fail err
    }

    fn expect(ch: u8) {
        alt self.get() {
            coe_char(c) {
                if c != ch {
                    self.parse_err(#fmt("expected '%c'", ch as char));
                }
            }
            coe_eof {
                self.parse_err(#fmt("expected '%c' at eof", ch as char));
            }
        }
    }

    fn parse_ident() -> str {
        let mut result: [u8] = [];
        loop {
            alt self.get() {
                coe_char(c) {
                    if (c.is_alpha()) {
                        result += [c];
                    } else if result.len() == 0u {
                        self.parse_err("expected ident");
                    } else {
                        self.unget(c);
                        break;
                    }
                }
                coe_eof {
                    self.parse_err("expected ident");
                }
            }
        }
        ret str::from_bytes(result);
    }

    fn expect_ident(expected: str) {
        let actual = self.parse_ident();
        if expected != actual {
            self.parse_err(#fmt("expected '%s' but found '%s'",
                                expected, actual));
        }
    }

    fn eat_whitespace() {
        loop {
            alt self.get() {
                coe_char(c) {
                  if !c.is_whitespace() {
                        self.unget(c);
                        ret;
                    }
                }
                coe_eof {
                    ret;
                }
            }
        }
    }

    fn parse_html() -> html::token {
        let mut ch: u8;
        alt self.get() {
            coe_char(c) { ch = c; }
            coe_eof { ret html::to_eof; }
        }

        let token = alt self.state {
          ps_html_normal   { self.parse_in_normal_state(ch) }
          ps_html_tag      { self.parse_in_tag_state(ch) }
          _                { fail "Parsing in html mode when not in " + 
                                "an html state" }
        };

        #debug["token=%?", token];
        ret token;
    }

    fn parse_css() -> css::token {
        let mut ch: u8;
        alt self.get() {
            coe_char(c) { ch = c; }
            coe_eof { ret css::to_eof; }
        }

        let token = alt self.state {
          ps_css_desc        { self.parse_css_description(ch) }
          ps_css_attribute   { self.parse_css_attribute(ch) }
          ps_css_elmt        { self.parse_css_element(ch) }
          ps_css_relation    { self.parse_css_relation(ch) }
          _                  { fail "Parsing in css mode when not in " + 
                                  "a css state" }
        };

        #debug["token=%?", token];
        ret token;
    }
}

mod html {
    enum token {
        to_start_opening_tag(str),
        to_end_opening_tag,
        to_end_tag(str),
        to_self_close_tag,
        to_text(str),
        to_attr(str, str),
        to_doctype,
        to_eof
    }

    impl html_methods for parser {
        fn parse_in_normal_state(c: u8) -> token {
            let mut ch = c;
            if ch == ('<' as u8) {
                alt self.get() {
                  coe_char(c) { ch = c; }
                  coe_eof { self.parse_err("eof after '<'") }
                }

                if ch == ('!' as u8) {
                    self.eat_whitespace();
                    self.expect_ident("DOCTYPE");
                    self.eat_whitespace();
                    self.expect_ident("html");
                    self.eat_whitespace();
                    self.expect('>' as u8);
                    ret to_doctype;
                }

                if ch == ('/' as u8) {
                    let ident = self.parse_ident();
                    self.expect('>' as u8);
                    ret to_end_tag(ident);
                }

                self.unget(ch);

                self.eat_whitespace();
                let ident = self.parse_ident();
                self.eat_whitespace();

                self.state = ps_html_tag;
                ret to_start_opening_tag(ident);
            }
            
            // Make a text node.
            let mut s: [u8] = [ch];
            loop {
                alt self.get() {
                  coe_char(c) {
                    if c == ('<' as u8) {
                        self.unget(c);
                        ret s.to_html_token();
                    }
                    s += [c];
                  }
                  coe_eof { ret s.to_html_token(); }
                }
            }
        }
        
        fn parse_in_tag_state(c: u8) -> token {
            let mut ch = c;
            
            if ch == ('>' as u8) {
                self.state = ps_html_normal;
                ret to_end_opening_tag;
            }

            if ch == ('/' as u8) {
                self.state = ps_html_normal;
                ret to_self_close_tag;
            }

            if !ch.is_alpha() {
                fail #fmt("expected alphabetical in tag but found %c", 
                          ch as char);
            }

            // Parse an attribute.
            let mut attribute_name = [ch];
            loop {
                alt self.get() {
                  coe_char(c) {
                    if c == ('=' as u8) { break; }
                    attribute_name += [c];
                  }
                  coe_eof {
                    ret to_attr(attribute_name.to_str(),
                                attribute_name.to_str()); }
                }
            }

            // Parse the attribute value.
            self.expect('"' as u8);
            let mut attribute_value = [];
            loop {
                alt self.get() {
                  coe_char(c) {
                    if c == ('"' as u8) { break; }
                    attribute_value += [c];
                  }
                  coe_eof {
                    ret to_attr(attribute_name.to_str(),
                                attribute_value.to_str());
                  }
                }
            }

            // Eat whitespacpe.
            self.eat_whitespace();

            ret to_attr(attribute_name.to_str(), attribute_value.to_str());
        }
    }
}

mod css {
    enum token {
        to_start_desc,
        to_end_desc,
        to_descendant,
        to_child,
        to_sibling,
        to_comma,
        to_elmt(str),
        to_attr(style::attr), 
        to_desc(str, str),
        to_eof
    }

    impl css_methods for parser {
        fn parse_css_relation(c : u8) -> token {
            self.state = ps_css_elmt;

            let token = alt c {
              '{' as u8  { self.state = ps_css_desc; to_start_desc }
              '>' as u8  { to_child }
              '+' as u8  { to_sibling }
              ',' as u8  { to_comma }
              _          { self.unget(c); to_descendant }
            };

            self.eat_whitespace();
            
            ret token;
        }

        fn parse_css_element(c : u8) -> token {
            assert is_none(self.lookahead);

            /* Check for special attributes with an implied element,
            or a wildcard which is not a alphabet character.*/
            if c == '.' as u8 || c == '#' as u8 {
                self.state = ps_css_attribute;
                self.unget(c);
                ret to_elmt("*");
            } else if c == '*' as u8 {
                self.state = ps_css_attribute;
                ret to_elmt("*");
            }

            self.unget(c);
            let element = self.parse_ident();

            self.state = ps_css_attribute;
            
            ret to_elmt(element);
        }

        fn parse_css_attribute(c : u8) -> token {
            let mut ch = c;
            
            /* If we've reached the end of this list of attributes,
            look for the relation to the next element.*/
            if c.is_whitespace() {
                self.state = ps_css_relation;
                self.eat_whitespace();

                alt self.get() {
                  coe_char(c)  { ch = c }
                  coe_eof      { fail "File ended before description " +
                                    "of style" }
                }

                ret self.parse_css_relation(ch);
            }
            
            alt ch {
              '.' as u8 { ret to_attr(
                  style::includes("class", self.parse_ident())); }
              '#' as u8 { ret to_attr(
                  style::includes("id", self.parse_ident())); }
              '[' as u8 {
                let attr_name = self.parse_ident();
                
                alt self.get() {
                  coe_char(c)    { ch = c; }
                  coe_eof        { fail "File ended before " + 
                                      "description finished"; }
                }

                if ch == ']' as u8 {
                    ret to_attr(style::exists(attr_name));
                } else if ch == '=' as u8 {
                    let attr_val = self.parse_ident();
                    self.expect(']' as u8);
                    ret to_attr(style::exact(attr_name, attr_val));
                } else if ch == '~' as u8 {
                    self.expect('=' as u8);
                    let attr_val = self.parse_ident();
                    self.expect(']' as u8);
                    ret to_attr(style::includes(attr_name, attr_val));
                } else if ch == '|' as u8 {
                    self.expect('=' as u8);
                    let attr_val = self.parse_ident();
                    self.expect(']' as u8);
                    ret to_attr(style::starts_with(attr_name, attr_val));
                }
                
                fail #fmt("Unexpected symbol %c in attribute", ch as char);
              }
              _   { fail #fmt("Unexpected symbol %c in attribute", 
                              ch as char); }
            }
        }

        fn parse_css_description(c: u8) -> token {
            let mut ch = c;

            if ch == '}' as u8 {
                self.state = ps_css_elmt;
                self.eat_whitespace();
                ret to_end_desc;
            } else if ch.is_whitespace() {
                self.eat_whitespace();

                alt self.get() {
                  coe_char(c)  { ch = c }
                  coe_eof      { fail "Reached end of file " +  
                                    "in CSS description" }
                }
            }
            
            let mut desc_name = [];
            
            // Get the name of the descriptor
            loop {
                if ch.is_whitespace() {
                    self.eat_whitespace();
                } else if ch == ':' as u8 {
                    if desc_name.len() == 0u {
                        fail "Expected descriptor name";
                    } else {
                        break;
                    }
                } else {
                    desc_name += [ch];
                }

                alt self.get() {
                  coe_char(c)  { ch = c }
                  coe_eof      { fail "Reached end of file " +  
                                    "in CSS description" }
                }
            }

            self.eat_whitespace();
            let mut desc_val = [];

            // Get the value of the descriptor
            loop {
                alt self.get() {
                  coe_char(c)  { ch = c }
                  coe_eof      { fail "Reached end of file " +  
                                    "in CSS description" }
                }

                if ch.is_whitespace() {
                    self.eat_whitespace();
                } else if ch == '}' as u8 {
                    if desc_val.len() == 0u {
                        fail "Expected descriptor value";
                    } else {
                        self.unget('}' as u8);
                        break;
                    }
                } else if ch == ';' as u8 {
                    if desc_val.len() == 0u {
                        fail "Expected descriptor value";
                    } else {
                        break;
                    }
                } else {
                    desc_val += [ch];
                }
            }

            ret to_desc(desc_name.to_str(), desc_val.to_str());
        }
    }
}

fn parser(reader: io::reader, state : parse_state) -> parser {
    ret { mut lookahead: none, mut state: state, reader: reader };
}

fn spawn_html_parser_task(filename: str) -> port<html::token> {
    let result_port = port();
    let result_chan = chan(result_port);
    task::spawn {||
        let file_data = io::read_whole_file(filename).get();
        let reader = io::bytes_reader(file_data);
        
        assert filename.ends_with(".html");
        let parser = parser(reader, ps_html_normal);

        loop {
            let token = parser.parse_html();
            result_chan.send(token);
            if token == html::to_eof { break; }
        }
    };
    ret result_port;
}

fn spawn_css_lexer_task(filename: str) -> port<css::token> {
    let result_port = port();
    let result_chan = chan(result_port);
    task::spawn {||
        assert filename.ends_with(".css");

        let file_try = io::read_whole_file(filename);

        // Check if the given css file existed, if it does, parse it,
        // otherwise just send an eof.  This is a hack to allow
        // guessing that if foo.html exists, foo.css is the
        // corresponding stylesheet.
        if file_try.is_success() {
            #debug["Lexing css sheet %s", filename];
            let file_data = file_try.get();
            let reader = io::bytes_reader(file_data);
        
            let parser : parser = parser(reader, ps_css_elmt);

            loop {
                let token = parser.parse_css();
                result_chan.send(token);
                if token == css::to_eof { break; }
            }
        } else {
            #debug["Failed to open css sheet %s", filename];
            result_chan.send(css::to_eof);
        }
    };
    ret result_port;
}
