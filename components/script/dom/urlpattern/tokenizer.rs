/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::error::{Error, Fallible};

/// <https://urlpattern.spec.whatwg.org/#tokenize>
pub(super) fn tokenize(input: &str, policy: TokenizePolicy) -> Fallible<Vec<Token>> {
    // Step 1. Let tokenizer be a new tokenizer.
    // Step 2. Set tokenizer’s input to input.
    // Step 3. Set tokenizer’s policy to policy.
    let mut tokenizer = Tokenizer {
        input,
        policy,
        index: 0,
        next_index: 0,
        token_list: vec![],
        code_point: char::MIN,
    };

    // Step 4. While tokenizer’s index is less than tokenizer’s input’s code point length:
    while tokenizer.index < tokenizer.input.len() {
        // Step 4.1 Run seek and get the next code point given tokenizer and tokenizer’s index.
        tokenizer.seek_and_get_the_next_code_point(tokenizer.index);

        match tokenizer.code_point {
            // Step 4.2 If tokenizer’s code point is U+002A (*):
            '*' => {
                // Step 4.2.1 Run add a token with default position and length given tokenizer and "asterisk".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Asterisk);

                // Step 4.2.2 Continue.
                continue;
            },
            // Step 4.3 If tokenizer’s code point is U+002B (+) or U+003F (?):
            '+' | '?' => {
                // Step 4.3.1 Run add a token with default position and length given tokenizer and "other-modifier".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::OtherModifier);

                // Step 4.3.2 Continue.
                continue;
            },
            // Step 4.4 If tokenizer’s code point is U+005C (\):
            '\\' => {
                // Step 4.4.1 If tokenizer’s index is equal to tokenizer’s input’s code point length − 1:
                if tokenizer.is_done() {
                    // Step 4.4.1.1 Run process a tokenizing error given tokenizer, tokenizer’s next index,
                    // and tokenizer’s index.
                    tokenizer.process_a_tokenizing_error(tokenizer.next_index, tokenizer.index)?;

                    // Step 4.4.1.2 Continue.
                    continue;
                }

                // Step 4.4.2 Let escaped index be tokenizer’s next index.
                let escaped_index = tokenizer.index;

                // Step 4.4.3 Run get the next code point given tokenizer.
                tokenizer.get_the_next_code_point();

                // Step 4.4.4 Run add a token with default length given tokenizer, "escaped-char",
                // tokenizer’s next index, and escaped index.
                tokenizer.add_a_token_with_default_length(
                    TokenType::EscapedChar,
                    tokenizer.next_index,
                    escaped_index,
                );

                // Step 4.4.5 Continue.
                continue;
            },
            // Step 4.5 If tokenizer’s code point is U+007B ({):
            '{' => {
                // Step 4.5.1 Run add a token with default position and length given tokenizer and "open".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Open);

                // Step 4.5.2 Continue.
                continue;
            },
            // Step 4.6 If tokenizer’s code point is U+007D (}):
            '}' => {
                // Step 4.6.1 Run add a token with default position and length given tokenizer and "close".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Close);

                // Step 4.6.2 Continue.
                continue;
            },
            // Step 4.7 If tokenizer’s code point is U+003A (:):
            ':' => {
                // Step 4.7.1 Let name position be tokenizer’s next index.
                let mut name_position = tokenizer.next_index;

                // Step 4.7.2 Let name start be name position.
                let name_start = name_position;

                // Step 4.7.3 While name position is less than tokenizer’s input’s code point length:
                while name_position < tokenizer.input.len() {
                    // Step 4.7.3.1 Run seek and get the next code point given tokenizer and name position.
                    tokenizer.seek_and_get_the_next_code_point(name_position);

                    // Step 4.7.3.2 Let first code point be true if name position equals name start
                    // and false otherwise.
                    let first_code_point = name_position == name_start;

                    // Step 4.7.3.3 Let valid code point be the result of running is a valid name
                    // code point given tokenizer’s code point and first code point.
                    let valid_code_point =
                        is_a_valid_name_code_point(tokenizer.code_point, first_code_point);

                    // Step 4.7.3.4 If valid code point is false break.
                    if !valid_code_point {
                        break;
                    }

                    // Step 4.6.3.5 Set name position to tokenizer’s next index.
                    name_position = tokenizer.next_index;
                }

                // Step 4.7.4 If name position is less than or equal to name start:
                if name_position <= name_start {
                    // Step 4.7.4.1 Run process a tokenizing error given tokenizer, name start, and tokenizer’s index.
                    tokenizer.process_a_tokenizing_error(name_start, tokenizer.index)?;

                    // Step 4.7.4.2 Continue.
                    continue;
                }

                // Step 4.7.5 Run add a token with default length given tokenizer, "name", name position,
                // and name start.
                tokenizer.add_a_token_with_default_length(
                    TokenType::Name,
                    name_position,
                    name_start,
                );

                // Step 4.7.6 Continue.
                continue;
            },
            // Step 4.8 If tokenizer’s code point is U+0028 (():
            '(' => {
                // Step 4.8.1 Let depth be 1.
                let mut depth = 1;

                // Step 4.8.2 Let regexp position be tokenizer’s next index.
                let mut regexp_position = tokenizer.next_index;

                // Step 4.8.3 Let regexp start be regexp position.
                let regexp_start = regexp_position;

                // Step 4.8.4 Let error be false.
                let mut error = false;

                // Step 4.8.5 While regexp position is less than tokenizer’s input’s code point length:
                while regexp_position < tokenizer.input.len() {
                    // Step 4.8.5.1 Run seek and get the next code point given tokenizer and regexp position.
                    tokenizer.seek_and_get_the_next_code_point(regexp_position);

                    // Step 4.8.5.2 If tokenizer’s code point is not an ASCII code point:
                    if !tokenizer.code_point.is_ascii() {
                        // Step 4.8.5.1.1 Run process a tokenizing error given tokenizer, regexp start,
                        // and tokenizer’s index.
                        tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                        // Step 4.8.5.1.2 Set error to true.
                        error = true;

                        // Step 4.8.5.1.2 Break.
                        break;
                    }

                    // Step 4.8.5.3 If regexp position equals regexp start and tokenizer’s code point is U+003F (?):
                    if regexp_position == regexp_start && tokenizer.code_point == '?' {
                        // Step 4.8.5.3.1 Run process a tokenizing error given tokenizer, regexp start,
                        // and tokenizer’s index.
                        tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                        // Step 4.8.5.3.2 Set error to true.
                        error = true;

                        // Step 4.8.5.3.3 Break.
                        break;
                    }

                    // Step 4.8.5.4 If tokenizer’s code point is U+005C (\):
                    if tokenizer.code_point == '\\' {
                        // Step 4.8.5.4.1 If regexp position equals tokenizer’s input’s code point length − 1:
                        if tokenizer.is_last_character(regexp_position) {
                            // Step 4.8.5.4.1.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.4.1.2 Set error to true.
                            error = true;

                            // Step 4.8.5.4.1.3 Break
                            break;
                        }

                        // Step 4.8.5.4.2 Run get the next code point given tokenizer.
                        tokenizer.get_the_next_code_point();

                        // Step 4.8.5.4.3 If tokenizer’s code point is not an ASCII code point:
                        if !tokenizer.code_point.is_ascii() {
                            // Step 4.8.5.4.3.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.4.3.2 Set error to true.
                            error = true;

                            // Step 4.8.5.4.3.3 Break
                            break;
                        }

                        // Step 4.8.5.4.4 Set regexp position to tokenizer’s next index.
                        regexp_position = tokenizer.next_index;

                        // Step 4.8.5.4.5 Continue.
                        continue;
                    }

                    // Step 4.8.5.5 If tokenizer’s code point is U+0029 ()):
                    if tokenizer.code_point == ')' {
                        // Step 4.8.5.5.1 Decrement depth by 1.
                        depth -= 1;

                        // Step 4.8.5.5.2 If depth is 0:
                        if depth == 0 {
                            // Step 4.8.5.5.2.1 Set regexp position to tokenizer’s next index.
                            regexp_position = tokenizer.next_index;

                            // Step 4.8.5.5.2.2 Break.
                            break;
                        }
                    }
                    // Step 4.8.5.6 Otherwise if tokenizer’s code point is U+0028 (():
                    else if tokenizer.code_point == '(' {
                        // Step 4.8.5.6.1 Increment depth by 1.
                        depth += 1;

                        // Step 4.8.5.6.2 If regexp position equals tokenizer’s input’s code point length − 1:
                        if tokenizer.is_last_character(regexp_position) {
                            // Step 4.8.5.6.2.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.6.2.2 Set error to true.
                            error = true;

                            // Step 4.8.5.6.2.3 Break
                            break;
                        }

                        // Step 4.8.5.6.3 Let temporary position be tokenizer’s next index.
                        let temporary_position = tokenizer.next_index;

                        // Step 4.8.5.6.4 Run get the next code point given tokenizer.
                        tokenizer.get_the_next_code_point();

                        // Step 4.8.5.6.5 If tokenizer’s code point is not U+003F (?):
                        if tokenizer.code_point != '?' {
                            // Step 4.8.5.6.5.1 Run process a tokenizing error given tokenizer, regexp start,
                            // and tokenizer’s index.
                            tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                            // Step 4.8.5.6.5.2 Set error to true.
                            error = true;

                            // Step 4.8.5.6.5.3 Break.
                            break;
                        }

                        // Step 4.8.5.6.6 Set tokenizer’s next index to temporary position.
                        tokenizer.next_index = temporary_position;
                    }

                    // Step 4.8.5.7 Set regexp position to tokenizer’s next index.
                    regexp_position = tokenizer.next_index;
                }

                // Step 4.8.6 If error is true continue.
                if error {
                    continue;
                }

                // Step 4.8.7 If depth is not zero:
                if depth != 0 {
                    // Step 4.8.7.1 Run process a tokenizing error given tokenizer, regexp start,
                    // and tokenizer’s index
                    tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                    // Step 4.8.7.2 Continue.
                    continue;
                }

                // Step 4.8.8 Let regexp length be regexp position − regexp start − 1.
                let regexp_length = regexp_position - regexp_start - 1;

                // Step 4.8.9 If regexp length is zero:
                if regexp_length == 0 {
                    // Step 4.8.9.1 Run process a tokenizing error given tokenizer, regexp start,
                    // and tokenizer’s index.
                    tokenizer.process_a_tokenizing_error(regexp_start, tokenizer.index)?;

                    // Step 4.8.9.2 Continue.
                    continue;
                }

                // Step 4.8.10 Run add a token given tokenizer, "regexp", regexp position,
                // regexp start, and regexp length.
                tokenizer.add_a_token(
                    TokenType::Regexp,
                    regexp_position,
                    regexp_start,
                    regexp_length,
                );

                // Step 4.8.11 Continue.
                continue;
            },
            _ => {
                // Step 4.9 Run add a token with default position and length given tokenizer and "char".
                tokenizer.add_a_token_with_default_position_and_length(TokenType::Char);
            },
        }
    }

    // Step 5. Run add a token with default length given tokenizer, "end", tokenizer’s index, and tokenizer’s index.
    tokenizer.add_a_token_with_default_length(TokenType::End, tokenizer.index, tokenizer.index);

    // Step 6.Return tokenizer’s token list.
    Ok(tokenizer.token_list)
}

/// <https://urlpattern.spec.whatwg.org/#tokenizer>
struct Tokenizer<'a> {
    /// <https://urlpattern.spec.whatwg.org/#tokenizer-input>
    input: &'a str,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-policy>
    policy: TokenizePolicy,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-index>
    ///
    /// Note that we deviate the from the spec and index bytes, not code points.
    index: usize,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-next-index>
    ///
    /// Note that we deviate the from the spec and index bytes, not code points.
    next_index: usize,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-token-list>
    token_list: Vec<Token<'a>>,

    /// <https://urlpattern.spec.whatwg.org/#tokenizer-code-point>
    code_point: char,
}

/// <https://urlpattern.spec.whatwg.org/#token>
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // index isn't used yet, because constructor strings aren't parsed
pub(super) struct Token<'a> {
    /// <https://urlpattern.spec.whatwg.org/#token-index>
    pub(super) index: usize,

    /// <https://urlpattern.spec.whatwg.org/#token-value>
    pub(super) value: &'a str,

    /// <https://urlpattern.spec.whatwg.org/#token-type>
    pub(super) token_type: TokenType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TokenType {
    /// <https://urlpattern.spec.whatwg.org/#token-type-open>
    Open,

    /// <https://urlpattern.spec.whatwg.org/#token-type-close>
    Close,

    /// <https://urlpattern.spec.whatwg.org/#token-type-regexp>
    Regexp,

    /// <https://urlpattern.spec.whatwg.org/#token-type-name>
    Name,

    /// <https://urlpattern.spec.whatwg.org/#token-type-char>
    Char,

    /// <https://urlpattern.spec.whatwg.org/#token-type-escaped-char>
    EscapedChar,

    /// <https://urlpattern.spec.whatwg.org/#token-type-other-modifier>
    OtherModifier,

    /// <https://urlpattern.spec.whatwg.org/#token-type-asterisk>
    Asterisk,

    /// <https://urlpattern.spec.whatwg.org/#token-type-end>
    End,

    /// <https://urlpattern.spec.whatwg.org/#token-type-invalid-char>
    InvalidChar,
}

/// <https://urlpattern.spec.whatwg.org/#tokenize-policy>
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TokenizePolicy {
    /// <https://urlpattern.spec.whatwg.org/#tokenize-policy-strict>
    Strict,

    /// <https://urlpattern.spec.whatwg.org/#tokenize-policy-lenient>
    Lenient,
}

impl Tokenizer<'_> {
    fn is_last_character(&self, position: usize) -> bool {
        self.input[position..].chars().count() == 1
    }

    fn is_done(&self) -> bool {
        self.input[self.next_index..].is_empty()
    }

    /// <https://urlpattern.spec.whatwg.org/#get-the-next-code-point>
    fn get_the_next_code_point(&mut self) {
        // Step 1. Set tokenizer’s code point to the Unicode code point in tokenizer’s
        // input at the position indicated by tokenizer’s next index.
        self.code_point = self.input[self.next_index..]
            .chars()
            .next()
            .expect("URLPattern tokenizer is trying to read out of bounds");

        // Step 2. Increment tokenizer’s next index by 1.
        // NOTE: Because our next_index is indexing bytes (not code points) we use
        // the utf8 length of the code point instead.
        self.next_index = self.next_index.wrapping_add(self.code_point.len_utf8());
    }

    /// <https://urlpattern.spec.whatwg.org/#seek-and-get-the-next-code-point>
    fn seek_and_get_the_next_code_point(&mut self, index: usize) {
        // Step 1. Set tokenizer’s next index to index.
        self.next_index = index;

        // Step 2. Run get the next code point given tokenizer.
        self.get_the_next_code_point();
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-token>
    fn add_a_token(
        &mut self,
        token_type: TokenType,
        next_position: usize,
        value_position: usize,
        value_length: usize,
    ) {
        // Step 1. Let token be a new token.
        // Step 2. Set token’s type to type.
        // Step 3. Set token’s index to tokenizer’s index.
        // Step 4. Set token’s value to the code point substring from value position
        // with length value length within tokenizer’s input.
        let token = Token {
            token_type,
            index: self.index,
            value: &self.input[value_position..][..value_length],
        };

        // Step 5. Append token to the back of tokenizer’s token list.
        self.token_list.push(token);

        // Step 6. Set tokenizer’s index to next position.
        self.index = next_position;
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-token-with-default-position-and-length>
    fn add_a_token_with_default_position_and_length(&mut self, token_type: TokenType) {
        // Step 1. Run add a token with default length given tokenizer, type,
        // tokenizer’s next index, and tokenizer’s index.
        self.add_a_token_with_default_length(token_type, self.next_index, self.index);
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-token-with-default-length>
    fn add_a_token_with_default_length(
        &mut self,
        token_type: TokenType,
        next_position: usize,
        value_position: usize,
    ) {
        // Step 1. Let computed length be next position − value position.
        let computed_length = next_position - value_position;

        // Step 2. Run add a token given tokenizer, type, next position, value position, and computed length.
        self.add_a_token(token_type, next_position, value_position, computed_length);
    }

    /// <https://urlpattern.spec.whatwg.org/#process-a-tokenizing-error>
    fn process_a_tokenizing_error(
        &mut self,
        next_position: usize,
        value_position: usize,
    ) -> Fallible<()> {
        // Step 1. If tokenizer’s policy is "strict", then throw a TypeError.
        if self.policy == TokenizePolicy::Strict {
            return Err(Error::Type("Failed to tokenize URL pattern".into()));
        }

        // Step 2. Assert: tokenizer’s policy is "lenient".
        debug_assert_eq!(self.policy, TokenizePolicy::Lenient);

        // Step 3. Run add a token with default length given tokenizer, "invalid-char",
        // next position, and value position.
        self.add_a_token_with_default_length(TokenType::InvalidChar, next_position, value_position);

        Ok(())
    }
}

/// <https://urlpattern.spec.whatwg.org/#is-a-valid-name-code-point>
pub(super) fn is_a_valid_name_code_point(code_point: char, first: bool) -> bool {
    // FIXME: implement this check
    _ = first;
    code_point.is_alphabetic()
}
