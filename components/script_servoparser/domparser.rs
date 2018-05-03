/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::document_loader::DocumentLoader;
use script::dom::bindings::codegen::Bindings::DOMParserBinding;
use script::dom::bindings::codegen::Bindings::DOMParserBinding::DOMParserMethods;
use script::dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::Application_xhtml_xml;
use script::dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::Application_xml;
use script::dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::Text_html;
use script::dom::bindings::codegen::Bindings::DOMParserBinding::SupportedType::Text_xml;
use script::dom::bindings::codegen::Bindings::DocumentBinding::DocumentReadyState;
use script::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use script::dom::bindings::error::Fallible;
use script::dom::bindings::reflector::{Reflector, reflect_dom_object};
use script::dom::bindings::root::{Dom, DomRoot};
use script::dom::bindings::str::DOMString;
use script::dom::document::{Document, HasBrowsingContext, IsHTMLDocument};
use script::dom::document::DocumentSource;
use script::dom::servoparser::ServoParser;
use script::dom::window::Window;
use script_traits::DocumentActivity;
use script::typeholder::TypeHolderTrait;
use script;
use script::dom::domparser::DOMParserTrait;
use script::dom::bindings::utils::DOMClass;
use script::dom::bindings::conversions::IDLInterface;
use TypeHolder;

#[derive(DomObject, DenyPublicFields, JSTraceable, MallocSizeOf)]
        #[base = "script"]
        #[must_root]
        #[repr(C)]
pub struct DOMParser {
    reflector_: Reflector<TypeHolder>,
    window: Dom<Window<TypeHolder>>, // XXXjdm Document instead?
}

impl DOMParserTrait<TypeHolder> for DOMParser  {
    fn Constructor(window: &Window<TypeHolder>) -> Fallible<DomRoot<DOMParser>> {
        Ok(DOMParser::new(window))
    }
}

impl IDLInterface for DOMParser {
    fn derives(_: &'static DOMClass) -> bool {
        false
    }
}


impl DOMParser {
    fn new_inherited(window: &Window<TypeHolder>) -> DOMParser {
        DOMParser {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub fn new(window: &Window<TypeHolder>) -> DomRoot<DOMParser> {
        reflect_dom_object(Box::new(DOMParser::new_inherited(window)),
                           window,
                           DOMParserBinding::Wrap)
    }
}

impl DOMParserMethods<TypeHolder> for DOMParser {
    // https://w3c.github.io/DOM-Parsing/#the-domparser-interface
    fn ParseFromString(&self,
                       s: DOMString,
                       ty: DOMParserBinding::SupportedType)
                       -> Fallible<DomRoot<Document<TypeHolder>>> {
        let url = self.window.get_url();
        let content_type = ty.as_str().parse().expect("Supported type is not a MIME type");
        let doc = self.window.Document();
        let loader = DocumentLoader::new(&*doc.loader());
        match ty {
            Text_html => {
                let document = Document::new(&self.window,
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
                                             Default::default());
                <TypeHolder as TypeHolderTrait>::ServoParser::parse_html_document(&document, s, url);
                document.set_ready_state(DocumentReadyState::Complete);
                Ok(document)
            }
            Text_xml | Application_xml | Application_xhtml_xml => {
                let document = Document::new(&self.window,
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
                                             Default::default());
                <TypeHolder as TypeHolderTrait>::ServoParser::parse_xml_document(&document, s, url);
                document.set_ready_state(DocumentReadyState::Complete);
                Ok(document)
            }
        }
    }
}
