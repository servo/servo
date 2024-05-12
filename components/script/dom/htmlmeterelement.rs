/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::{Add, Div};

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLMeterElementBinding::HTMLMeterElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::nodelist::NodeList;

#[dom_struct]
pub struct HTMLMeterElement {
    htmlelement: HTMLElement,
    labels_node_list: MutNullableDom<NodeList>,
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
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLMeterElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLMeterElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HTMLMeterElementMethods for HTMLMeterElement {
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
    fn SetValue(&self, value: Finite<f64>) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("value"), string_value);
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
    fn SetMin(&self, value: Finite<f64>) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("min"), string_value);
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
    fn SetMax(&self, value: Finite<f64>) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("max"), string_value);
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
    fn SetLow(&self, value: Finite<f64>) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("low"), string_value);
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
    fn SetHigh(&self, value: Finite<f64>) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("high"), string_value);
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
    fn SetOptimum(&self, value: Finite<f64>) {
        let mut string_value = DOMString::from_string((*value).to_string());

        string_value.set_best_representation_of_the_floating_point_number();

        self.upcast::<Element>()
            .set_string_attribute(&local_name!("optimum"), string_value);
    }
}
