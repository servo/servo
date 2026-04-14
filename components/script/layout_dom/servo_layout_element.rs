/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]
#![deny(missing_docs)]

use std::fmt;
use std::hash::Hash;

use html5ever::{LocalName, Namespace, local_name, ns};
use layout_api::{
    LayoutDataTrait, LayoutElement, LayoutNode, LayoutNodeType, PseudoElementChain, StyleData,
};
use servo_arc::Arc;
use style::attr::AttrValue;
use style::context::SharedStyleContext;
use style::data::{ElementDataMut, ElementDataRef};
use style::properties::ComputedValues;
use style::selector_parser::{PseudoElement, PseudoElementCascadeType};
use style::stylist::RuleInclusion;

use crate::dom::bindings::root::LayoutDom;
use crate::dom::element::Element;
use crate::dom::node::{Node, NodeFlags};
use crate::layout_dom::{
    ServoDangerousStyleElement, ServoDangerousStyleShadowRoot, ServoLayoutNode,
};

impl fmt::Debug for LayoutDom<'_, Element> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.local_name())?;
        if let Some(id) = unsafe { (*self.id_attribute()).as_ref() } {
            write!(f, " id={id}")?;
        }
        write!(f, "> ({:#x})", self.upcast::<Node>().opaque().0)
    }
}

/// An implementation of [`LayoutElement`] for Servo's `script` crate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ServoLayoutElement<'dom> {
    /// The wrapped private DOM Element.
    pub(super) element: LayoutDom<'dom, Element>,
    /// The possibly nested [`PseudoElementChain`] for this element.
    pub(super) pseudo_element_chain: PseudoElementChain,
}

unsafe impl Send for ServoLayoutElement<'_> {}
unsafe impl Sync for ServoLayoutElement<'_> {}

impl<'dom> ServoLayoutElement<'dom> {
    pub(super) fn is_html_element(&self) -> bool {
        self.element.is_html_element()
    }

    /// The shadow root that this [`ServoLayoutElement`] is a host of, if it has one.
    ///
    /// Note: This should *not* be exposed to layout as it allows access to an ancestor element.
    pub(super) fn shadow_root(&self) -> Option<ServoDangerousStyleShadowRoot<'dom>> {
        self.element.get_shadow_root_for_layout().map(Into::into)
    }
}

impl<'dom> From<LayoutDom<'dom, Element>> for ServoLayoutElement<'dom> {
    fn from(element: LayoutDom<'dom, Element>) -> Self {
        Self {
            element,
            pseudo_element_chain: Default::default(),
        }
    }
}

impl<'dom> LayoutElement<'dom> for ServoLayoutElement<'dom> {
    type ConcreteLayoutNode = ServoLayoutNode<'dom>;
    type ConcreteStyleElement = ServoDangerousStyleElement<'dom>;

    fn with_pseudo(&self, pseudo_element: PseudoElement) -> Option<Self> {
        if pseudo_element.is_eager() &&
            self.element_data()
                .styles
                .pseudos
                .get(&pseudo_element)
                .is_none()
        {
            return None;
        }

        if pseudo_element == PseudoElement::DetailsContent &&
            (self.element.local_name() != &local_name!("details") ||
                self.element.namespace() != &ns!(html) ||
                self.attribute(&ns!(), &local_name!("open")).is_none())
        {
            return None;
        }

        // These pseudo-element type cannot be nested.
        if !self.pseudo_element_chain.is_empty() {
            assert!(!pseudo_element.is_eager());
            assert!(pseudo_element != PseudoElement::DetailsContent);
        }

        Some(Self {
            element: self.element,
            pseudo_element_chain: self.pseudo_element_chain.with_pseudo(pseudo_element),
        })
    }

    fn pseudo_element_chain(&self) -> PseudoElementChain {
        self.pseudo_element_chain
    }

    fn as_node(&self) -> ServoLayoutNode<'dom> {
        ServoLayoutNode {
            node: self.element.upcast(),
            pseudo_element_chain: self.pseudo_element_chain,
        }
    }

    fn initialize_style_and_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self) {
        if self.element.style_data().is_none() {
            unsafe { self.element.initialize_style_data() };
        }

        let node = self.element.upcast::<Node>();
        if node.layout_data().is_none() {
            unsafe { node.initialize_layout_data(Box::<RequestedLayoutDataType>::default()) };
        }
    }

    fn unset_snapshot_flags(&self) {
        unsafe {
            self.as_node()
                .node
                .set_flag(NodeFlags::HAS_SNAPSHOT | NodeFlags::HANDLED_SNAPSHOT, false);
        }
    }

    fn set_has_snapshot(&self) {
        unsafe {
            self.as_node().node.set_flag(NodeFlags::HAS_SNAPSHOT, true);
        }
    }

    fn style_data(self) -> Option<&'dom StyleData> {
        self.element.style_data()
    }

    fn element_data(&self) -> ElementDataRef<'dom> {
        self.style_data()
            .expect("Unstyled layout node?")
            .element_data
            .borrow()
    }

    fn element_data_mut(&self) -> ElementDataMut<'dom> {
        self.style_data()
            .expect("Unstyled layout node?")
            .element_data
            .borrow_mut()
    }

    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues> {
        let get_style_for_pseudo_element =
            |data: &ElementDataRef<'_>,
             base_style: &Arc<ComputedValues>,
             pseudo_element: PseudoElement| {
                // Precompute non-eagerly-cascaded pseudo-element styles if not
                // cached before.
                match pseudo_element.cascade_type() {
                    // Already computed during the cascade.
                    PseudoElementCascadeType::Eager => {
                        data.styles.pseudos.get(&pseudo_element).unwrap().clone()
                    },
                    PseudoElementCascadeType::Precomputed => context
                        .stylist
                        .precomputed_values_for_pseudo::<Self::ConcreteStyleElement>(
                        &context.guards,
                        &pseudo_element,
                        Some(base_style),
                    ),
                    PseudoElementCascadeType::Lazy => {
                        context
                            .stylist
                            .lazily_compute_pseudo_element_style(
                                &context.guards,
                                unsafe { self.dangerous_style_element() },
                                &pseudo_element,
                                RuleInclusion::All,
                                base_style,
                                /* is_probe = */ false,
                                /* matching_func = */ None,
                            )
                            .unwrap()
                    },
                }
            };

        let data = self.element_data();
        let element_style = data.styles.primary();
        let pseudo_element_chain = self.pseudo_element_chain();

        let primary_pseudo_style = match pseudo_element_chain.primary {
            Some(pseudo_element) => {
                get_style_for_pseudo_element(&data, element_style, pseudo_element)
            },
            None => return element_style.clone(),
        };
        match pseudo_element_chain.secondary {
            Some(pseudo_element) => {
                get_style_for_pseudo_element(&data, &primary_pseudo_style, pseudo_element)
            },
            None => primary_pseudo_style,
        }
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        self.as_node().type_id()
    }

    unsafe fn dangerous_style_element(self) -> ServoDangerousStyleElement<'dom> {
        self.element.into()
    }

    fn local_name(&self) -> &LocalName {
        self.element.local_name()
    }

    fn attribute(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue> {
        self.element.get_attr_for_layout(namespace, name)
    }

    fn attribute_as_str<'a>(&'a self, namespace: &Namespace, name: &LocalName) -> Option<&'a str> {
        self.element.get_attr_val_for_layout(namespace, name)
    }

    fn is_shadow_host(&self) -> bool {
        self.element.get_shadow_root_for_layout().is_some()
    }

    fn is_body_element_of_html_element_root(&self) -> bool {
        self.element.is_body_element_of_html_element_root()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.element.is_html_element() &&
            self.element
                .upcast::<Node>()
                .owner_doc_for_layout()
                .is_html_document_for_layout()
    }

    fn is_root(&self) -> bool {
        self.element.is_root()
    }
}
