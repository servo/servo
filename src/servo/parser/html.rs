import comm::{port, chan};

enum parse_state {
    ps_normal,
    ps_tag
}

type parser = {
    mut lookahead: option<char_or_eof>,
    mut state: parse_state,
    reader: io::reader
};

enum token {
    to_start_opening_tag(str),
    to_end_opening_tag,
    to_end_tag(str),
    to_text(str),
    to_attr(str, str),
    to_doctype,
    to_eof
}

enum char_or_eof {
    coe_char(u8),
    coe_eof
}

impl u8_methods for u8 {
    fn is_alpha() -> bool {
        ret (self >= ('A' as u8) && self <= ('Z' as u8)) ||
            (self >= ('a' as u8) && self <= ('z' as u8));
    }
}

impl u8_vec_methods for [u8] {
    fn to_str() -> str { ret str::from_bytes(self); }
    fn to_str_token() -> token { ret to_text(self.to_str()); }
}

impl methods for parser {
    fn get() -> char_or_eof {
        alt self.lookahead {
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
        assert self.lookahead.is_none();
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
                    if c != (' ' as u8) && c != ('\n' as u8) &&
                           c != ('\t' as u8) {
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

    fn parse() -> token {
        let mut ch: u8;
        alt self.get() {
            coe_char(c) { ch = c; }
            coe_eof { ret to_eof; }
        }

        ret alt self.state {
            ps_normal   { self.parse_in_normal_state(ch) }
            ps_tag      { self.parse_in_tag_state(ch)    }
        }
    }

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

            self.state = ps_tag;
            ret to_start_opening_tag(ident);
        }

        // Make a text node.
        let mut s: [u8] = [ch];
        loop {
            alt self.get() {
                coe_char(c) {
                    if c == ('<' as u8) {
                        self.unget(c);
                        ret s.to_str_token();
                    }
                    s += [c];
                }
                coe_eof { ret s.to_str_token(); }
            }
        }
    }

    fn parse_in_tag_state(c: u8) -> token {
        let mut ch = c;

        if ch == ('>' as u8) {
            self.state = ps_normal;
            ret to_end_opening_tag;
        }

        if !ch.is_alpha() {
            fail "expected alphabetical in tag";
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

        // Eat whitespace.
        self.eat_whitespace();

        ret to_attr(attribute_name.to_str(), attribute_value.to_str());
    }
}

fn parser(reader: io::reader) -> parser {
    ret { mut lookahead: none, mut state: ps_normal, reader: reader };
}

fn spawn_parser_task(filename: str) -> port<token> {
    let result_port = port();
    let result_chan = chan(result_port);
    task::spawn {||
        let file_data = io::read_whole_file(filename).get();
        let reader = io::bytes_reader(file_data);
        let parser = parser(reader);

        loop {
            let token = parser.parse();
            result_chan.send(token);
            if token == to_eof { break; }
        }
    };
    ret result_port;
}

