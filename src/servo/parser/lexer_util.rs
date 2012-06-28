#[doc = "A collection of functions that are useful for both css and html parsing."]

import option::is_none;
import str::from_bytes;
import vec::push;

enum CharOrEof {
    CoeChar(u8),
    CoeEof
}

type InputState = {
    mut lookahead: option<CharOrEof>,
    reader: io::reader
};

trait u8_methods {
    fn is_whitespace() -> bool;
    fn is_alpha() -> bool;
}

impl u8_methods of u8_methods for u8 {
    fn is_whitespace() -> bool {
        ret self == ' ' as u8 || self == '\n' as u8 || self == '\t' as u8;
    }

    fn is_alpha() -> bool {
        ret (self >= ('A' as u8) && self <= ('Z' as u8)) ||
            (self >= ('a' as u8) && self <= ('z' as u8));
    }
}

trait util_methods {
    fn get() -> CharOrEof;
    fn unget(ch: u8);
    fn parse_err(err: ~str) -> !;
    fn expect(ch: u8);
    fn parse_ident() -> ~str;
    fn expect_ident(expected: ~str);
    fn eat_whitespace();
}

impl util_methods of util_methods for InputState {
    fn get() -> CharOrEof {
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

        if self.reader.eof() { ret CoeEof; }
        ret CoeChar(self.reader.read_byte() as u8);
    }

    fn unget(ch: u8) {
        assert is_none(self.lookahead);
        self.lookahead = some(CoeChar(ch));
    }

    fn parse_err(err: ~str) -> ! {
        fail err
    }

    fn expect(ch: u8) {
        alt self.get() {
          CoeChar(c) { if c != ch { self.parse_err(#fmt("expected '%c'", ch as char)); } }
          CoeEof { self.parse_err(#fmt("expected '%c' at eof", ch as char)); }
        }
    }
        
    fn parse_ident() -> ~str {
        let mut result: ~[u8] = ~[];
        loop {
            alt self.get() {
              CoeChar(c) {
                if (c.is_alpha()) { push(result, c); }
                else if result.len() == 0u { self.parse_err(~"expected ident"); }
                else {
                    self.unget(c);
                    break;
                }
              }
              CoeEof {
                self.parse_err(~"expected ident");
              }
            }
        }
        ret str::from_bytes(result);
    }

    fn expect_ident(expected: ~str) {
        let actual = self.parse_ident();
        if expected != actual {
            self.parse_err(#fmt("expected '%s' but found '%s'", expected, actual));
        }
    }

    fn eat_whitespace() {
        loop {
            alt self.get() {
              CoeChar(c) {
                if !c.is_whitespace() {
                    self.unget(c);
                    ret;
                }
              }
              CoeEof {
                ret;  
              }
            }
        }
    }
}
