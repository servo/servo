/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;

use html5ever::{LocalName, QualName, ns};
use js::context::JSContext;

use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::element::element::Element;
use crate::dom::element::{CustomElementCreationMode, ElementCreator};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::shadowroot::shadowroot::{IsUserAgentWidget, ShadowRoot};

pub(crate) trait UAShadowRoot<ShadowTree>:
    NodeTraits + Castable + DerivedFrom<Element> + DerivedFrom<Node>
{
    fn create_element_in_ua_shadowroot(
        &self,
        cx: &mut JSContext,
        local_name: LocalName,
    ) -> DomRoot<Element> {
        let document = self.owner_document();
        Element::create(
            cx,
            QualName::new(None, ns!(html), local_name),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        )
    }

    fn shadow_tree(&self, cx: &mut JSContext) -> Ref<'_, ShadowTree> {
        if !self.upcast::<Element>().is_shadow_host() {
            self.create_shadow_tree(cx);
        }

        Ref::filter_map(self.borrow_for_shadow_tree(), Option::as_ref)
            .ok()
            .expect("UA shadow tree was not created")
    }

    fn store_for_shadow_tree(&self, cx: &mut JSContext, shadow_root: DomRoot<ShadowRoot>);

    fn borrow_for_shadow_tree(&self) -> Ref<'_, Option<ShadowTree>>;
}

trait UAShadowRootHelpers<ShadowTree>: UAShadowRoot<ShadowTree> {
    fn attach_ua_shadow_root(cx: &mut JSContext, element: &Element) -> DomRoot<ShadowRoot>;
    fn create_shadow_tree(&self, cx: &mut JSContext);
}

impl<T: UAShadowRoot<ShadowTree>, ShadowTree> UAShadowRootHelpers<ShadowTree> for T {
    /// Attach a UA widget shadow root with its default parameters.
    /// Additionally mark ShadowRoot to use styling configuration for a UA widget.
    ///
    /// The general trait of these elements is that it would hide the implementation.
    /// Thus, we would make it inaccessible (i.e., closed mode, not cloneable, and
    /// not serializable).
    ///
    /// With UA shadow root element being assumed as one element, any focus should
    /// be delegated to its host.
    ///
    // TODO: Ideally, all of the UA shadow root should use UA widget styling, but
    //       some of the UA widget implemented prior to the implementation of Gecko's
    //       UA widget matching might need some tweaking.
    // FIXME: We are yet to implement more complex focusing with that is necessary
    //        for delegate focus, and we are using workarounds for that right now.
    fn attach_ua_shadow_root(cx: &mut JSContext, element: &Element) -> DomRoot<ShadowRoot> {
        let root = element
            .attach_shadow(
                cx,
                IsUserAgentWidget::Yes,
                ShadowRootMode::Closed,
                false,
                false,
                false,
                SlotAssignmentMode::Manual,
            )
            .expect("Attaching UA shadow root failed");

        root.upcast::<Node>().set_in_ua_widget(true);
        root
    }

    fn create_shadow_tree(&self, cx: &mut JSContext) {
        let root = Self::attach_ua_shadow_root(cx, self.upcast());

        self.store_for_shadow_tree(cx, root);

        self.upcast::<Node>()
            .dirty(cx.no_gc(), crate::dom::node::NodeDamage::Other);
    }
}
