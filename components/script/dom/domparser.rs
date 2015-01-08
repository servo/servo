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
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::{Document, DocumentHelpers, IsHTMLDocument};
use dom::document::DocumentSource;
use dom::window::Window;
use parse::html::{HTMLInput, parse_html};
use servo_util::str::DOMString;

#[dom_struct]
pub struct DOMParser {
    reflector_: Reflector,
    window: JS<Window>, //XXXjdm Document instead?
}

impl DOMParser {
    fn new_inherited(window: JSRef<Window>) -> DOMParser {
        DOMParser {
            reflector_: Reflector::new(),
            window: JS::from_rooted(window),
        }
    }

    pub fn new(window: JSRef<Window>) -> Temporary<DOMParser> {
        reflect_dom_object(box DOMParser::new_inherited(window), GlobalRef::Window(window),
                           DOMParserBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Temporary<DOMParser>> {
        Ok(DOMParser::new(global.as_window()))
    }
}

impl<'a> DOMParserMethods for JSRef<'a, DOMParser> {
    // http://domparsing.spec.whatwg.org/#the-domparser-interface
    fn ParseFromString(self,
                       s: DOMString,
                       ty: DOMParserBinding::SupportedType)
                       -> Fallible<Temporary<Document>> {
        let window = self.window.root();
        let url = window.r().get_url();
        let content_type = DOMParserBinding::SupportedTypeValues::strings[ty as uint].into_string();
        match ty {
            Text_html => {
                let document = Document::new(window.r(), Some(url.clone()),
                                             IsHTMLDocument::HTMLDocument,
                                             Some(content_type),
                                             DocumentSource::FromParser).root();
                parse_html(document.r(), HTMLInput::InputString(s), &url);
                document.r().set_ready_state(DocumentReadyState::Complete);
                Ok(Temporary::from_rooted(document.r()))
            }
            Text_xml => {
                //FIXME: this should probably be FromParser when we actually parse the string (#3756).
                Ok(Document::new(window.r(), Some(url.clone()),
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

