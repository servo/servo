#[doc = "A collection of functions that are useful for both css and html parsing."]

import option::is_none;
import str::from_bytes;
import vec::push;
import comm::Port;
import resource::resource_task::{ProgressMsg, Payload, Done};

enum CharOrEof {
    CoeChar(u8),
    CoeEof
}

type InputState = {
    mut lookahead: option<CharOrEof>,
    mut buffer: ~[u8],
    input_port: Port<ProgressMsg>,
    mut eof: bool
};

trait U8Methods {
    fn is_whitespace() -> bool;
    fn is_alpha() -> bool;
}

impl u8 : U8Methods {
    fn is_whitespace() -> bool {
        return self == ' ' as u8 || self == '\n' as u8 || self == '\t' as u8;
    }

    fn is_alpha() -> bool {
        return (self >= ('A' as u8) && self <= ('Z' as u8)) ||
               (self >= ('a' as u8) && self <= ('z' as u8));
    }
}

trait InputStateUtil {
    fn get() -> CharOrEof;
    fn unget(ch: u8);
    fn parse_err(err: ~str) -> !;
    fn expect(ch: u8);
    fn parse_ident() -> ~str;
    fn expect_ident(expected: ~str);
    fn eat_whitespace();
}

impl InputState : InputStateUtil {
    fn get() -> CharOrEof {
        match copy self.lookahead {
          some(coe) => {
            let rv = coe;
            self.lookahead = none;
            return rv;
          }
          none => {
            /* fall through */
          }
        }

        // FIXME: Lots of copies here

        if self.buffer.len() > 0 {
            return CoeChar(vec::shift(self.buffer));
        }

        if self.eof {
            return CoeEof;
        }

        match self.input_port.recv() {
          Payload(data) => {
            // TODO: change copy to move once we have match move
            self.buffer = copy data;
            return CoeChar(vec::shift(self.buffer));
          }
          Done(*) => {
            self.eof = true;
            return CoeEof;
          }
        }
    }

    fn unget(ch: u8) {
        assert is_none(self.lookahead);
        self.lookahead = some(CoeChar(ch));
    }

    fn parse_err(err: ~str) -> ! {
        fail err
    }

    fn expect(ch: u8) {
        match self.get() {
          CoeChar(c) => { if c != ch { self.parse_err(#fmt("expected '%c'", ch as char)); } }
          CoeEof => { self.parse_err(#fmt("expected '%c' at eof", ch as char)); }
        }
    }
        
    fn parse_ident() -> ~str {
        let mut result: ~[u8] = ~[];
        loop {
            match self.get() {
              CoeChar(c) => {
                if (c.is_alpha()) { push(result, c); }
                else if result.len() == 0u { self.parse_err(~"expected ident"); }
                else {
                    self.unget(c);
                    break;
                }
              }
              CoeEof => {
                self.parse_err(~"expected ident");
              }
            }
        }
        return str::from_bytes(result);
    }

    fn expect_ident(expected: ~str) {
        let actual = self.parse_ident();
        if expected != actual {
            self.parse_err(#fmt("expected '%s' but found '%s'", expected, actual));
        }
    }

    fn eat_whitespace() {
        loop {
            match self.get() {
              CoeChar(c) => {
                if !c.is_whitespace() {
                    self.unget(c);
                    return;
                }
              }
              CoeEof => {
                return;
              }
            }
        }
    }
}
