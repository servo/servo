/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLProgressElementBinding::HTMLProgressElementMethods;
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
pub struct HTMLProgressElement {
    htmlelement: HTMLElement,
    labels_node_list: MutNullableDom<NodeList>,
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
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLProgressElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLProgressElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HTMLProgressElementMethods for HTMLProgressElement {
    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    // https://html.spec.whatwg.org/multipage/#dom-progress-value
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
    fn SetValue(&self, new_val: Finite<f64>) {
        if *new_val >= 0.0 {
            let mut string_value = DOMString::from_string((*new_val).to_string());

            string_value.set_best_representation_of_the_floating_point_number();

            self.upcast::<Element>()
                .set_string_attribute(&local_name!("value"), string_value);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-progress-max
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
    fn SetMax(&self, new_val: Finite<f64>) {
        if *new_val > 0.0 {
            let mut string_value = DOMString::from_string((*new_val).to_string());

            string_value.set_best_representation_of_the_floating_point_number();

            self.upcast::<Element>()
                .set_string_attribute(&local_name!("max"), string_value);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-progress-position
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
