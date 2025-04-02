/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;
use std::ops::{Add, Div};

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use stylo_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLMeterElementBinding::HTMLMeterElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmldivelement::HTMLDivElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, ChildrenMutation, Node, NodeTraits};
use crate::dom::nodelist::NodeList;
use crate::dom::shadowroot::IsUserAgentWidget;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLMeterElement {
    htmlelement: HTMLElement,
    labels_node_list: MutNullableDom<NodeList>,
    shadow_tree: DomRefCell<Option<ShadowTree>>,
}

/// Holds handles to all slots in the UA shadow tree
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct ShadowTree {
    meter_value: Dom<HTMLDivElement>,
}

/// <https://html.spec.whatwg.org/multipage/#the-meter-element>
impl HTMLMeterElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLMeterElement {
        HTMLMeterElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            labels_node_list: MutNullableDom::new(None),
            shadow_tree: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLMeterElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLMeterElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    fn create_shadow_tree(&self, can_gc: CanGc) {
        let document = self.owner_document();
        let root = self
            .upcast::<Element>()
            .attach_shadow(
                IsUserAgentWidget::Yes,
                ShadowRootMode::Closed,
                false,
                false,
                false,
                SlotAssignmentMode::Manual,
                can_gc,
            )
            .expect("Attaching UA shadow root failed");

        let meter_value = HTMLDivElement::new(local_name!("div"), None, &document, None, can_gc);
        root.upcast::<Node>()
            .AppendChild(meter_value.upcast::<Node>())
            .unwrap();

        let _ = self.shadow_tree.borrow_mut().insert(ShadowTree {
            meter_value: meter_value.as_traced(),
        });
        self.upcast::<Node>()
            .dirty(crate::dom::node::NodeDamage::OtherNodeDamage);
    }

    fn shadow_tree(&self, can_gc: CanGc) -> Ref<'_, ShadowTree> {
        if !self.upcast::<Element>().is_shadow_host() {
            self.create_shadow_tree(can_gc);
        }

        Ref::filter_map(self.shadow_tree.borrow(), Option::as_ref)
            .ok()
            .expect("UA shadow tree was not created")
    }

    fn update_state(&self, can_gc: CanGc) {
        let value = *self.Value();
        let low = *self.Low();
        let high = *self.High();
        let min = *self.Min();
        let max = *self.Max();
        let optimum = *self.Optimum();

        // If the optimum point is less than the low boundary, then the region between the minimum value and
        // the low boundary must be treated as the optimum region, the region from the low boundary up to the
        // high boundary must be treated as a suboptimal region, and the remaining region must be treated as
        // an even less good region
        let element_state = if optimum < low {
            if value < low {
                ElementState::OPTIMUM
            } else if value <= high {
                ElementState::SUB_OPTIMUM
            } else {
                ElementState::SUB_SUB_OPTIMUM
            }
        }
        // If the optimum point is higher than the high boundary, then the situation is reversed; the region between
        // the high boundary and the maximum value must be treated as the optimum region, the region from the high
        // boundary down to the low boundary must be treated as a suboptimal region, and the remaining region must
        // be treated as an even less good region.
        else if optimum > high {
            if value > high {
                ElementState::OPTIMUM
            } else if value >= low {
                ElementState::SUB_OPTIMUM
            } else {
                ElementState::SUB_SUB_OPTIMUM
            }
        }
        // If the optimum point is equal to the low boundary or the high boundary, or anywhere in between them,
        // then the region between the low and high boundaries of the gauge must be treated as the optimum region,
        // and the low and high parts, if any, must be treated as suboptimal.
        else if (low..=high).contains(&value) {
            ElementState::OPTIMUM
        } else {
            ElementState::SUB_OPTIMUM
        };

        // Set the correct pseudo class
        self.upcast::<Element>()
            .set_state(ElementState::METER_OPTIMUM_STATES, false);
        self.upcast::<Element>().set_state(element_state, true);

        // Update the visual width of the meter
        let shadow_tree = self.shadow_tree(can_gc);
        let position = (value - min) / (max - min) * 100.0;
        let style = format!("width: {position}%");
        shadow_tree
            .meter_value
            .upcast::<Element>()
            .set_string_attribute(&local_name!("style"), style.into(), can_gc);
    }
}

impl HTMLMeterElementMethods<crate::DomTypeHolder> for HTMLMeterElement {
    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-actual>
    fn Value(&self) -> Finite<f64> {
        let min = *self.Min();
        let max = *self.Max();

        Finite::wrap(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("value"))
                .parse_floating_point_number()
                .map_or(0.0, |candidate_actual_value| {
                    candidate_actual_value.clamp(min, max)
                }),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-meter-value>
    fn SetValue(&self, value: Finite<f64>, can_gc: CanGc) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("value"), string_value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-minimum>
    fn Min(&self) -> Finite<f64> {
        Finite::wrap(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("min"))
                .parse_floating_point_number()
                .unwrap_or(0.0),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-meter-min>
    fn SetMin(&self, value: Finite<f64>, can_gc: CanGc) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("min"), string_value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-maximum>
    fn Max(&self) -> Finite<f64> {
        Finite::wrap(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("max"))
                .parse_floating_point_number()
                .unwrap_or(1.0)
                .max(*self.Min()),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-maximum>
    fn SetMax(&self, value: Finite<f64>, can_gc: CanGc) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("max"), string_value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-low>
    fn Low(&self) -> Finite<f64> {
        let min = *self.Min();
        let max = *self.Max();

        Finite::wrap(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("low"))
                .parse_floating_point_number()
                .map_or(min, |candidate_low_boundary| {
                    candidate_low_boundary.clamp(min, max)
                }),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-meter-low>
    fn SetLow(&self, value: Finite<f64>, can_gc: CanGc) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("low"), string_value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-high>
    fn High(&self) -> Finite<f64> {
        let max: f64 = *self.Max();
        let low: f64 = *self.Low();

        Finite::wrap(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("high"))
                .parse_floating_point_number()
                .map_or(max, |candidate_high_boundary| {
                    if candidate_high_boundary < low {
                        return low;
                    }

                    candidate_high_boundary.clamp(*self.Min(), max)
                }),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-meter-high>
    fn SetHigh(&self, value: Finite<f64>, can_gc: CanGc) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("high"), string_value, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-meter-optimum>
    fn Optimum(&self) -> Finite<f64> {
        let max = *self.Max();
        let min = *self.Min();

        Finite::wrap(
            self.upcast::<Element>()
                .get_string_attribute(&local_name!("optimum"))
                .parse_floating_point_number()
                .map_or(max.add(min).div(2.0), |candidate_optimum_point| {
                    candidate_optimum_point.clamp(min, max)
                }),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-meter-optimum>
    fn SetOptimum(&self, value: Finite<f64>, can_gc: CanGc) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>().set_string_attribute(
            &local_name!("optimum"),
            string_value,
            can_gc,
        );
    }
}

impl VirtualMethods for HTMLMeterElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        let is_important_attribute = matches!(
            attr.local_name(),
            &local_name!("high") |
                &local_name!("low") |
                &local_name!("min") |
                &local_name!("max") |
                &local_name!("optimum") |
                &local_name!("value")
        );
        if is_important_attribute {
            self.update_state(CanGc::note());
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        self.super_type().unwrap().children_changed(mutation);

        self.update_state(CanGc::note());
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);

        self.update_state(CanGc::note());
    }
}
