/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentReadyState;
use dom::bindings::codegen::Bindings::DOMParserBinding;
use dom::bindings::codegen::Bindings::DOMParserBinding::DOMParserMethods;
use dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::{Text_html, Text_xml};
use dom::bindings::error::Fallible;
use dom::bindings::error::Error::FailureUnknown;
use dom::bindings::global::GlobalRef;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::document::{Document, DocumentHelpers, IsHTMLDocument};
use dom::document::DocumentSource;
use dom::servohtmlparser::ServoHTMLParser;
use dom::window::Window;
use parse::Parser;
use servo_util::str::DOMString;

#[dom_struct]
pub struct DOMParser {
    window: JS<Window>, //XXXjdm Document instead?
    reflector_: Reflector
}

impl DOMParser {
    fn new_inherited(window: JSRef<Window>) -> DOMParser {
        DOMParser {
            window: JS::from_rooted(window),
            reflector_: Reflector::new()
        }
    }

    pub fn new(window: JSRef<Window>) -> Temporary<DOMParser> {
        reflect_dom_object(box DOMParser::new_inherited(window), global::Window(window),
                           DOMParserBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalRef) -> Fallible<Temporary<DOMParser>> {
        Ok(DOMParser::new(global.as_window()))
    }
}

impl<'a> DOMParserMethods for JSRef<'a, DOMParser> {
    // http://domparsing.spec.whatwg.org/#the-domparser-interface
    fn ParseFromString(self,
                       s: DOMString,
                       ty: DOMParserBinding::SupportedType)
                       -> Fallible<Temporary<Document>> {
        let window = self.window.root().clone();
        let url = Some(window.get_url());
        let content_type = DOMParserBinding::SupportedTypeValues::strings[ty as uint].to_string();
        match ty {
            Text_html => {
                let document = Document::new(window, url.clone(),
                                             IsHTMLDocument::HTMLDocument,
                                             Some(content_type),
                                             DocumentSource::FromParser).root().clone();
                let parser = ServoHTMLParser::new(url.clone(), document).root().clone();
                parser.parse_chunk(s);
                parser.finish();
                document.set_ready_state(DocumentReadyState::Complete);
                Ok(Temporary::from_rooted(document))
            }
            Text_xml => {
                //FIXME: this should probably be FromParser when we actually parse the string (#3756).
                Ok(Document::new(window, url.clone(),
                                 IsHTMLDocument::NonHTMLDocument,
                                 Some(content_type),
                                 DocumentSource::NotFromParser))
            }
            _ => {
                Err(FailureUnknown)
            }
        }
    }
}

impl Reflectable for DOMParser {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
