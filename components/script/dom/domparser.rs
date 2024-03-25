/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_traits::DocumentActivity;

use crate::document_loader::DocumentLoader;
use crate::dom::bindings::codegen::Bindings::DOMParserBinding;
use crate::dom::bindings::codegen::Bindings::DOMParserBinding::DOMParserMethods;
use crate::dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::{
    Application_xhtml_xml, Application_xml, Image_svg_xml, Text_html, Text_xml,
};
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentReadyState;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::servoparser::ServoParser;
use crate::dom::window::Window;

#[dom_struct]
pub struct DOMParser {
    reflector_: Reflector,
    window: Dom<Window>, // XXXjdm Document instead?
}

impl DOMParser {
    fn new_inherited(window: &Window) -> DOMParser {
        DOMParser {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    fn new(window: &Window, proto: Option<HandleObject>) -> DomRoot<DOMParser> {
        reflect_dom_object_with_proto(Box::new(DOMParser::new_inherited(window)), window, proto)
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<DOMParser>> {
        Ok(DOMParser::new(window, proto))
    }
}

impl DOMParserMethods for DOMParser {
    // https://w3c.github.io/DOM-Parsing/#the-domparser-interface
    fn ParseFromString(
        &self,
        s: DOMString,
        ty: DOMParserBinding::SupportedType,
    ) -> Fallible<DomRoot<Document>> {
        let url = self.window.get_url();
        let content_type = ty
            .as_str()
            .parse()
            .expect("Supported type is not a MIME type");
        let doc = self.window.Document();
        let loader = DocumentLoader::new(&doc.loader());
        match ty {
            Text_html => {
                let document = Document::new(
                    &self.window,
                    HasBrowsingContext::No,
                    Some(url.clone()),
                    doc.origin().clone(),
                    IsHTMLDocument::HTMLDocument,
                    Some(content_type),
                    None,
                    DocumentActivity::Inactive,
                    DocumentSource::FromParser,
                    loader,
                    None,
                    None,
                    Default::default(),
                );
                ServoParser::parse_html_document(&document, Some(s), url);
                document.set_ready_state(DocumentReadyState::Complete);
                Ok(document)
            },
            Text_xml | Application_xml | Application_xhtml_xml | Image_svg_xml => {
                let document = Document::new(
                    &self.window,
                    HasBrowsingContext::No,
                    Some(url.clone()),
                    doc.origin().clone(),
                    IsHTMLDocument::NonHTMLDocument,
                    Some(content_type),
                    None,
                    DocumentActivity::Inactive,
                    DocumentSource::FromParser,
                    loader,
                    None,
                    None,
                    Default::default(),
                );
                ServoParser::parse_xml_document(&document, Some(s), url);
                document.set_ready_state(DocumentReadyState::Complete);
                Ok(document)
            },
        }
    }
}
