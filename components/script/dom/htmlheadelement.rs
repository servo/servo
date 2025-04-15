/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use content_security_policy::{CspList, PolicyDisposition, PolicySource};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlmetaelement::HTMLMetaElement;
use crate::dom::node::{BindContext, Node, NodeTraits, ShadowIncluding};
use crate::dom::userscripts::load_script;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLHeadElement {
    htmlelement: HTMLElement,
}

impl HTMLHeadElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLHeadElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLHeadElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-meta-http-equiv-content-security-policy>
    pub(crate) fn set_content_security_policy(&self) {
        let doc = self.owner_document();

        if doc.GetHead().as_deref() != Some(self) {
            return;
        }

        let mut csp_list: Option<CspList> = None;
        let node = self.upcast::<Node>();
        let candidates = node
            .traverse_preorder(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Element>)
            .filter(|elem| elem.is::<HTMLMetaElement>())
            .filter(|elem| {
                elem.get_string_attribute(&local_name!("http-equiv"))
                    .to_ascii_lowercase() ==
                    *"content-security-policy"
            })
            .filter(|elem| {
                elem.get_attribute(&ns!(), &local_name!("content"))
                    .is_some()
            });

        for meta in candidates {
            if let Some(ref content) = meta.get_attribute(&ns!(), &local_name!("content")) {
                let content = content.value();
                let content_val = content.trim();
                if !content_val.is_empty() {
                    let policies =
                        CspList::parse(content_val, PolicySource::Meta, PolicyDisposition::Enforce);
                    match csp_list {
                        Some(ref mut csp_list) => csp_list.append(policies),
                        None => csp_list = Some(policies),
                    }
                }
            }
        }

        doc.set_csp_list(csp_list);
    }
}

impl VirtualMethods for HTMLHeadElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }
    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }
        load_script(self);
    }
}
