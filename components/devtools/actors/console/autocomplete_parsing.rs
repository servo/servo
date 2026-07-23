/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::{Duration, SystemTime};

use regex::regex;
use unicode_segmentation::UnicodeSegmentation;

const TIMEOUT: Duration = Duration::from_millis(2500);
const OPEN_BODY: &str = "{[(";
const CLOSE_BODY: &str = "}])";
const NO_AUTOCOMPLETE_PREFIXES: [&str; 6] = ["var", "const", "let", "function", "class", "using"];
const OPERATOR_CHARS_SET: &str = ";,:=<>+-*%|&^~!";

pub(super) struct ParserAnalysis<'a> {
    state: ParserEndState,
    /// The last statement in the string
    pub last_statement: String,
    /// Whether last_statement has an open element access (e.g. `x["match`).
    pub is_element_access: bool,
    /// Whether we are accessing property (e.g `true` in `var a = {b: 1};a.b`)
    pub is_property_access: bool,
    /// The part of the expression that should match the properties on the mainExpression
    /// E.g. `que` when expression is `document.body.que`
    pub match_prop: Option<String>,
    /// The part of the expression before any property access
    /// E.g. `a.b` if expression is `a.b.`
    pub main_expression: String,
    /// The part of the expression before property access
    /// E.g `var a = {b: 1};a` if expression is `var a = {b: 1};a.b`
    pub expression_before_property_access: &'a str,
}

#[derive(PartialEq, Clone, Copy)]
pub(super) enum ParserState {
    Normal,
    Quote,
    Dquote,
    TemplateLiteral,
    EscapeQuote,
    EscapeDquote,
    EscapeTemplateLiteral,
    Slash,
    InlineComment,
    MultilineComment,
    MultilineCommentClose,
    QuestionMark,
}

#[derive(PartialEq, Clone, Copy)]
pub(super) enum ParserEndState {
    Normal,
    Quote,
    Dquote,
}

impl From<ParserState> for ParserEndState {
    fn from(value: ParserState) -> Self {
        match value {
            ParserState::Quote => ParserEndState::Quote,
            ParserState::Dquote => ParserEndState::Dquote,
            _ => ParserEndState::Normal,
        }
    }
}

pub(super) fn analyze_autocomplete_input_string(string: &str) -> Result<ParserAnalysis<'_>, ()> {
    struct StackElement<'a> {
        token: &'a str,
        last_statement: String,
        index: usize,
    }

    let mut body_stack: Vec<StackElement> = Vec::new();
    let mut state = ParserState::Normal;
    let mut previous_non_whitespace_char: Option<&str> = None;
    let mut last_statement = String::new();
    let mut current_index = -1isize;
    let mut dot_index: Option<usize> = None;
    let mut pending_whitespace = String::new();
    let starting_time = SystemTime::now();

    for c in string.graphemes(true) {
        if starting_time.elapsed().is_ok_and(|d| d > TIMEOUT) {
            return Err(());
        }

        current_index += c.len() as isize;
        let mut reset_last_statement = false;
        let is_whitespace = c.trim().is_empty();

        if state == ParserState::Slash {
            if c == "/" {
                state = ParserState::InlineComment;
                continue;
            } else if c == "*" {
                state = ParserState::MultilineComment;
                continue;
            } else {
                last_statement.clear();
                state = ParserState::Normal;
            }
        }

        match state {
            ParserState::Slash => {
                // Already handled
            },
            ParserState::Normal => {
                if last_statement.ends_with("?.") &&
                    c.chars().next().is_some_and(|c1| c1.is_ascii_digit())
                {
                    // If the current char is a number, the engine will consider we're not
                    // performing an optional chaining, but a ternary (e.g. x ?.4 : 2).
                    last_statement.clear();
                }

                if c == "." {
                    dot_index = Some(current_index as usize)
                }

                // If the last characters were spaces, and the current one is not.
                if !pending_whitespace.is_empty() && !is_whitespace {
                    // If we have a legitimate property/element access, or potential optional
                    // chaining call, we append the spaces.
                    if c == "[" || c == "." || c == "?" {
                        last_statement.extend(pending_whitespace.drain(..));
                    } else {
                        // If not, we can be sure the statement was over, and we can start a new one.
                        last_statement.clear();
                        pending_whitespace.clear();
                    }
                }

                if c == "\"" {
                    state = ParserState::Dquote
                } else if c == "'" {
                    state = ParserState::Quote
                } else if c == "`" {
                    state = ParserState::TemplateLiteral
                } else if c == "/" {
                    state = ParserState::Slash
                } else if c == "?" {
                    state = ParserState::QuestionMark
                } else if OPERATOR_CHARS_SET.contains(c) {
                    // If the character is an operator, we can update the current statement.
                    reset_last_statement = true;
                } else if is_whitespace {
                    // If the previous char isn't a dot or opening bracket, and the current computed
                    // statement is not a variable/function/class declaration, we track the number
                    // of consecutive spaces, so we can re-use them at some point (or drop them).
                    if previous_non_whitespace_char != Some(".") &&
                        previous_non_whitespace_char != Some("[") &&
                        NO_AUTOCOMPLETE_PREFIXES.contains(&last_statement.as_str())
                    {
                        pending_whitespace += c;
                        continue;
                    }
                } else if OPEN_BODY.contains(c) {
                    // When opening a bracket or a parens, we store the current statement, in order
                    // to be able to retrieve it later.
                    body_stack.push(StackElement {
                        token: c,
                        last_statement: last_statement.clone(),
                        index: current_index as usize,
                    });
                    // And we compute a new statement.
                    reset_last_statement = true;
                } else if CLOSE_BODY.contains(c) {
                    let last = body_stack.pop();
                    if let Some(last) = last &&
                        open_close_body(last.token) == Some(c)
                    {
                        if c == "}" {
                            reset_last_statement = true;
                        } else {
                            last_statement = last.last_statement
                        }
                    } else {
                        // Syntax error
                        return Err(());
                    }
                }
            },

            // Escaped quote
            ParserState::EscapeQuote => state = ParserState::Quote,
            ParserState::EscapeDquote => state = ParserState::Dquote,
            ParserState::EscapeTemplateLiteral => state = ParserState::TemplateLiteral,

            ParserState::Quote => {
                if c == "\\" {
                    state = ParserState::EscapeQuote
                } else if c == "\n" {
                    // unterminated string literal
                    return Err(());
                } else if c == "'" {
                    state = ParserState::Normal;
                }
            },
            ParserState::Dquote => {
                if c == "\\" {
                    state = ParserState::EscapeDquote
                } else if c == "\n" {
                    // unterminated string literal
                    return Err(());
                } else if c == "\"" {
                    state = ParserState::Normal;
                }
            },
            ParserState::TemplateLiteral => {
                if c == "\\" {
                    state = ParserState::EscapeTemplateLiteral
                } else if c == "`" {
                    state = ParserState::Normal;
                }
            },
            ParserState::InlineComment => {
                if c == "\n" {
                    state = ParserState::Normal;
                    reset_last_statement = true;
                }
            },
            ParserState::MultilineComment => {
                if c == "*" {
                    state = ParserState::MultilineCommentClose;
                }
            },
            ParserState::MultilineCommentClose => {
                if c == "/" {
                    state = ParserState::Normal;
                    reset_last_statement = true;
                } else {
                    state = ParserState::MultilineComment;
                }
            },
            ParserState::QuestionMark => {
                state = ParserState::Normal;
                if c == "?" {
                    // If we have a nullish coalescing operator, we start a new statement
                    reset_last_statement = true;
                } else if c != "." {
                    // If we're not dealing with optional chaining (?.), it means we have a ternary,
                    // so we are starting a new statement that includes the current character.
                    last_statement.clear();
                } else {
                    dot_index = Some(current_index as usize);
                }
            },
        }

        if !is_whitespace {
            previous_non_whitespace_char = Some(c);
        }
        if reset_last_statement {
            last_statement.clear();
        } else {
            last_statement += c;
        }

        // We update all the open stacks lastStatement so they are up-to-date.
        body_stack.iter_mut().for_each(|element| {
            if element.token == "}" {
                element.last_statement += c;
            }
        });
    }

    let mut is_element_access = false;
    let mut last_opening_bracket_index: Option<usize> = None;
    if let Some(element) = body_stack.first() &&
        body_stack.len() == 1 &&
        element.token == "["
    {
        last_statement = element.last_statement.clone();
        last_opening_bracket_index = Some(element.index);
        is_element_access = true;

        if state == ParserState::Dquote ||
            state == ParserState::Quote ||
            state == ParserState::TemplateLiteral ||
            state == ParserState::EscapeQuote ||
            state == ParserState::EscapeDquote ||
            state == ParserState::EscapeTemplateLiteral
        {
            state = ParserState::Normal
        }
    } else if !pending_whitespace.is_empty() {
        last_statement.clear();
    }

    let last_completion_char_index = if is_element_access {
        last_opening_bracket_index
    } else {
        dot_index
    };

    let last_completion_char_range = 0..last_completion_char_index.unwrap_or(usize::MAX);
    let string_before_last_completion_char = &string[last_completion_char_range];

    let is_property_access = last_completion_char_index.is_some_and(|i| i > 0);

    // Compute `isOptionalAccess`, so that we can use it
    // later for computing `expressionBeforePropertyAccess`.
    // Check `?.` before `[` for element access ( e.g `a?.["b` or `a  ?. ["b` )
    // and `?` before `.` for regular property access ( e.g `a?.b` or `a ?. b` )
    let optional_element_access_regex = regex!(r"\?\.\s*$");
    let is_optional_access = if is_element_access {
        optional_element_access_regex.is_match(string_before_last_completion_char)
    } else if is_property_access {
        let index = last_completion_char_index.unwrap();
        is_property_access && &string[index - 1..index + 1] == ".?"
    } else {
        false
    };

    // Get the filtered string for the properties (e.g if `document.qu` then `qu`)
    let match_prop: Option<String> = if is_property_access {
        let slice = &string[last_completion_char_index.unwrap() + 1..];
        Some(slice.trim_start().to_string())
    } else {
        None
    };

    let expression_before_property_access = if is_property_access {
        // For optional access, we can take all the chars before the last "?" char.
        let end_index = if is_optional_access {
            string_before_last_completion_char
                .rfind('?')
                .expect("Optional access must have '?'")
        } else {
            last_completion_char_index.unwrap_or(usize::MAX)
        };
        &string_before_last_completion_char[0..end_index]
    } else {
        string
    };

    let mut main_expression = last_statement.clone();
    if is_property_access {
        if is_optional_access {
            // Strip anything before the last `?`.
            let index = string_before_last_completion_char
                .rfind('?')
                .expect("Optional access must have '?'");
            main_expression = (main_expression[0..index]).to_string()
        } else {
            let drop_count = string.len() - last_completion_char_index.unwrap_or(0);
            main_expression = (main_expression[0..(string.len() - drop_count)]).to_string()
        }
    }

    Ok(ParserAnalysis {
        state: state.into(),
        last_statement,
        is_property_access,
        is_element_access,
        match_prop,
        main_expression,
        expression_before_property_access,
    })
}

fn open_close_body(open: &str) -> Option<&str> {
    match open {
        "{" => Some("}"),
        "[" => Some("]"),
        "(" => Some(")"),
        _ => None,
    }
}

impl ParserAnalysis<'_> {
    pub(super) fn should_be_autocompleted(&self) -> bool {
        // If the current state is not STATE_NORMAL, then we are inside string,
        // which means that no completion is possible.
        if self.state != ParserEndState::Normal {
            return false;
        }

        // Don't complete on just an empty string.
        if self.last_statement.trim() == "" {
            return false;
        }

        if NO_AUTOCOMPLETE_PREFIXES
            .into_iter()
            .any(|prefix| self.last_statement.starts_with(&(prefix.to_owned() + " ")))
        {
            return false;
        }

        true
    }
}
