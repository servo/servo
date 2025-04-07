/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::error::{Error, Fallible};

use crate::dom::urlpattern::tokenizer::{Token, TokenType, TokenizePolicy, tokenize};
use crate::dom::urlpattern::{
    EncodingCallback, FULL_WILDCARD_REGEXP_VALUE, Options, Part, PartModifier, PartType,
    generate_a_segment_wildcard_regexp,
};

/// <https://urlpattern.spec.whatwg.org/#parse-a-pattern-string>
pub(super) fn parse_a_pattern_string(
    input: &str,
    options: Options,
    encoding_callback: EncodingCallback,
) -> Fallible<Vec<Part>> {
    // Step 1. Let parser be a new pattern parser whose encoding callback is encoding callback and
    // segment wildcard regexp is the result of running generate a segment wildcard regexp given options.
    let mut parser = PatternParser::new(
        generate_a_segment_wildcard_regexp(options),
        encoding_callback,
    );

    // Step 2. Set parser’s token list to the result of running tokenize given input and "strict".
    parser.token_list = tokenize(input, TokenizePolicy::Strict)?;

    // Step 3. While parser’s index is less than parser’s token list’s size:
    while parser.index < parser.token_list.len() {
        // Step 3.1 Let char token be the result of running try to consume a token given parser and "char".
        let char_token = parser.try_to_consume_a_token(TokenType::Char);

        // Step 3.2 Let name token be the result of running try to consume a token given parser and "name".
        let mut name_token = parser.try_to_consume_a_token(TokenType::Name);

        // Step 3.3 Let regexp or wildcard token be the result of running try to consume a
        // regexp or wildcard token given parser and name token.
        let mut regexp_or_wildcard_token =
            parser.try_to_consume_a_regexp_or_wildcard_token(name_token);

        // Step 3.4 If name token is not null or regexp or wildcard token is not null:
        if name_token.is_some() || regexp_or_wildcard_token.is_some() {
            // Step 3.4.1 Let prefix be the empty string.
            let mut prefix = "";

            // Step 3.4.2 If char token is not null then set prefix to char token’s value.
            if let Some(char_token) = char_token {
                prefix = char_token.value;
            }

            // Step 3.4.3 If prefix is not the empty string and not options’s prefix code point:
            let prefix_is_prefix_code_point = options.prefix_code_point.is_some_and(|c| {
                let mut buffer = [0; 4];
                prefix == c.encode_utf8(&mut buffer)
            });
            if !prefix.is_empty() && !prefix_is_prefix_code_point {
                // Step 3.4.3.1 Append prefix to the end of parser’s pending fixed value.
                parser.pending_fixed_value.push_str(prefix);

                // Step 3.4.3.2 Set prefix to the empty string.
                prefix = "";
            }

            // Step 3.4.4 Run maybe add a part from the pending fixed value given parser.
            parser.maybe_add_a_part_from_the_pending_fixed_value()?;

            // Step 3.4.5 Let modifier token be the result of running try to consume a modifier token given parser.
            let modifier_token = parser.try_to_consume_a_modifier_token();

            // Step 3.4.6 Run add a part given parser, prefix, name token, regexp or wildcard token,
            // the empty string, and modifier token.
            parser.add_a_part(
                prefix,
                name_token,
                regexp_or_wildcard_token,
                "",
                modifier_token,
            )?;

            // Step 3.4.7 Continue.
            continue;
        }

        // Step 3.5 Let fixed token be char token.
        let mut fixed_token = char_token;

        // Step 3.6 If fixed token is null, then set fixed token to the result of running
        // try to consume a token given parser and "escaped-char".
        if fixed_token.is_none() {
            fixed_token = parser.try_to_consume_a_token(TokenType::EscapedChar);
        }

        // Step 3.7 If fixed token is not null:
        if let Some(fixed_token) = fixed_token {
            // Step 3.7.1 Append fixed token’s value to parser’s pending fixed value.
            parser.pending_fixed_value.push_str(fixed_token.value);

            // Step 3.7.2 Continue.
            continue;
        }

        // Step 3.8 Let open token be the result of running try to consume a token given parser and "open".
        let open_token = parser.try_to_consume_a_token(TokenType::Open);

        // Step 3.9 If open token is not null:
        if open_token.is_some() {
            // Step 3.9.1 Let prefix be the result of running consume text given parser.
            let prefix = parser.consume_text();

            // Step 3.9.2 Set name token to the result of running try to consume a token given parser and "name".
            name_token = parser.try_to_consume_a_token(TokenType::Name);

            // Step 3.9.3 Set regexp or wildcard token to the result of running try to consume a regexp or wildcard
            // token given parser and name token.
            regexp_or_wildcard_token = parser.try_to_consume_a_regexp_or_wildcard_token(name_token);

            // Step 3.9.4 Let suffix be the result of running consume text given parser.
            let suffix = parser.consume_text();

            // Step 3.9.5 Run consume a required token given parser and "close".
            parser.consume_a_required_token(TokenType::Close)?;

            // Step 3.9.6 Let modifier token be the result of running try to consume a modifier token given parser.
            let modifier_token = parser.try_to_consume_a_modifier_token();

            // Step 3.9.7 Run add a part given parser, prefix, name token, regexp or wildcard token,
            // suffix, and modifier token.
            parser.add_a_part(
                &prefix,
                name_token,
                regexp_or_wildcard_token,
                &suffix,
                modifier_token,
            )?;

            // Step 3.9.8 Continue.
            continue;
        }

        // Step 3.10 Run maybe add a part from the pending fixed value given parser.
        parser.maybe_add_a_part_from_the_pending_fixed_value()?;

        // Step 3.11 Run consume a required token given parser and "end".
        parser.consume_a_required_token(TokenType::End)?;
    }

    Ok(parser.part_list)
}

/// <https://urlpattern.spec.whatwg.org/#pattern-parser>
struct PatternParser<'a> {
    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-token-list>
    token_list: Vec<Token<'a>>,

    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-encoding-callback>
    encoding_callback: EncodingCallback,

    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-segment-wildcard-regexp>
    segment_wildcard_regexp: String,

    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-part-list>
    part_list: Vec<Part>,

    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-pending-fixed-value>
    pending_fixed_value: String,

    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-index>
    index: usize,

    /// <https://urlpattern.spec.whatwg.org/#pattern-parser-next-numeric-name>
    next_numeric_name: usize,
}

impl<'a> PatternParser<'a> {
    fn new(segment_wildcard_regexp: String, encoding_callback: EncodingCallback) -> Self {
        Self {
            token_list: vec![],
            segment_wildcard_regexp,
            part_list: vec![],
            pending_fixed_value: String::new(),
            index: 0,
            next_numeric_name: 0,
            encoding_callback,
        }
    }

    /// <https://urlpattern.spec.whatwg.org/#try-to-consume-a-token>
    fn try_to_consume_a_token(&mut self, token_type: TokenType) -> Option<Token<'a>> {
        // Step 1. Assert: parser’s index is less than parser’s token list size.
        debug_assert!(self.index < self.token_list.len());

        // Step 2. Let next token be parser’s token list[parser’s index].
        let next_token = self.token_list[self.index];

        // Step 3. If next token’s type is not type return null.
        if next_token.token_type != token_type {
            return None;
        }

        // Step 4. Increment parser’s index by 1.
        self.index += 1;

        // Step 5. Return next token.
        Some(next_token)
    }

    /// <https://urlpattern.spec.whatwg.org/#try-to-consume-a-modifier-token>
    fn try_to_consume_a_modifier_token(&mut self) -> Option<Token<'a>> {
        // Step 1. Let token be the result of running try to consume a token given parser and "other-modifier".
        let token = self.try_to_consume_a_token(TokenType::OtherModifier);

        // Step 2. If token is not null, then return token.
        if token.is_some() {
            return token;
        }

        // Step 3. Set token to the result of running try to consume a token given parser and "asterisk".
        let token = self.try_to_consume_a_token(TokenType::Asterisk);

        // Step 4. Return token.
        token
    }

    /// <https://urlpattern.spec.whatwg.org/#consume-a-required-token>
    fn consume_a_required_token(&mut self, token_type: TokenType) -> Fallible<Token<'a>> {
        // Step 1. Let result be the result of running try to consume a token given parser and type.
        let result = self.try_to_consume_a_token(token_type);

        // Step 2. If result is null, then throw a TypeError.
        let Some(result) = result else {
            return Err(Error::Type(format!(
                "Missing required token {token_type:?}"
            )));
        };

        // Step 3. Return result.
        Ok(result)
    }

    /// <https://urlpattern.spec.whatwg.org/#try-to-consume-a-regexp-or-wildcard-token>
    fn try_to_consume_a_regexp_or_wildcard_token(
        &mut self,
        name_token: Option<Token<'a>>,
    ) -> Option<Token<'a>> {
        // Step 1. Let token be the result of running try to consume a token given parser and "regexp".
        let mut token = self.try_to_consume_a_token(TokenType::Regexp);

        // Step 2. If name token is null and token is null, then set token to the result of running
        // try to consume a token given parser and "asterisk".
        if name_token.is_none() && token.is_none() {
            token = self.try_to_consume_a_token(TokenType::Asterisk);
        }

        // Step 3. Return token.
        token
    }

    /// <https://urlpattern.spec.whatwg.org/#maybe-add-a-part-from-the-pending-fixed-value>
    fn maybe_add_a_part_from_the_pending_fixed_value(&mut self) -> Fallible<()> {
        // Step 1. If parser’s pending fixed value is the empty string, then return.
        if self.pending_fixed_value.is_empty() {
            return Ok(());
        }

        // Step 2. Let encoded value be the result of running parser’s encoding callback
        // given parser’s pending fixed value.
        let encoded_value = (self.encoding_callback)(&self.pending_fixed_value)?;

        // Step 3. Set parser’s pending fixed value to the empty string.
        self.pending_fixed_value.clear();

        // Step 4. Let part be a new part whose type is "fixed-text", value is encoded value, and modifier is "none".
        let part = Part::new(PartType::FixedText, encoded_value, PartModifier::None);

        // Step 5. Append part to parser’s part list.
        self.part_list.push(part);

        Ok(())
    }

    /// <https://urlpattern.spec.whatwg.org/#add-a-part>
    fn add_a_part(
        &mut self,
        prefix: &str,
        name_token: Option<Token<'a>>,
        regexp_or_wildcard_token: Option<Token<'a>>,
        suffix: &str,
        modifier_token: Option<Token<'a>>,
    ) -> Fallible<()> {
        // Step 1. Let modifier be "none".
        let mut modifier = PartModifier::None;

        // Step 2. If modifier token is not null:
        if let Some(modifier_token) = modifier_token {
            // Step 2.1 If modifier token’s value is "?" then set modifier to "optional".
            if modifier_token.value == "?" {
                modifier = PartModifier::Optional;
            }
            // Step 2.2 Otherwise if modifier token’s value is "*" then set modifier to "zero-or-more".
            else if modifier_token.value == "*" {
                modifier = PartModifier::ZeroOrMore;
            }
            // Step 2.3 Otherwise if modifier token’s value is "+" then set modifier to "one-or-more".
            else if modifier_token.value == "+" {
                modifier = PartModifier::OneOrMore;
            }
        }

        // Step 3. If name token is null and regexp or wildcard token is null and modifier is "none":
        if name_token.is_none() &&
            regexp_or_wildcard_token.is_none() &&
            modifier == PartModifier::None
        {
            // Step 3.1 Append prefix to the end of parser’s pending fixed value.
            self.pending_fixed_value.push_str(prefix);

            // Step 3.2 Return
            return Ok(());
        }

        // Step 4. Run maybe add a part from the pending fixed value given parser.
        self.maybe_add_a_part_from_the_pending_fixed_value()?;

        // Step 5. If name token is null and regexp or wildcard token is null:
        if name_token.is_none() && regexp_or_wildcard_token.is_none() {
            // Step 5.1 Assert: suffix is the empty string.
            debug_assert!(suffix.is_empty());

            // Step 5.2 If prefix is the empty string, then return.
            if prefix.is_empty() {
                return Ok(());
            }

            // Step 5.3 Let encoded value be the result of running parser’s encoding callback given prefix.
            let encoded_value = (self.encoding_callback)(prefix)?;

            // Step 5.4 Let part be a new part whose type is "fixed-text",
            // value is encoded value, and modifier is modifier.
            let part = Part::new(PartType::FixedText, encoded_value, modifier);

            // Step 5.5 Append part to parser’s part list.
            self.part_list.push(part);

            // Step 6. Return.
            return Ok(());
        }

        // Step 6. Let regexp value be the empty string.
        let mut regexp_value = {
            // Step 7. If regexp or wildcard token is null, then set regexp value to parser’s segment wildcard regexp.
            match regexp_or_wildcard_token {
                None => self.segment_wildcard_regexp.clone(),
                Some(token) => {
                    // Step 8. Otherwise if regexp or wildcard token’s type is "asterisk",
                    // then set regexp value to the full wildcard regexp value.
                    if token.token_type == TokenType::Asterisk {
                        FULL_WILDCARD_REGEXP_VALUE.into()
                    }
                    // Step 9. Otherwise set regexp value to regexp or wildcard token’s value.
                    else {
                        token.value.to_owned()
                    }
                },
            }
        };

        // Step 10. Let type be "regexp".
        let mut part_type = PartType::Regexp;

        // Step 11. If regexp value is parser’s segment wildcard regexp:
        if regexp_value == self.segment_wildcard_regexp {
            // Step 11.1 Set type to "segment-wildcard".
            part_type = PartType::SegmentWildcard;

            // Step 11.2 Set regexp value to the empty string.
            regexp_value.clear();
        }
        // Step 12. Otherwise if regexp value is the full wildcard regexp value:
        else if regexp_value == FULL_WILDCARD_REGEXP_VALUE {
            // Step 12.1 Set type to "full-wildcard".
            part_type = PartType::FullWildcard;

            // Step 12.2 Set regexp value to the empty string.
            regexp_value.clear();
        }

        // Step 13. Let name be the empty string.
        let mut name = String::new();

        // Step 14. If name token is not null, then set name to name token’s value.
        if let Some(name_token) = name_token {
            name = name_token.value.to_owned();
        }
        // Step 15. Otherwise if regexp or wildcard token is not null:
        else if regexp_or_wildcard_token.is_some() {
            // Step 15.1 Set name to parser’s next numeric name, serialized.
            name = self.next_numeric_name.to_string();

            // Step 15.2 Increment parser’s next numeric name by 1.
            self.next_numeric_name = self.next_numeric_name.wrapping_add(1);
        }

        // Step 16. If the result of running is a duplicate name given parser and name is true, then throw a TypeError.
        if self.is_a_duplicate_name(&name) {
            return Err(Error::Type(format!("Duplicate part name: {name:?}")));
        }

        // Step 17. Let encoded prefix be the result of running parser’s encoding callback given prefix.
        let encoded_prefix = (self.encoding_callback)(prefix)?;

        // Step 18. Let encoded suffix be the result of running parser’s encoding callback given suffix.
        let encoded_suffix = (self.encoding_callback)(suffix)?;

        // Step 19. Let part be a new part whose type is type, value is regexp value, modifier is modifier,
        // name is name, prefix is encoded prefix, and suffix is encoded suffix.
        let part = Part {
            part_type,
            value: regexp_value,
            modifier,
            name,
            prefix: encoded_prefix,
            suffix: encoded_suffix,
        };

        // Step 20. Append part to parser’s part list.
        self.part_list.push(part);

        Ok(())
    }

    // <https://urlpattern.spec.whatwg.org/#is-a-duplicate-name>
    fn is_a_duplicate_name(&self, name: &str) -> bool {
        // Step 1. For each part of parser’s part list:
        for part in &self.part_list {
            // Step 1.1 If part’s name is name, then return true.
            if part.name == name {
                return true;
            }
        }

        // Step 2. Return false.
        false
    }

    /// <https://urlpattern.spec.whatwg.org/#consume-text>
    fn consume_text(&mut self) -> String {
        // Step 1. Let result be the empty string.
        let mut result = String::new();

        // Step 2. While true:
        loop {
            // Step 2.1 Let token be the result of running try to consume a token given parser and "char".
            let mut token = self.try_to_consume_a_token(TokenType::Char);

            // Step 2.2 If token is null, then set token to the result of running
            // try to consume a token given parser and "escaped-char".
            if token.is_none() {
                token = self.try_to_consume_a_token(TokenType::EscapedChar);
            }

            // Step 2.3 If token is null, then break.
            let Some(token) = token else {
                break;
            };

            // Step 2.4 Append token’s value to the end of result.
            result.push_str(token.value);
        }

        result
    }
}
