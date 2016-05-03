/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::servohtmlparser::ServoHTMLParser;
use dom::servoxmlparser::ServoXMLParser;
use dom::window::Window;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::ptr;

pub mod html;
pub mod xml;

pub trait Parser {
    fn parse_chunk(self, input: String);
    fn finish(self);
}

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub enum ParserField {
    HTML(JS<ServoHTMLParser>),
    XML(JS<ServoXMLParser>),
}

#[must_root]
#[derive(JSTraceable, HeapSizeOf)]
pub struct MutNullableParserField {
    #[ignore_heap_size_of = "XXXjdm"]
    ptr: UnsafeCell<Option<ParserField>>,
}

impl Default for MutNullableParserField {
    #[allow(unrooted_must_root)]
    fn default() -> MutNullableParserField {
        MutNullableParserField {
            ptr: UnsafeCell::new(None),
        }
    }
}

impl MutNullableParserField {
    #[allow(unsafe_code)]
    pub fn set(&self, val: Option<ParserRef>) {
        unsafe {
            *self.ptr.get() = val.map(|val| {
                match val {
                    ParserRef::HTML(parser) => ParserField::HTML(JS::from_ref(parser)),
                    ParserRef::XML(parser) => ParserField::XML(JS::from_ref(parser)),
                }
            });
        }
    }

    #[allow(unsafe_code, unrooted_must_root)]
    pub fn get(&self) -> Option<ParserRoot> {
        unsafe {
            ptr::read(self.ptr.get()).map(|o| {
                match o {
                    ParserField::HTML(parser) => ParserRoot::HTML(Root::from_ref(&*parser)),
                    ParserField::XML(parser) => ParserRoot::XML(Root::from_ref(&*parser)),
                }
            })
        }
    }
}

pub enum ParserRoot {
    HTML(Root<ServoHTMLParser>),
    XML(Root<ServoXMLParser>),
}

impl ParserRoot {
    pub fn r(&self) -> ParserRef {
        match *self {
            ParserRoot::HTML(ref parser) => ParserRef::HTML(parser.r()),
            ParserRoot::XML(ref parser) => ParserRef::XML(parser.r()),
        }
    }
}

pub enum TrustedParser {
    HTML(Trusted<ServoHTMLParser>),
    XML(Trusted<ServoXMLParser>),
}

impl TrustedParser {
    pub fn root(&self) -> ParserRoot {
        match *self {
            TrustedParser::HTML(ref parser) => ParserRoot::HTML(parser.root()),
            TrustedParser::XML(ref parser) => ParserRoot::XML(parser.root()),
        }
    }
}

pub enum ParserRef<'a> {
    HTML(&'a ServoHTMLParser),
    XML(&'a ServoXMLParser),
}

impl<'a> ParserRef<'a> {
    pub fn parse_chunk(&self, input: String) {
        match *self {
            ParserRef::HTML(parser) => parser.parse_chunk(input),
            ParserRef::XML(parser) => parser.parse_chunk(input),
        }
    }

    pub fn window(&self) -> &Window {
        match *self {
            ParserRef::HTML(parser) => parser.window(),
            ParserRef::XML(parser) => parser.window(),
        }
    }

    pub fn resume(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.resume(),
            ParserRef::XML(parser) => parser.resume(),
        }
    }

    pub fn suspend(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.suspend(),
            ParserRef::XML(parser) => parser.suspend(),
        }
    }

    pub fn is_suspended(&self) -> bool {
        match *self {
            ParserRef::HTML(parser) => parser.is_suspended(),
            ParserRef::XML(parser) => parser.is_suspended(),
        }
    }

    pub fn pending_input(&self) -> &DOMRefCell<Vec<String>> {
        match *self {
            ParserRef::HTML(parser) => parser.pending_input(),
            ParserRef::XML(parser) => parser.pending_input(),
        }
    }

    pub fn set_plaintext_state(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.set_plaintext_state(),
            ParserRef::XML(parser) => parser.set_plaintext_state(),
        }
    }

    pub fn parse_sync(&self) {
        match *self {
            ParserRef::HTML(parser) => parser.parse_sync(),
            ParserRef::XML(parser) => parser.parse_sync(),
        }
    }

    pub fn document(&self) -> &Document {
        match *self {
            ParserRef::HTML(parser) => parser.document(),
            ParserRef::XML(parser) => parser.document(),
        }
    }

    pub fn last_chunk_received(&self) -> &Cell<bool> {
        match *self {
            ParserRef::HTML(parser) => parser.last_chunk_received(),
            ParserRef::XML(parser) => parser.last_chunk_received(),
        }
    }
}

