import comm::{port, chan};
import dom::style;
import option::is_none;
import str::from_bytes;
import lexer_util::*;

enum Token {
    StartOpeningTag(str),
    EndOpeningTag,
    EndTag(str),
    SelfCloseTag,
    Text(str),
    Attr(str, str),
    Doctype,
    Eof
}

enum ParseState {
    NormalHtml,
    TagHtml,
}

type HtmlLexer = {
    input_state: InputState,
    mut parser_state: ParseState
};

impl html_methods for HtmlLexer {
    fn parse_html() -> Token {
        let mut ch: u8;
        alt self.input_state.get() {
          CoeChar(c) { ch = c; }
          CoeEof { ret Eof; }
        }
        let token = alt self.parser_state {
          NormalHtml   { self.parse_in_normal_state(ch) }
          TagHtml      { self.parse_in_tag_state(ch) }
        };

        #debug["token=%?", token];
        ret token;
    }

    fn parse_in_normal_state(c: u8) -> Token {
        let mut ch = c;
        if ch == ('<' as u8) {
            alt self.input_state.get() {
              CoeChar(c) { ch = c; }
              CoeEof { self.input_state.parse_err("eof after '<'") }
            }

            if ch == ('!' as u8) {
                self.input_state.eat_whitespace();
                self.input_state.expect_ident("DOCTYPE");
                self.input_state.eat_whitespace();
                self.input_state.expect_ident("html");
                self.input_state.eat_whitespace();
                self.input_state.expect('>' as u8);
                ret Doctype;
            }

            if ch == ('/' as u8) {
                let ident = self.input_state.parse_ident();
                self.input_state.expect('>' as u8);
                ret EndTag(ident);
            }

            self.input_state.unget(ch);

            self.input_state.eat_whitespace();
            let ident = self.input_state.parse_ident();
            self.input_state.eat_whitespace();

            self.parser_state = TagHtml;
            ret StartOpeningTag(ident);
        }
        
        // Make a text node.
        let mut s: [u8] = [ch];
        loop {
            alt self.input_state.get() {
              CoeChar(c) {
                if c == ('<' as u8) {
                    self.input_state.unget(c);
                    ret Text(from_bytes(s));
                }
                s += [c];
              }
              CoeEof { ret Text(from_bytes(s)); }
            }
        }
    }
    
    fn parse_in_tag_state(c: u8) -> Token {
        let mut ch = c;
        
        if ch == ('>' as u8) {
            self.parser_state = NormalHtml;
            ret EndOpeningTag;
        }

        if ch == ('/' as u8) {
            alt self.input_state.get() {
              CoeChar(c) {
                if c == ('>' as u8) {
                    self.parser_state = NormalHtml;
                    ret SelfCloseTag;
                } else {
                    #warn["/ not followed by > in a tag"];
                }
              }
              CoeEof {
                #warn["/ not followed by > at end of file"];
              }
            }
        }

        if !ch.is_alpha() {
            fail #fmt("expected alphabetical in tag but found %c", ch as char);
        }

        // Parse an attribute.
        let mut attribute_name = [ch];
        loop {
            alt self.input_state.get() {
              CoeChar(c) {
                if c == ('=' as u8) { break; }
                attribute_name += [c];
              }
              CoeEof {
                let name = from_bytes(attribute_name);
                ret Attr(copy name, name);
              }
            }
        }

        // Parse the attribute value.
        self.input_state.expect('"' as u8);
        let mut attribute_value = [];
        loop {
            alt self.input_state.get() {
              CoeChar(c) {
                if c == ('"' as u8) { break; }
                attribute_value += [c];
              }
              CoeEof {
                ret Attr(from_bytes(attribute_name), from_bytes(attribute_value));
              }
            }
        }

        // Eat whitespacpe.
        self.input_state.eat_whitespace();

        ret Attr(from_bytes(attribute_name), from_bytes(attribute_value));
    }
}

fn lexer(reader: io::reader, state : ParseState) -> HtmlLexer {
    ret { input_state: {mut lookahead: none, reader: reader}, mut parser_state: state };
}

#[warn(no_non_implicitly_copyable_typarams)]
fn spawn_html_lexer_task(-filename: ~str) -> port<Token> {
    let html_port = port();
    let html_chan = chan(html_port);
    let html_file = copy filename;

    task::spawn(|| {
        let filename = copy html_file;
        assert (copy *filename).ends_with(".html");
        let file_data = io::read_whole_file(*filename).get();
        let reader = io::bytes_reader(file_data);
        
        let lexer = lexer(reader, NormalHtml);

        loop {
            let token = lexer.parse_html();
            let should_break = token == Eof;
            html_chan.send(token);
            if should_break { break; }
        }
    });

    ret html_port;
}
