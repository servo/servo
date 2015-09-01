/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DOMParserBinding;
use dom::bindings::codegen::Bindings::DOMParserBinding::DOMParserMethods;
use dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::{Text_html, Text_xml};
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentReadyState;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::DocumentSource;
use dom::document::{Document, IsHTMLDocument};
use dom::window::Window;
use parse::html::{ParseContext, parse_html};
use util::str::DOMString;

use std::borrow::ToOwned;

#[dom_struct]
pub struct DOMParser {
    reflector_: Reflector,
    window: JS<Window>, //XXXjdm Document instead?
}

impl DOMParser {
    fn new_inherited(window: &Window) -> DOMParser {
        DOMParser {
            reflector_: Reflector::new(),
            window: JS::from_ref(window),
        }
    }

    pub fn new(window: &Window) -> Root<DOMParser> {
        reflect_dom_object(box DOMParser::new_inherited(window), GlobalRef::Window(window),
                           DOMParserBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<DOMParser>> {
        Ok(DOMParser::new(global.as_window()))
    }
}

impl DOMParserMethods for DOMParser {
    // https://domparsing.spec.whatwg.org/#the-domparser-interface
    fn ParseFromString(&self,
                       s: DOMString,
                       ty: DOMParserBinding::SupportedType)
                       -> Fallible<Root<Document>> {
        let window = self.window.root();
        let url = window.r().get_url();
        let content_type = DOMParserBinding::SupportedTypeValues::strings[ty as usize].to_owned();
        let doc = window.r().Document();
        let doc = doc.r();
        let loader = DocumentLoader::new(&*doc.loader());
        match ty {
            Text_html => {
                let document = Document::new(window.r(), Some(url.clone()),
                                             IsHTMLDocument::HTMLDocument,
                                             Some(content_type),
                                             None,
                                             DocumentSource::FromParser,
                                             loader);
                parse_html(document.r(), s, &url, ParseContext::Owner(None));
                document.r().set_ready_state(DocumentReadyState::Complete);
                Ok(document)
            }
            Text_xml => {
                //FIXME: this should probably be FromParser when we actually parse the string (#3756).
                Ok(Document::new(window.r(), Some(url.clone()),
                                 IsHTMLDocument::NonHTMLDocument,
                                 Some(content_type),
                                 None,
                                 DocumentSource::NotFromParser,
                                 loader))
            }
        }
    }
}
