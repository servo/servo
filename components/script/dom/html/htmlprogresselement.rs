/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ElementBinding::Element_Binding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLProgressElementBinding::HTMLProgressElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, CustomElementCreationMode, Element, ElementCreator};
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, Node, NodeTraits};
use crate::dom::nodelist::NodeList;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLProgressElement {
    htmlelement: HTMLElement,
    labels_node_list: MutNullableDom<NodeList>,
    shadow_tree: DomRefCell<Option<ShadowTree>>,
}

/// Holds handles to all slots in the UA shadow tree
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ShadowTree {
    progress_bar: Dom<Element>,
}

impl HTMLProgressElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLProgressElement {
        HTMLProgressElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            labels_node_list: MutNullableDom::new(None),
            shadow_tree: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLProgressElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLProgressElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    fn create_shadow_tree(&self, cx: &mut JSContext) {
        let document = self.owner_document();
        let root = self.upcast::<Element>().attach_ua_shadow_root(cx, true);

        let progress_bar = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("div")),
            None,
            &document,
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        // FIXME: This should use ::-moz-progress-bar
        progress_bar.SetId("-servo-progress-bar".into(), CanGc::from_cx(cx));
        root.upcast::<Node>()
            .AppendChild(cx, progress_bar.upcast::<Node>())
            .unwrap();

        let _ = self.shadow_tree.borrow_mut().insert(ShadowTree {
            progress_bar: progress_bar.as_traced(),
        });
        self.upcast::<Node>()
            .dirty(crate::dom::node::NodeDamage::Other);
    }

    fn shadow_tree(&self, cx: &mut JSContext) -> Ref<'_, ShadowTree> {
        if !self.upcast::<Element>().is_shadow_host() {
            self.create_shadow_tree(cx);
        }

        Ref::filter_map(self.shadow_tree.borrow(), Option::as_ref)
            .ok()
            .expect("UA shadow tree was not created")
    }

    /// Update the visual width of bar
    fn update_state(&self, cx: &mut JSContext) {
        let shadow_tree = self.shadow_tree(cx);
        let position = (*self.Value() / *self.Max()) * 100.0;
        let style = format!("width: {}%", position);

        shadow_tree.progress_bar.set_string_attribute(
            &local_name!("style"),
            style.into(),
            CanGc::from_cx(cx),
        );
    }
}

impl HTMLProgressElementMethods<crate::DomTypeHolder> for HTMLProgressElement {
    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    /// <https://html.spec.whatwg.org/multipage/#dom-progress-value>
    fn Value(&self) -> Finite<f64> {
        // In case of missing `value`, parse error, or negative `value`, `value` should be
        // interpreted as 0.  As `get_string_attribute` returns an empty string in case the
        // attribute is missing, this case is handeled as the default of the `map_or` function.
        //
        // It is safe to wrap the number coming from `parse_floating_point_number` as it will
        // return Err on inf and nan
        self.upcast::<Element>()
            .get_string_attribute(&local_name!("value"))
            .parse_floating_point_number()
            .map_or(Finite::wrap(0.0), |v| {
                if v < 0.0 {
                    Finite::wrap(0.0)
                } else {
                    Finite::wrap(v.min(*self.Max()))
                }
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-progress-value>
    fn SetValue(&self, new_val: Finite<f64>, can_gc: CanGc) {
        if *new_val >= 0.0 {
            let mut string_value = DOMString::from((*new_val).to_string());
            string_value.set_best_representation_of_the_floating_point_number();
            self.upcast::<Element>().set_string_attribute(
                &local_name!("value"),
                string_value,
                can_gc,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-progress-max>
    fn Max(&self) -> Finite<f64> {
        // In case of missing `max`, parse error, or negative `max`, `max` should be interpreted as
        // 1.0. As `get_string_attribute` returns an empty string in case the attribute is missing,
        // these cases are handeled by `map_or`
        self.upcast::<Element>()
            .get_string_attribute(&local_name!("max"))
            .parse_floating_point_number()
            .map_or(Finite::wrap(1.0), |m| {
                if m <= 0.0 {
                    Finite::wrap(1.0)
                } else {
                    Finite::wrap(m)
                }
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-progress-max>
    fn SetMax(&self, new_val: Finite<f64>, can_gc: CanGc) {
        if *new_val > 0.0 {
            let mut string_value = DOMString::from((*new_val).to_string());
            string_value.set_best_representation_of_the_floating_point_number();
            self.upcast::<Element>().set_string_attribute(
                &local_name!("max"),
                string_value,
                can_gc,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-progress-position>
    fn Position(&self) -> Finite<f64> {
        let value = self
            .upcast::<Element>()
            .get_string_attribute(&local_name!("value"));
        if value.is_empty() {
            Finite::wrap(-1.0)
        } else {
            let value = self.Value();
            let max = self.Max();
            // An unsafe Finite constructor might be nice here, as it's unlikely for the
            // compiler to infer the following guarantees. It is probably premature
            // optimization though.
            //
            // Safety: `ret` have to be a finite, defined number. This is the case since both
            // value and max is finite, max > 0, and a value >> max cannot exist, as
            // Self::Value(&self) enforces value <= max.
            Finite::wrap(*value / *max)
        }
    }
}

impl VirtualMethods for HTMLProgressElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);

        let is_important_attribute = matches!(
            attr.local_name(),
            &local_name!("value") | &local_name!("max")
        );
        if is_important_attribute {
            self.update_state(cx);
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        self.super_type().unwrap().bind_to_tree(cx, context);

        self.update_state(cx);
    }
}
