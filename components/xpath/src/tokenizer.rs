/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::{is_valid_continuation, is_valid_start};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A variable reference (like `$foo`) failed to parse.
    InvalidVariableReference,
    InvalidNCName,
    ExpectedOperator,
    UnterminatedStringLiteral,
    IllegalCharacter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CNameToken<'a> {
    pub(crate) prefix: Option<&'a str>,
    pub(crate) local_name: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OperatorToken {
    And,
    Or,
    Multiply,
    Modulo,
    Divide,
    Add,
    Subtract,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LiteralToken<'a> {
    Integer(i64),
    Decimal(f64),
    String(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Token<'a> {
    VariableReference(&'a str),
    CName(CNameToken<'a>),
    Operator(OperatorToken),
    Literal(LiteralToken<'a>),
    /// e.g. `child::`
    AxisIdentifier(&'a str),
    /// `..`
    ParentNode,
    /// `.`
    SelfNode,
    /// `/`
    Parent,
    /// `//`
    Ancestor,
    /// `foo(`
    FunctionCall(&'a str),
    /// `(`
    OpeningParenthesis,
    /// `)`
    ClosingParenthesis,
    /// `[`
    OpeningBracket,
    /// `]`
    ClosingBracket,
    /// `,`
    Comma,
    /// `@`
    AtSign,
    /// `processing-instruction(`
    ProcessingInstructionTest,
    /// `comment(`
    CommentTest,
    /// `node(`
    NodeTest,
    /// `text(`
    TextTest,
    /// `|`
    Union,
}

struct Tokenizer<'a> {
    remaining: &'a str,
}

impl<'a> Tokenizer<'a> {
    /// If the result is `Err(_)` then `self.remaining` is unchanged.
    fn consume_ncname(&mut self, allow_wildcard: bool) -> Result<&'a str, Error> {
        if allow_wildcard && self.remaining.starts_with('*') {
            self.remaining = &self.remaining[1..];
            return Ok("*");
        }

        let mut chars = self.remaining.char_indices();

        if !chars
            .next()
            .is_some_and(|(_, character)| is_valid_start(character) && character != ':')
        {
            return Err(Error::InvalidNCName);
        }

        let name_end = chars
            .find(|(_, character)| !is_valid_continuation(*character) || *character == ':')
            .map(|(index, _)| index)
            .unwrap_or(self.remaining.len());

        let (ncname, remaining) = self.remaining.split_at(name_end);
        self.remaining = remaining;
        Ok(ncname)
    }

    /// Parses a single token from the beginning and updates the remaining input accordingly.
    ///
    /// ## Panics
    /// Panics when the remaining input is empty.
    fn consume_single_token(&mut self, expect_operator_token: bool) -> Result<Token<'a>, Error> {
        if self.remaining.starts_with('$') {
            self.remaining = &self.remaining[1..];
            let variable_name = self
                .consume_ncname(false)
                .map_err(|_| Error::InvalidVariableReference)?;
            return Ok(Token::VariableReference(variable_name));
        }

        if let Ok(ncname) = self.consume_ncname(true) {
            if expect_operator_token {
                return match_operator_name(ncname).map(Token::Operator);
            }

            if self.remaining.starts_with(':') {
                self.remaining = &self.remaining[1..];
                if self.remaining.starts_with(':') {
                    // This is an axis identifier
                    self.remaining = &self.remaining[1..];
                    return Ok(Token::AxisIdentifier(ncname));
                }

                // The previous name was the prefix of a qualified name (foo:bar)
                return Ok(Token::CName(CNameToken {
                    prefix: Some(ncname),
                    local_name: self.consume_ncname(true)?,
                }));
            } else if self.remaining.starts_with('(') {
                self.remaining = &self.remaining[1..];
                let token = match ncname {
                    "processing-instruction" => Token::ProcessingInstructionTest,
                    "node" => Token::NodeTest,
                    "text" => Token::TextTest,
                    "comment" => Token::CommentTest,
                    _ => Token::FunctionCall(ncname),
                };
                return Ok(token);
            } else {
                return Ok(Token::CName(CNameToken {
                    prefix: None,
                    local_name: ncname,
                }));
            }
        }

        match self
            .remaining
            .chars()
            .next()
            .expect("consume_single_token called with empty input")
        {
            '0'..='9' => {
                let number = self.consume_numeric_literal();
                Ok(Token::Literal(number))
            },
            '\'' | '"' => {
                let string = self.consume_string_literal()?;
                Ok(Token::Literal(LiteralToken::String(string)))
            },
            '.' => {
                // This is tricky: A period can either be
                // the parent node (".."), a numeric literal (".123") or
                // self-node (".").
                match self.remaining.chars().nth(1) {
                    Some('0'..='9') => Ok(Token::Literal(self.consume_numeric_literal())),
                    Some('.') => {
                        self.remaining = &self.remaining[2..];
                        Ok(Token::ParentNode)
                    },
                    _ => {
                        self.remaining = &self.remaining[1..];
                        Ok(Token::SelfNode)
                    },
                }
            },
            '/' => {
                if self.remaining.chars().nth(1).is_some_and(|c| c == '/') {
                    self.remaining = &self.remaining[2..];
                    Ok(Token::Ancestor)
                } else {
                    self.remaining = &self.remaining[1..];
                    Ok(Token::Parent)
                }
            },
            '-' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::Operator(OperatorToken::Subtract))
            },
            '(' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::OpeningParenthesis)
            },
            ')' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::ClosingParenthesis)
            },
            '[' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::OpeningBracket)
            },
            ']' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::ClosingBracket)
            },
            ',' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::Comma)
            },
            '@' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::AtSign)
            },
            '<' => {
                self.remaining = &self.remaining[1..];
                if self.remaining.starts_with('=') {
                    self.remaining = &self.remaining[1..];
                    Ok(Token::Operator(OperatorToken::LessThanOrEqual))
                } else {
                    Ok(Token::Operator(OperatorToken::LessThan))
                }
            },
            '>' => {
                self.remaining = &self.remaining[1..];
                if self.remaining.starts_with('=') {
                    self.remaining = &self.remaining[1..];
                    Ok(Token::Operator(OperatorToken::GreaterThanOrEqual))
                } else {
                    Ok(Token::Operator(OperatorToken::GreaterThan))
                }
            },
            '!' => {
                if self.remaining.starts_with("!=") {
                    self.remaining = &self.remaining[2..];
                    Ok(Token::Operator(OperatorToken::NotEqual))
                } else {
                    Err(Error::IllegalCharacter)
                }
            },
            '=' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::Operator(OperatorToken::Equal))
            },
            '|' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::Union)
            },
            '+' => {
                self.remaining = &self.remaining[1..];
                Ok(Token::Operator(OperatorToken::Add))
            },
            other => {
                log::debug!("Illegal character: {other:?}");
                Err(Error::IllegalCharacter)
            },
        }
    }

    fn consume_string_literal(&mut self) -> Result<&'a str, Error> {
        let quote_character = self.remaining.chars().next().unwrap();
        debug_assert!(quote_character == '\'' || quote_character == '"');
        let Some((literal, remaining)) = self.remaining[1..].split_once(quote_character) else {
            return Err(Error::UnterminatedStringLiteral);
        };
        self.remaining = remaining;
        Ok(literal)
    }

    /// <https://www.w3.org/TR/1999/REC-xpath-19991116/#NT-Number>
    fn consume_numeric_literal(&mut self) -> LiteralToken<'a> {
        let mut has_period = false;
        let mut end = self.remaining.len();
        for (index, c) in self.remaining.char_indices() {
            let is_first_period = !has_period && c == '.';
            if !c.is_ascii_digit() && !is_first_period {
                end = index;
                break;
            }

            has_period |= c == '.';
        }

        let (mut number, remaining) = self.remaining.split_at(end);
        debug_assert!(
            !(number.is_empty() || number == "."),
            "Why did we even try to parse this as a literal",
        );
        self.remaining = remaining;

        // Treat the literal as a float iff it has a period character
        // that is not at the very end.
        let mut is_integer_literal = !has_period;
        if let Some(integer_literal) = number.strip_suffix('.') {
            number = integer_literal;
            is_integer_literal = true;
        };

        // FIXME: When the literal is negated, use a negative number in case
        // of a parsing error.
        if is_integer_literal {
            let value = number
                .parse()
                .inspect_err(|error| {
                    log::warn!(
                        "Failed to parse numeric literal ({number:?}) that looked valid: {error:?}"
                    )
                })
                .unwrap_or(i64::MAX);
            LiteralToken::Integer(value)
        } else {
            let value = number
                .parse()
                .inspect_err(|error| {
                    log::warn!(
                        "Failed to parse numeric literal ({number:?}) that looked valid: {error:?}"
                    )
                })
                .unwrap_or(f64::NAN);
            LiteralToken::Decimal(value)
        }
    }

    fn skip_whitespace(&mut self) {
        self.remaining = self
            .remaining
            .trim_start_matches(|c: char| c.is_ascii_whitespace());
    }
}

fn match_operator_name(operator_name: &str) -> Result<OperatorToken, Error> {
    let operator = match operator_name {
        "and" => OperatorToken::And,
        "or" => OperatorToken::Or,
        "mod" => OperatorToken::Modulo,
        "div" => OperatorToken::Divide,
        "*" => OperatorToken::Multiply,
        _ => {
            log::debug!("Expected Operator, found {operator_name:?}");
            return Err(Error::ExpectedOperator);
        },
    };

    Ok(operator)
}

impl OperatorToken {
    /// Return a handle that can be used to compare two [OperatorToken]s in terms of precedence (binding order).
    pub(crate) fn precedence(&self) -> impl Ord {
        match self {
            Self::Or => 0,
            Self::And => 1,
            Self::Equal | Self::NotEqual => 2,
            Self::LessThan |
            Self::LessThanOrEqual |
            Self::GreaterThan |
            Self::GreaterThanOrEqual => 3,
            Self::Add | Self::Subtract => 4,
            Self::Multiply | Self::Divide | Self::Modulo => 5,
        }
    }
}

impl<'a> Token<'a> {
    pub(crate) fn is_start_of_location_step(&self) -> bool {
        matches!(
            self,
            Self::AxisIdentifier(_) |
                Self::AtSign |
                Self::ParentNode |
                Self::SelfNode |
                Self::CName(_) |
                Self::CommentTest |
                Self::NodeTest |
                Self::ProcessingInstructionTest |
                Self::TextTest
        )
    }

    /// Used to implement the first bullet point of <https://www.w3.org/TR/1999/REC-xpath-19991116/#exprlex>.
    fn followed_by_operator(&self) -> bool {
        matches!(
            self,
            Self::Literal(_) |
                Self::CName(_) |
                Self::VariableReference(_) |
                Self::ParentNode |
                Self::SelfNode |
                Self::ClosingBracket |
                Self::ClosingParenthesis
        )
    }
}

pub(crate) fn tokenize(input: &str) -> Result<Vec<Token<'_>>, Error> {
    let mut tokenizer = Tokenizer { remaining: input };
    let mut tokens: Vec<Token> = vec![];

    // https://www.w3.org/TR/1999/REC-xpath-19991116/#exprlex:
    // > If there is a preceding token and the preceding token is not one of @, ::, (, [, ,
    // > or an Operator, then a * must be recognized as a MultiplyOperator and an NCName
    // > must be recognized as an OperatorName.
    let mut expect_operator_token = false;

    tokenizer.skip_whitespace();
    while !tokenizer.remaining.is_empty() {
        let token = tokenizer.consume_single_token(expect_operator_token)?;
        tokens.push(token);
        expect_operator_token = token.followed_by_operator();
        tokenizer.skip_whitespace();
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_name_without_prefix() {
        let mut tokenizer = Tokenizer { remaining: "foo" };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::CName(CNameToken {
                prefix: None,
                local_name: "foo"
            }))
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_name_with_prefix() {
        let mut tokenizer = Tokenizer {
            remaining: "foo:bar",
        };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::CName(CNameToken {
                prefix: Some("foo"),
                local_name: "bar"
            }))
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_name_with_wildcard_prefix() {
        let mut tokenizer = Tokenizer { remaining: "*:bar" };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::CName(CNameToken {
                prefix: Some("*"),
                local_name: "bar"
            }))
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_name_with_wildcard_local_name() {
        let mut tokenizer = Tokenizer { remaining: "*" };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::CName(CNameToken {
                prefix: None,
                local_name: "*"
            }))
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_variable_reference() {
        let mut tokenizer = Tokenizer {
            remaining: "$servo",
        };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::VariableReference("servo"))
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_floating_point_literal() {
        let mut tokenizer = Tokenizer { remaining: "13.5" };
        assert_eq!(
            tokenizer.consume_numeric_literal(),
            LiteralToken::Decimal(13.5)
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_floating_point_literal_without_leading_digit() {
        let mut tokenizer = Tokenizer { remaining: ".42" };
        assert_eq!(
            tokenizer.consume_numeric_literal(),
            LiteralToken::Decimal(0.42)
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_floating_point_literal_that_can_be_optimized_to_integer_literal() {
        let mut tokenizer = Tokenizer { remaining: "42." };
        assert_eq!(
            tokenizer.consume_numeric_literal(),
            LiteralToken::Integer(42)
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_integer_literal() {
        let mut tokenizer = Tokenizer { remaining: "12" };
        assert_eq!(
            tokenizer.consume_numeric_literal(),
            LiteralToken::Integer(12)
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_function_name() {
        let mut tokenizer = Tokenizer { remaining: "foo(" };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::FunctionCall("foo"))
        );
        assert!(tokenizer.remaining.is_empty());
    }

    #[test]
    fn parse_axis_identifier() {
        let mut tokenizer = Tokenizer { remaining: "foo::" };
        assert_eq!(
            tokenizer.consume_single_token(false),
            Ok(Token::AxisIdentifier("foo"))
        );
        assert!(tokenizer.remaining.is_empty());
    }
}
