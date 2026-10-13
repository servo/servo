/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell};
use std::fmt;
use std::iter::Peekable;
use std::rc::{Rc, Weak};
use std::str::Chars;

use crate::collectors::collect_webvtt_timestamp;

/// <https://w3c.github.io/webvtt/#webvtt-internal-node-object>
#[derive(Debug, Default, PartialEq)]
pub enum WebVTTNodeObjectKind {
    /// <https://w3c.github.io/webvtt/#list-of-webvtt-node-objects>
    #[default]
    List,
    /// <https://w3c.github.io/webvtt/#webvtt-class-object>
    Class,
    /// <https://w3c.github.io/webvtt/#webvtt-italic-object>
    Italic,
    /// <https://w3c.github.io/webvtt/#webvtt-bold-object>
    Bold,
    /// <https://w3c.github.io/webvtt/#webvtt-underline-object>
    Underline,
    /// <https://w3c.github.io/webvtt/#webvtt-ruby-object>
    Ruby,
    /// <https://w3c.github.io/webvtt/#webvtt-ruby-text-object>
    RubyText,
    /// <https://w3c.github.io/webvtt/#webvtt-voice-object>
    /// The string is the name of the voice
    Voice(String),
    /// <https://w3c.github.io/webvtt/#webvtt-language-object>
    Language,
    /// <https://w3c.github.io/webvtt/#webvtt-text-object>
    Text(String),
    /// <https://w3c.github.io/webvtt/#webvtt-timestamp-object>
    /// The float is the timestamp value
    Timestamp(WebVTTTimestamp),
}

/// <https://w3c.github.io/webvtt/#webvtt-timestamp>
#[derive(Debug, Default, PartialEq)]
pub struct WebVTTTimestamp(f64);

impl fmt::Display for WebVTTTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Step 1. Optionally (required if hours is non-zero):
        let time = if self.0 >= 60. * 60. {
            let hours = self.0.div_euclid(60. * 60.);
            // Step 1.1. Two or more ASCII digits,
            // representing the hours as a base ten integer.
            if hours < 10. {
                write!(f, "0")?;
            }
            // Step 1.2. A U+003A COLON character (:)
            write!(f, "{hours}:")?;
            self.0.rem_euclid(60. * 60.)
        } else {
            // In https://w3c.github.io/webvtt/#dom-construction-rules it
            // states that it should also include all optional parts. Currently
            // this is the only place where we serialize a timestamp, hence
            // for now we always write the hours
            write!(f, "00:")?;
            self.0
        };
        // Step 2. Two ASCII digits, representing the minutes as
        // a base ten integer in the range 0 ≤ minutes ≤ 59.
        let minutes = time.div_euclid(60.);
        debug_assert!(minutes < 60., "{minutes} minutes is 60 or more");
        if minutes < 10. {
            write!(f, "0")?;
        }
        // Step 3. A U+003A COLON character (:)
        write!(f, "{minutes}:")?;
        // Step 4. Two ASCII digits, representing the seconds as
        // a base ten integer in the range 0 ≤ seconds ≤ 59.
        // Step 5. A U+002E FULL STOP character (.).
        // Step 6. Three ASCII digits, representing the thousandths
        // of a second seconds-frac as a base ten integer.
        let seconds = time.rem_euclid(60.);
        debug_assert!(seconds < 60., "{seconds} seconds is 60 or more");
        if seconds < 10. {
            write!(f, "0")?;
        }
        write!(f, "{seconds:.3}")?;
        Ok(())
    }
}

/// <https://w3c.github.io/webvtt/#webvtt-node-object>
#[derive(Debug, Default)]
pub struct WebVTTNodeObject {
    /// <https://w3c.github.io/webvtt/#webvtt-node-objects-applicable-classes>
    pub applicable_classes: Vec<String>,
    /// <https://w3c.github.io/webvtt/#webvtt-node-objects-applicable-language>
    pub applicable_language: String,
    /// <https://w3c.github.io/webvtt/#webvtt-internal-node-object>
    pub kind: WebVTTNodeObjectKind,
    children: RefCell<Vec<Rc<WebVTTNodeObject>>>,
    parent: Weak<WebVTTNodeObject>,
    index: usize,
}

impl WebVTTNodeObject {
    pub fn children(&self) -> Ref<'_, Vec<Rc<WebVTTNodeObject>>> {
        self.children.borrow()
    }
}

impl PartialEq for WebVTTNodeObject {
    fn eq(&self, other: &Self) -> bool {
        // We compare all fields except parent
        self.children.eq(&other.children) &&
            self.applicable_classes.eq(&other.applicable_classes) &&
            self.applicable_language.eq(&other.applicable_language) &&
            self.kind.eq(&other.kind)
    }
}

pub trait WebVTTNodeObjectIterable {
    fn into_iter(self) -> WebVTTNodeObjectIterator;
}

impl WebVTTNodeObjectIterable for Rc<WebVTTNodeObject> {
    fn into_iter(self) -> WebVTTNodeObjectIterator {
        WebVTTNodeObjectIterator {
            root: self.clone(),
            current: Some(self),
            current_child: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum WebVTTNodeObjectIteratorDirection {
    NewChild(Rc<WebVTTNodeObject>),
    BackToParent,
}

pub struct WebVTTNodeObjectIterator {
    // To make sure that the weak pointers are kept alive
    // while iterating through all objects
    root: Rc<WebVTTNodeObject>,
    current: Option<Rc<WebVTTNodeObject>>,
    current_child: usize,
}

impl Iterator for WebVTTNodeObjectIterator {
    type Item = WebVTTNodeObjectIteratorDirection;

    fn next(&mut self) -> Option<WebVTTNodeObjectIteratorDirection> {
        let current = self.current.clone()?;
        if let Some(current_child) = current.children.borrow().get(self.current_child) {
            if matches!(current_child.kind, WebVTTNodeObjectKind::Text(_)) {
                self.current_child += 1;
            } else {
                self.current_child = 0;
                self.current = Some(current_child.clone());
            }
            return Some(WebVTTNodeObjectIteratorDirection::NewChild(
                current_child.clone(),
            ));
        }
        self.current_child = current.index + 1;
        self.current = current.parent.upgrade();
        if self.current.as_ref().is_some_and(|current| {
            current != &self.root || current.children.borrow().len() > self.current_child
        }) {
            return Some(WebVTTNodeObjectIteratorDirection::BackToParent);
        }
        None
    }
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-text-parsing-rules>
pub fn webvtt_cue_text_parsing_rules(
    input: &str,
    language: Option<String>,
) -> Rc<WebVTTNodeObject> {
    // Step 1. Let input be the string being parsed.
    // Step 2. Let position be a pointer into input,
    // initially pointing at the start of the string.
    let mut chars = input.chars().peekable();
    // Step 3. Let result be a list of WebVTT Node Objects, initially empty.
    let mut result = WebVTTNodeObject::default();
    // Step 5. Let language stack be a stack of language tags, initially empty.
    let mut language_stack = vec![];
    // Step 6. If language is set, set result’s applicable language to language,
    // and push language onto the language stack.
    if let Some(language) = language {
        result.applicable_language = language.clone();
        language_stack.push(language);
    }
    // Step 4. Let current be the WebVTT Internal Node Object result.
    let result = Rc::new(result);
    let mut current = result.clone();
    // Step 7. Loop: If position is past the end of input, return result and abort these steps.
    // Step 10. Jump to the step labeled loop.
    while chars.peek().is_some() {
        // Step 8. Let token be the result of invoking the WebVTT cue text tokenizer.
        let token = webvtt_cue_text_tokenizer(&mut chars);
        // Step 9. Run the appropriate steps given the type of token:
        match token {
            // > If token is a string
            CueTokenizerResult::String(str_) => {
                // Step 1. Create a WebVTT Text Object whose value is the value of the string token token.
                // Step 2. Append the newly created WebVTT Text Object to current.
                let mut children = current.children.borrow_mut();

                let text = Rc::new(WebVTTNodeObject {
                    kind: WebVTTNodeObjectKind::Text(str_),
                    index: children.len(),
                    parent: Rc::downgrade(&current),
                    ..Default::default()
                });
                children.push(text);
            },
            // > If token is a start tag
            CueTokenizerResult::StartTag(tag_name, applicable_classes, annotation) => {
                // > How the start tag token token is processed depends on its tag name, as follows:
                let kind = match tag_name.as_ref() {
                    // > If the tag name is "c"
                    "c" => {
                        // > Attach a WebVTT Class Object.
                        WebVTTNodeObjectKind::Class
                    },
                    // > If the tag name is "i"
                    "i" => {
                        // > Attach a WebVTT Italic Object.
                        WebVTTNodeObjectKind::Italic
                    },
                    // > If the tag name is "b"
                    "b" => {
                        // > Attach a WebVTT Bold Object.
                        WebVTTNodeObjectKind::Bold
                    },
                    // > If the tag name is "u"
                    "u" => {
                        // > Attach a WebVTT Underline Object.
                        WebVTTNodeObjectKind::Underline
                    },
                    // > If the tag name is "ruby"
                    "ruby" => {
                        // > Attach a WebVTT Ruby Object.
                        WebVTTNodeObjectKind::Ruby
                    },
                    // > If the tag name is "rt"
                    "rt" => {
                        // > If current is a WebVTT Ruby Object,
                        // > then attach a WebVTT Ruby Text Object.
                        if current.kind != WebVTTNodeObjectKind::Ruby {
                            continue;
                        }
                        WebVTTNodeObjectKind::RubyText
                    },
                    // > If the tag name is "v"
                    "v" => {
                        // > Attach a WebVTT Voice Object,
                        // > and set its value to the token’s annotation string,
                        // > or the empty string if there is no annotation string.
                        WebVTTNodeObjectKind::Voice(annotation)
                    },
                    // > If the tag name is "lang"
                    "lang" => {
                        // > Push the value of the token’s annotation string,
                        // > or the empty string if there is no annotation string,
                        // > onto the language stack; then attach a WebVTT Language Object.
                        language_stack.push(annotation);
                        WebVTTNodeObjectKind::Language
                    },
                    // > Otherwise
                    _ => {
                        // > Ignore the token.
                        continue;
                    },
                };
                // https://w3c.github.io/webvtt/#attach-a-webvtt-internal-node-object
                // Step 1. Create a new WebVTT Internal Node Object of the specified concrete class.
                current = {
                    let mut children = current.children.borrow_mut();
                    let new = WebVTTNodeObject {
                        kind,
                        // Step 2. Set the new object’s list of applicable classes
                        // to the list of classes in the token,
                        // excluding any classes that are the empty string.
                        applicable_classes,
                        // Step 3. Set the new object’s applicable language to the top entry on the language stack,
                        // if the stack is not empty.
                        applicable_language: language_stack
                            .last()
                            .map(|str_| str_.to_string())
                            .unwrap_or_default(),
                        index: children.len(),
                        parent: Rc::downgrade(&current),
                        ..Default::default()
                    };
                    // Step 4. Append the newly created node object to current.
                    let new = Rc::new(new);
                    children.push(new.clone());
                    // Step 5. Let current be the newly created node object.
                    new
                };
            },
            // > If token is an end tag
            CueTokenizerResult::EndTag(tag_name) => {
                // > If any of the following conditions is true,
                // > then let current be the parent node of current.
                let matches = match tag_name.as_ref() {
                    // > The tag name of the end tag token token is "c" and current is a WebVTT Class Object.
                    "c" => current.kind == WebVTTNodeObjectKind::Class,
                    // > The tag name of the end tag token token is "i" and current is a WebVTT Italic Object.
                    "i" => current.kind == WebVTTNodeObjectKind::Italic,
                    // > The tag name of the end tag token token is "b" and current is a WebVTT Bold Object.
                    "b" => current.kind == WebVTTNodeObjectKind::Bold,
                    // > The tag name of the end tag token token is "u" and current is a WebVTT Underline Object.
                    "u" => current.kind == WebVTTNodeObjectKind::Underline,
                    // > The tag name of the end tag token token is "ruby" and current is a WebVTT Ruby Object.
                    "ruby" => {
                        // > Otherwise, if the tag name of the end tag token token is "ruby"
                        // > and current is a WebVTT Ruby Text Object,
                        // > then let current be the parent node of the parent node of current.
                        if current.kind == WebVTTNodeObjectKind::RubyText {
                            current = current.parent.upgrade().expect("Must always have a parent");
                            true
                        } else {
                            current.kind == WebVTTNodeObjectKind::Ruby
                        }
                    },
                    // > The tag name of the end tag token token is "rt" and current is a WebVTT Ruby Text Object.
                    "rt" => current.kind == WebVTTNodeObjectKind::RubyText,
                    // > The tag name of the end tag token token is "v" and current is a WebVTT Voice Object.
                    "v" => matches!(current.kind, WebVTTNodeObjectKind::Voice(_)),
                    // > Otherwise, if the tag name of the end tag token token is "lang"
                    // > and current is a WebVTT Language Object,
                    // > then let current be the parent node of current,
                    // > and pop the top value from the language stack.
                    "lang" => {
                        let matches = current.kind == WebVTTNodeObjectKind::Language;
                        if matches {
                            language_stack.pop();
                        }
                        matches
                    },
                    // > Otherwise, ignore the token.
                    _ => false,
                };
                if matches && let Some(parent) = current.parent.upgrade() {
                    current = parent;
                }
            },
            // > If token is a timestamp tag
            CueTokenizerResult::TimestampTag(input) => {
                // Step 1. Let input be the tag value.
                //
                // That's part of the match

                // Step 2. Let position be a pointer into input, initially pointing at the start of the string.
                let mut position = input.chars().peekable();
                // Step 3. Collect a WebVTT timestamp.
                // Step 4. If that algorithm does not fail, and if position now points at the end of input
                // (i.e. there are no trailing characters after the timestamp),
                // then create a WebVTT Timestamp Object whose value is the collected time,
                // then append it to current.
                // Otherwise, ignore the token.
                if let Some(timestamp) = collect_webvtt_timestamp(position.by_ref()) &&
                    position.peek().is_none()
                {
                    let mut children = current.children.borrow_mut();
                    let new = WebVTTNodeObject {
                        kind: WebVTTNodeObjectKind::Timestamp(WebVTTTimestamp(timestamp)),
                        parent: Rc::downgrade(&current),
                        index: children.len(),
                        ..Default::default()
                    };
                    children.push(Rc::new(new));
                }
            },
        }
    }
    result
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-text-tokenizer>
#[derive(PartialEq)]
enum TokenizerState {
    WebVTTData,
    HTMLCharacterReferenceInData,
    WebVTTTag,
    WebVTTStartTag,
    WebVTTStartTagClass,
    WebVTTStartTagAnnotation,
    HTMLCharacterReferenceInAnnotation,
    WebVTTEndTag,
    WebVTTTimestampTag,
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-text-tokenizer>
enum CueTokenizerResult {
    String(String),
    StartTag(String, Vec<String>, String),
    EndTag(String),
    TimestampTag(String),
}

/// <https://w3c.github.io/webvtt/#webvtt-cue-text-tokenizer>
fn webvtt_cue_text_tokenizer(position: &mut Peekable<Chars<'_>>) -> CueTokenizerResult {
    // Step 1. Let input and position be the same variables as
    // those of the same name in the algorithm that invoked these steps.
    //
    // Passed in as arguments

    // Step 2. Let tokenizer state be WebVTT data state.
    let mut tokenizer_state = TokenizerState::WebVTTData;
    // Step 3. Let result be the empty string.
    let mut result = String::new();
    let mut buffer = String::new();
    // Step 4. Let classes be an empty list.
    let mut classes = vec![];
    loop {
        // Step 5. Loop: If position is past the end of input,
        // let c be an end-of-file marker. Otherwise, let c be the character in input pointed to by position.
        //
        // We codify the end-of-file marker as `None`
        let c = position.peek().copied();
        // Step 6. Jump to the state given by tokenizer state:
        match tokenizer_state {
            // https://w3c.github.io/webvtt/#webvtt-data-state
            TokenizerState::WebVTTData => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+0026 AMPERSAND (&)
                    Some('\u{0026}') => {
                        // > Set tokenizer state to the HTML character reference in data state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::HTMLCharacterReferenceInData;
                    },
                    // > U+003C LESS-THAN SIGN (<)
                    Some('\u{003C}') => {
                        // > If result is the empty string, then set tokenizer state
                        // > to the WebVTT tag state and jump to the step labeled next.
                        if result.is_empty() {
                            tokenizer_state = TokenizerState::WebVTTTag;
                        } else {
                            // > Otherwise, return a string token whose value is result and abort these steps.
                            return CueTokenizerResult::String(result);
                        }
                    },
                    // > End-of-file marker
                    None => {
                        // > Return a string token whose value is result and abort these steps.
                        return CueTokenizerResult::String(result);
                    },
                    // > Anything else
                    Some(c) => {
                        // > Append c to result and jump to the step labeled next.
                        result.push(c);
                    },
                }
            },
            // https://w3c.github.io/webvtt/#html-character-reference-in-data-state
            TokenizerState::HTMLCharacterReferenceInData => {
                // > Attempt to consume an HTML character reference, with no additional allowed character.
                // > If nothing is returned, append a U+0026 AMPERSAND character (&) to result.
                // > Otherwise, append the data of the character tokens that were returned to result.
                // TODO: The character reference case
                result.push('\u{0026}');
                // > Then, in any case, set tokenizer state to the WebVTT data state,
                // > and jump to the step labeled next.
                tokenizer_state = TokenizerState::WebVTTData;
            },
            // https://w3c.github.io/webvtt/#webvtt-tag-state
            TokenizerState::WebVTTTag => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+0009 CHARACTER TABULATION (tab) character
                    // > U+000A LINE FEED (LF) character
                    // > U+000C FORM FEED (FF) character
                    // > U+0020 SPACE character
                    Some('\u{0009}' | '\u{000A}' | '\u{000C}' | '\u{0020}') => {
                        // > Set tokenizer state to the WebVTT start tag annotation state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTStartTagAnnotation;
                    },
                    // > U+002E FULL STOP character (.)
                    Some('\u{002E}') => {
                        // > Set tokenizer state to the WebVTT start tag class state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTStartTagClass;
                    },
                    // > U+002F SOLIDUS character (/)
                    Some('\u{002F}') => {
                        // > Set tokenizer state to the WebVTT end tag state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTEndTag;
                    },
                    // > ASCII digits
                    Some(c) if c.is_ascii_digit() => {
                        // > Set result to c, set tokenizer state to the WebVTT timestamp tag state,
                        // > and jump to the step labeled next.
                        result = String::new();
                        result.push(c);
                        tokenizer_state = TokenizerState::WebVTTTimestampTag;
                    },
                    // > U+003E GREATER-THAN SIGN character (>)
                    Some('\u{003E}') => {
                        // > Advance position to the next character in input,
                        // > then jump to the next "end-of-file marker" entry below.
                        position.next();
                        return CueTokenizerResult::StartTag(
                            Default::default(),
                            Default::default(),
                            Default::default(),
                        );
                    },
                    // > End-of-file marker
                    None => {
                        // > Return a start tag whose tag name is the empty string,
                        // > with no classes and no annotation, and abort these steps.
                        return CueTokenizerResult::StartTag(
                            Default::default(),
                            Default::default(),
                            Default::default(),
                        );
                    },
                    // > Anything else
                    Some(c) => {
                        // > Set result to c, set tokenizer state to the WebVTT start tag state,
                        // > and jump to the step labeled next.
                        result = c.into();
                        tokenizer_state = TokenizerState::WebVTTStartTag;
                    },
                }
            },
            // https://w3c.github.io/webvtt/#webvtt-start-tag-state
            TokenizerState::WebVTTStartTag => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+0009 CHARACTER TABULATION (tab) character
                    // > U+000C FORM FEED (FF) character
                    // > U+0020 SPACE character
                    Some('\u{0009}' | '\u{000C}' | '\u{0020}') => {
                        // > Set tokenizer state to the WebVTT start tag annotation state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTStartTagAnnotation;
                    },
                    // > U+000A LINE FEED (LF) character
                    Some('\u{000A}') => {
                        // > Set buffer to c, set tokenizer state to the WebVTT start tag annotation state,
                        // > and jump to the step labeled next.
                        buffer = '\u{000A}'.into();
                        tokenizer_state = TokenizerState::WebVTTStartTagAnnotation;
                    },
                    // > U+002E FULL STOP character (.)
                    Some('\u{002E}') => {
                        // > Set tokenizer state to the WebVTT start tag class state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTStartTagClass;
                    },
                    // > U+003E GREATER-THAN SIGN character (>)
                    Some('\u{003E}') => {
                        // > Advance position to the next character in input,
                        // > then jump to the next "end-of-file marker" entry below.
                        position.next();
                        return CueTokenizerResult::StartTag(
                            result,
                            Default::default(),
                            Default::default(),
                        );
                    },
                    // > End-of-file marker
                    None => {
                        // > Return a start tag whose tag name is result,
                        // > with no classes and no annotation, and abort these steps.
                        return CueTokenizerResult::StartTag(
                            result,
                            Default::default(),
                            Default::default(),
                        );
                    },
                    // > Anything else
                    Some(c) => {
                        // > Append c to result and jump to the step labeled next.
                        result.push(c);
                    },
                }
            },
            // https://w3c.github.io/webvtt/#webvtt-start-tag-class-state
            TokenizerState::WebVTTStartTagClass => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+0009 CHARACTER TABULATION (tab) character
                    // > U+000C FORM FEED (FF) character
                    // > U+0020 SPACE character
                    Some('\u{0009}' | '\u{000C}' | '\u{0020}') => {
                        // > Append to classes an entry whose value is buffer,
                        classes.push(buffer);
                        // > set buffer to the empty string,
                        buffer = String::new();
                        // > set tokenizer state to the WebVTT start tag annotation state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTStartTagAnnotation;
                    },
                    // > U+000A LINE FEED (LF) character
                    Some('\u{000A}') => {
                        // > Append to classes an entry whose value is buffer,
                        classes.push(buffer);
                        // > set buffer to c,
                        buffer = '\u{000A}'.into();
                        // > set tokenizer state to the WebVTT start tag annotation state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::WebVTTStartTagAnnotation;
                    },
                    // > U+002E FULL STOP character (.)
                    Some('\u{002E}') => {
                        // > Append to classes an entry whose value is buffer,
                        classes.push(buffer);
                        // > set buffer to the empty string,
                        buffer = String::new();
                        // > and jump to the step labeled next.
                    },
                    // > U+003E GREATER-THAN SIGN character (>)
                    Some('\u{003E}') => {
                        // > Advance position to the next character in input,
                        // > then jump to the next "end-of-file marker" entry below.
                        position.next();
                        classes.push(buffer);
                        return CueTokenizerResult::StartTag(result, classes, String::new());
                    },
                    // > End-of-file marker
                    None => {
                        // > Append to classes an entry whose value is buffer,
                        classes.push(buffer);
                        // > then return a start tag whose tag name is result,
                        // > with the classes given in classes but no annotation,
                        // > and abort these steps.
                        return CueTokenizerResult::StartTag(result, classes, String::new());
                    },
                    // > Anything else
                    Some(c) => {
                        // > Append c to buffer and jump to the step labeled next.
                        buffer.push(c);
                    },
                }
            },
            // https://w3c.github.io/webvtt/#webvtt-start-tag-annotation-state
            TokenizerState::WebVTTStartTagAnnotation => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+0026 AMPERSAND (&)
                    Some('\u{0026}') => {
                        // > Set tokenizer state to the HTML character reference in annotation state,
                        // > and jump to the step labeled next.
                        tokenizer_state = TokenizerState::HTMLCharacterReferenceInAnnotation;
                    },
                    // > U+003E GREATER-THAN SIGN character (>)
                    Some('\u{003E}') => {
                        // > Advance position to the next character in input,
                        // > then jump to the next "end-of-file marker" entry below.
                        position.next();
                        return CueTokenizerResult::StartTag(
                            result,
                            classes,
                            buffer.trim().to_owned(),
                        );
                    },
                    // > End-of-file marker
                    None => {
                        // > Remove any leading or trailing ASCII whitespace characters from buffer,
                        // > and replace any sequence of one or more consecutive ASCII whitespace characters
                        // > in buffer with a single U+0020 SPACE character;
                        // > then, return a start tag whose tag name is result,
                        // > with the classes given in classes, and with buffer as the annotation,
                        // > and abort these steps.
                        // TODO: consecutive whitespace replacing
                        return CueTokenizerResult::StartTag(
                            result,
                            classes,
                            buffer.trim().to_owned(),
                        );
                    },
                    // > Anything else
                    Some(c) => {
                        // > Append c to buffer and jump to the step labeled next.
                        buffer.push(c);
                    },
                }
            },
            TokenizerState::HTMLCharacterReferenceInAnnotation => {
                // > Attempt to consume an HTML character reference,
                // > with the additional allowed character being U+003E GREATER-THAN SIGN (>).
                // > If nothing is returned, append a U+0026 AMPERSAND character (&) to buffer.
                // > Otherwise, append the data of the character tokens that were returned to buffer.
                // TODO: The character reference case
                buffer.push('\u{0026}');
                // > Then, in any case, set tokenizer state to the WebVTT start tag annotation state,
                // > and jump to the step labeled next.
                tokenizer_state = TokenizerState::WebVTTStartTagAnnotation;
            },
            // https://w3c.github.io/webvtt/#webvtt-end-tag-state
            TokenizerState::WebVTTEndTag => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+003E GREATER-THAN SIGN character (>)
                    Some('\u{003E}') => {
                        // > Advance position to the next character in input,
                        // > then jump to the next "end-of-file marker" entry below.
                        position.next();
                        return CueTokenizerResult::EndTag(result);
                    },
                    // > End-of-file marker
                    None => {
                        // > Return an end tag whose tag name is result and abort these steps.
                        return CueTokenizerResult::EndTag(result);
                    },
                    // > Anything else
                    Some(c) => {
                        // > Append c to result and jump to the step labeled next.
                        result.push(c);
                    },
                }
            },
            // https://w3c.github.io/webvtt/#webvtt-timestamp-tag-state
            TokenizerState::WebVTTTimestampTag => {
                // > Jump to the entry that matches the value of c:
                match c {
                    // > U+003E GREATER-THAN SIGN character (>)
                    Some('\u{003E}') => {
                        // > Advance position to the next character in input,
                        // > then jump to the next "end-of-file marker" entry below.
                        position.next();
                        return CueTokenizerResult::TimestampTag(result);
                    },
                    // > End-of-file marker
                    None => {
                        // > Return a timestamp tag whose tag name is result and abort these steps.
                        return CueTokenizerResult::TimestampTag(result);
                    },
                    // > Anything else
                    Some(c) => {
                        // > Append c to result and jump to the step labeled next.
                        result.push(c);
                    },
                }
            },
        }
        // Step 7. Next: Advance position to the next character in input.
        position.next();
        // Step 8. Jump to the step labeled loop.
    }
}

#[cfg(any(test, feature = "test-util"))]
impl WebVTTNodeObjectIterator {
    pub fn assert_and_return_next_child(&mut self) -> Rc<WebVTTNodeObject> {
        let Some(iterator_direction) = self.next() else {
            unreachable!("Must have a new item");
        };
        let WebVTTNodeObjectIteratorDirection::NewChild(node) = iterator_direction else {
            unreachable!("Must be a new child");
        };
        node
    }

    pub fn next_is_parent(&mut self) -> bool {
        let Some(iterator_direction) = self.next() else {
            unreachable!("Must have a new item");
        };
        iterator_direction == WebVTTNodeObjectIteratorDirection::BackToParent
    }
}

#[cfg(test)]
mod tests {
    use crate::cue::text::WebVTTTimestamp;
    use crate::shared_test_setup::compute_result_in_seconds;

    #[test]
    fn test_formats_timestamp_exact_hour_correctly() {
        let timestamp = WebVTTTimestamp(compute_result_in_seconds(1., 0., 0., 0.));

        assert_eq!(format!("{timestamp}"), "01:00:00.000");
    }

    #[test]
    fn test_formats_timestamp_hour_plus_one_minute_correctly() {
        let timestamp = WebVTTTimestamp(compute_result_in_seconds(1., 1., 0., 0.));

        assert_eq!(format!("{timestamp}"), "01:01:00.000");
    }
}
