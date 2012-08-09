import comm::{port, chan};
import dom::style;
import option::is_none;
import str::from_bytes;
import vec::push;
import lexer_util::*;
import resource::resource_task;
import resource_task::{ResourceTask, ProgressMsg, Load};
import std::net::url::url;

enum Token {
    StartOpeningTag(~str),
    EndOpeningTag,
    EndTag(~str),
    SelfCloseTag,
    Text(~str),
    Attr(~str, ~str),
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

trait HtmlLexerMethods {
    fn parse_html() -> Token;
    fn parse_in_normal_state(c: u8) -> Token;
    fn parse_in_tag_state(c: u8) -> Token;
}

impl HtmlLexer : HtmlLexerMethods {
    fn parse_html() -> Token {
        let mut ch: u8;
        match self.input_state.get() {
          CoeChar(c) => { ch = c; }
          CoeEof => { return Eof; }
        }
        let token = match self.parser_state {
          NormalHtml => { self.parse_in_normal_state(ch) }
          TagHtml => { self.parse_in_tag_state(ch) }
        };

        #debug["token=%?", token];
        return token;
    }

    fn parse_in_normal_state(c: u8) -> Token {
        let mut ch = c;
        if ch == ('<' as u8) {
            match self.input_state.get() {
              CoeChar(c) => { ch = c; }
              CoeEof => { self.input_state.parse_err(~"eof after '<'") }
            }

            if ch == ('!' as u8) {
                self.input_state.eat_whitespace();
                self.input_state.expect_ident(~"DOCTYPE");
                self.input_state.eat_whitespace();
                self.input_state.expect_ident(~"html");
                self.input_state.eat_whitespace();
                self.input_state.expect('>' as u8);
                return Doctype;
            }

            if ch == ('/' as u8) {
                let ident = self.input_state.parse_ident();
                self.input_state.expect('>' as u8);
                return EndTag(ident);
            }

            self.input_state.unget(ch);

            self.input_state.eat_whitespace();
            let ident = self.input_state.parse_ident();
            self.input_state.eat_whitespace();

            self.parser_state = TagHtml;
            return StartOpeningTag(ident);
        }
        
        // Make a text node.
        let mut s: ~[u8] = ~[ch];
        loop {
            match self.input_state.get() {
              CoeChar(c) => {
                if c == ('<' as u8) {
                    self.input_state.unget(c);
                    return Text(from_bytes(s));
                }
                push(s, c);
              }
              CoeEof => { return Text(from_bytes(s)); }
            }
        }
    }
    
    fn parse_in_tag_state(c: u8) -> Token {
        let mut ch = c;
        
        if ch == ('>' as u8) {
            self.parser_state = NormalHtml;
            return EndOpeningTag;
        }

        if ch == ('/' as u8) {
            match self.input_state.get() {
              CoeChar(c) => {
                if c == ('>' as u8) {
                    self.parser_state = NormalHtml;
                    return SelfCloseTag;
                } else {
                    #warn["/ not followed by > in a tag"];
                }
              }
              CoeEof => {
                #warn["/ not followed by > at end of file"];
              }
            }
        }

        if !ch.is_alpha() {
            fail #fmt("expected alphabetical in tag but found %c", ch as char);
        }

        // Parse an attribute.
        let mut attribute_name = ~[ch];
        loop {
            match self.input_state.get() {
              CoeChar(c) => {
                if c == ('=' as u8) { break; }
                push(attribute_name, c);
              }
              CoeEof => {
                let name = from_bytes(attribute_name);
                return Attr(copy name, name);
              }
            }
        }

        // Parse the attribute value.
        self.input_state.expect('"' as u8);
        let mut attribute_value = ~[];
        loop {
            match self.input_state.get() {
              CoeChar(c) => {
                if c == ('"' as u8) { break; }
                push(attribute_value, c);
              }
              CoeEof => {
                return Attr(from_bytes(attribute_name), from_bytes(attribute_value));
              }
            }
        }

        // Eat whitespacpe.
        self.input_state.eat_whitespace();

        return Attr(from_bytes(attribute_name), from_bytes(attribute_value));
    }
}

fn lexer(+input_port: port<resource_task::ProgressMsg>, state : ParseState) -> HtmlLexer {
    return {
           input_state: {
               mut lookahead: none,
               mut buffer: ~[],
               input_port: input_port,
               mut eof: false
           },
           mut parser_state: state
    };
}

#[allow(non_implicitly_copyable_typarams)]
fn spawn_html_lexer_task(-url: url, resource_task: ResourceTask) -> port<Token> {
    let html_port = port();
    let html_chan = chan(html_port);

    task::spawn(|| {
        let input_port = port();
        // TODO: change copy to move once we can move into closures
        resource_task.send(Load(copy url, input_port.chan()));
        
        let lexer = lexer(input_port, NormalHtml);

        loop {
            let token = lexer.parse_html();
            let should_break = token == Eof;
            html_chan.send(token);
            if should_break { break; }
        }
    });

    return html_port;
}
