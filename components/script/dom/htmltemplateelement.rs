/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, namespace_url};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{CloneChildrenFlag, Node, NodeTraits};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLTemplateElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#template-contents>
    contents: MutNullableDom<DocumentFragment>,
}

impl HTMLTemplateElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            contents: MutNullableDom::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLTemplateElement> {
        let n = Node::reflect_node_with_proto(
            Box::new(HTMLTemplateElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        );

        n.upcast::<Node>().set_weird_parser_insertion_mode();
        n
    }

    pub(crate) fn set_contents(&self, document_fragment: Option<&DocumentFragment>) {
        self.contents.set(document_fragment);
    }
}

#[allow(unused_doc_comments)]
impl HTMLTemplateElementMethods<crate::DomTypeHolder> for HTMLTemplateElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootmode>
    make_enumerated_getter!(
        ShadowRootMode,
        "shadowrootmode",
        "open" | "closed",
        missing => "",
        invalid => ""
    );

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootmode>
    make_atomic_setter!(SetShadowRootMode, "shadowrootmode");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootdelegatesfocus>
    make_bool_getter!(ShadowRootDelegatesFocus, "shadowrootdelegatesfocus");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootdelegatesfocus>
    make_bool_setter!(SetShadowRootDelegatesFocus, "shadowrootdelegatesfocus");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootclonable>
    make_bool_getter!(ShadowRootClonable, "shadowrootclonable");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootclonable>
    make_bool_setter!(SetShadowRootClonable, "shadowrootclonable");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootserializable>
    make_bool_getter!(ShadowRootSerializable, "shadowrootserializable");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-shadowrootserializable>
    make_bool_setter!(SetShadowRootSerializable, "shadowrootserializable");

    /// <https://html.spec.whatwg.org/multipage/#dom-template-content>
    fn Content(&self, can_gc: CanGc) -> DomRoot<DocumentFragment> {
        self.contents.or_init(|| {
            let doc = self.owner_document();
            doc.appropriate_template_contents_owner_document(can_gc)
                .CreateDocumentFragment(can_gc)
        })
    }
}

impl VirtualMethods for HTMLTemplateElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    /// <https://html.spec.whatwg.org/multipage/#template-adopting-steps>
    fn adopting_steps(&self, old_doc: &Document, can_gc: CanGc) {
        self.super_type().unwrap().adopting_steps(old_doc, can_gc);
        // Step 1.
        let doc = self
            .owner_document()
            .appropriate_template_contents_owner_document(CanGc::note());
        // Step 2.
        Node::adopt(self.Content(CanGc::note()).upcast(), &doc, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-template-element:concept-node-clone-ext>
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
        can_gc: CanGc,
    ) {
        self.super_type()
            .unwrap()
            .cloning_steps(copy, maybe_doc, clone_children, can_gc);
        if clone_children == CloneChildrenFlag::DoNotCloneChildren {
            // Step 1.
            return;
        }
        let copy = copy.downcast::<HTMLTemplateElement>().unwrap();
        // Steps 2-3.
        let copy_contents = DomRoot::upcast::<Node>(copy.Content(CanGc::note()));
        let copy_contents_doc = copy_contents.owner_doc();
        for child in self.Content(CanGc::note()).upcast::<Node>().children() {
            let copy_child = Node::clone(
                &child,
                Some(&copy_contents_doc),
                CloneChildrenFlag::CloneChildren,
                CanGc::note(),
            );
            copy_contents.AppendChild(&copy_child, can_gc).unwrap();
        }
    }
}
