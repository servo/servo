// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLLegendElementBinding::HTMLLegendElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::html::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::node::Node;
use crate::dom::node::virtualmethods::VirtualMethods;

#[dom_struct]
pub(crate) struct HTMLLegendElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
}

impl HTMLLegendElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLLegendElement {
        HTMLLegendElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLLegendElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLLegendElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl VirtualMethods for HTMLLegendElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }
}

impl HTMLLegendElementMethods<crate::DomTypeHolder> for HTMLLegendElement {
    /// <https://html.spec.whatwg.org/multipage/#dom-legend-form>
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        let parent = self.upcast::<Node>().GetParentElement()?;
        if parent.is::<HTMLFieldSetElement>() {
            return self.form_owner();
        }
        None
    }
}

impl FormControl for HTMLLegendElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, _cx: &mut JSContext, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_html_element(&self) -> &HTMLElement {
        self.upcast::<HTMLElement>()
    }
}
