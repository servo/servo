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
use crate::dom::bindings::codegen::UnionTypes::TrustedHTMLOrString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::servoparser::ServoParser;
use crate::dom::trustedhtml::TrustedHTML;
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
        s: TrustedHTMLOrString,
        ty: DOMParserBinding::SupportedType,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Document>> {
        // Step 1. Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, string, "DOMParser parseFromString", and "script".
        let compliant_string = TrustedHTML::get_trusted_script_compliant_string(
            self.window.as_global_scope(),
            s,
            "DOMParser parseFromString",
            can_gc,
        )?;
        let url = self.window.get_url();
        let content_type = ty
            .as_str()
            .parse()
            .expect("Supported type is not a MIME type");
        let doc = self.window.Document();
        let loader = DocumentLoader::new(&doc.loader());
        // Step 3. Switch on type:
        let document = match ty {
            Text_html => {
                // Step 2. Let document be a new Document, whose content type is type
                // and URL is this's relevant global object's associated Document's URL.
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
                    false,
                    Some(doc.insecure_requests_policy()),
                    doc.has_trustworthy_ancestor_or_current_origin(),
                    doc.custom_element_reaction_stack(),
                    doc.creation_sandboxing_flag_set(),
                    can_gc,
                );
                // Step switch-1. Parse HTML from a string given document and compliantString.
                ServoParser::parse_html_document(
                    &document,
                    Some(compliant_string),
                    url,
                    None,
                    None,
                    can_gc,
                );
                document
            },
            Text_xml | Application_xml | Application_xhtml_xml | Image_svg_xml => {
                // Step 2. Let document be a new Document, whose content type is type
                // and URL is this's relevant global object's associated Document's URL.
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
                    false,
                    Some(doc.insecure_requests_policy()),
                    doc.has_trustworthy_ancestor_or_current_origin(),
                    doc.custom_element_reaction_stack(),
                    doc.creation_sandboxing_flag_set(),
                    can_gc,
                );
                // Step switch-1. Create an XML parser parser, associated with document,
                // and with XML scripting support disabled.
                ServoParser::parse_xml_document(
                    &document,
                    Some(compliant_string),
                    url,
                    None,
                    can_gc,
                );
                document
            },
        };
        // Step 4. Return document.
        document.set_ready_state(DocumentReadyState::Complete, can_gc);
        Ok(document)
    }
}
