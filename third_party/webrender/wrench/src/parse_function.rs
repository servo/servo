/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::str::CharIndices;

// support arguments like '4', 'ab', '4.0', '>=10.14', '*123'
fn acceptable_arg_character(c: char) -> bool {
    c.is_alphanumeric() || c == '.' || c == '-' || c == '<' || c == '>' || c == '=' || c == '*'
}

// A crappy parser for parsing strings like "translate(1, 3) blahblah"
// Returns a tuple with three components:
// - First component is the function name (e.g. "translate")
// - Second component is the list of arguments (e.g. vec!["1", "3"])
// - Third component is the rest of the string "blahblah"
pub fn parse_function(s: &str) -> (&str, Vec<&str>, &str) {
    // XXX: This is not particularly easy to read. Sorry.
    struct Parser<'a> {
        itr: CharIndices<'a>,
        start: usize,
        o: Option<(usize, char)>,
    }
    impl<'a> Parser<'a> {
        fn skip_whitespace(&mut self) {
            while let Some(k) = self.o {
                if !k.1.is_whitespace() {
                    break;
                }
                self.start = k.0 + k.1.len_utf8();
                self.o = self.itr.next();
            }
        }
    }
    let mut c = s.char_indices();
    let o = c.next();
    let mut p = Parser {
        itr: c,
        start: 0,
        o: o,
    };

    p.skip_whitespace();

    let mut end = p.start;
    while let Some(k) = p.o {
        if !k.1.is_alphabetic() && k.1 != '_' && k.1 != '-' {
            break;
        }
        end = k.0 + k.1.len_utf8();
        p.o = p.itr.next();
    }

    let name = &s[p.start .. end];
    let mut args = Vec::new();

    p.skip_whitespace();

    if let Some(k) = p.o {
        if k.1 != '(' {
            return (name, args, &s[p.start ..]);
        }
        p.start = k.0 + k.1.len_utf8();
        p.o = p.itr.next();
    }

    loop {
        p.skip_whitespace();

        let mut end = p.start;
        let mut brackets: Vec<char> = Vec::new();
        while let Some(k) = p.o {
            let prev_bracket_count = brackets.len();
            match k.1 {
                '[' | '(' => brackets.push(k.1),
                ']' | ')' => {
                    let open_bracket = match k.1 {
                        ']' => '[',
                        ')' => '(',
                        _ => panic!(),
                    };
                    match brackets.pop() {
                        // Allow final closing ) for command invocation after args
                        None if k.1 == ')' => break,
                        Some(bracket) if bracket == open_bracket => {}
                        _ => panic!("Unexpected closing bracket {}", k.1),
                    }
                }
                _ => {}
            }

            let not_in_bracket = brackets.len() == 0 && prev_bracket_count == 0;
            if !acceptable_arg_character(k.1) && not_in_bracket {
                break;
            }
            end = k.0 + k.1.len_utf8();
            p.o = p.itr.next();
        }

        args.push(&s[p.start .. end]);

        p.skip_whitespace();

        if let Some(k) = p.o {
            p.start = k.0 + k.1.len_utf8();
            p.o = p.itr.next();
            // unless we find a comma we're done
            if k.1 != ',' {
                if k.1 != ')' {
                    panic!("Unexpected closing character: {}", k.1);
                }
                break;
            }
        } else {
            break;
        }
    }
    (name, args, &s[p.start ..])
}

#[test]
fn test() {
    assert_eq!(parse_function("rotate(40)").0, "rotate");
    assert_eq!(parse_function("  rotate(40)").0, "rotate");
    assert_eq!(parse_function("  rotate  (40)").0, "rotate");
    assert_eq!(parse_function("  rotate  (  40 )").1[0], "40");
    assert_eq!(parse_function("rotate(-40.0)").1[0], "-40.0");
    assert_eq!(parse_function("drop-shadow(0, [1, 2, 3, 4], 5)").1[0], "0");
    assert_eq!(parse_function("drop-shadow(0, [1, 2, 3, 4], 5)").1[1], "[1, 2, 3, 4]");
    assert_eq!(parse_function("drop-shadow(0, [1, 2, 3, 4], 5)").1[2], "5");
    assert_eq!(parse_function("drop-shadow(0, [1, 2, [3, 4]], 5)").1[1], "[1, 2, [3, 4]]");
    assert_eq!(parse_function("func(nest([1, 2]), [3, 4])").1[0], "nest([1, 2])");
    assert_eq!(parse_function("func(nest([1, 2]), [nest(3), nest(4)])").1[1], "[nest(3), nest(4)]");
}
