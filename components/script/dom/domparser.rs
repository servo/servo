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
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct DOMParser {
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

    fn new(window: &Window, proto: Option<HandleObject>, can_gc: CanGc) -> DomRoot<DOMParser> {
        reflect_dom_object_with_proto(
            Box::new(DOMParser::new_inherited(window)),
            window,
            proto,
            can_gc,
        )
    }
}

impl DOMParserMethods<crate::DomTypeHolder> for DOMParser {
    /// <https://html.spec.whatwg.org/multipage/#dom-domparser-constructor>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DOMParser>> {
        Ok(DOMParser::new(window, proto, can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-domparser-parsefromstring>
    fn ParseFromString(
        &self,
        s: DOMString,
        ty: DOMParserBinding::SupportedType,
        can_gc: CanGc,
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
                    false,
                    Some(doc.insecure_requests_policy()),
                    can_gc,
                );
                ServoParser::parse_html_document(&document, Some(s), url, can_gc);
                document.set_ready_state(DocumentReadyState::Complete, can_gc);
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
                    false,
                    Some(doc.insecure_requests_policy()),
                    can_gc,
                );
                ServoParser::parse_xml_document(&document, Some(s), url, can_gc);
                document.set_ready_state(DocumentReadyState::Complete, can_gc);
                Ok(document)
            },
        }
    }
}
