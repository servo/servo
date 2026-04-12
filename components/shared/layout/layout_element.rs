/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(unsafe_code)]
#![deny(missing_docs)]

use std::fmt::Debug;

use html5ever::{LocalName, Namespace};
use servo_arc::Arc;
use style::attr::AttrValue;
use style::context::SharedStyleContext;
use style::data::{ElementDataMut, ElementDataRef};
use style::dom::TElement;
use style::properties::ComputedValues;
use style::selector_parser::{PseudoElement, SelectorImpl};

use crate::{LayoutDataTrait, LayoutNode, LayoutNodeType, PseudoElementChain, StyleData};

/// A trait that exposes a DOM element to layout. Implementors of this trait must abide by certain
/// safety requirements. Layout will only ever access and mutate each element from a single thread
/// at a time, though children may be used in parallel from other threads. That is why this trait
/// does not allow access to parent nodes, as it would make it easy to cause race conditions and
/// memory errors.
///
/// Note that the related [`DangerousStyleElement`] trait *may* access parent nodes, which is why
/// that API is marked as `unsafe` here. In general [`DangerousStyleElement`] should only be used
/// when interfacing with the `stylo` and `selectors`.
pub trait LayoutElement<'dom>: Copy + Debug + Send + Sync {
    /// An associated type that refers to the concrete implementation of [`DangerousStyleElement`]
    /// implemented in `script`.
    type ConcreteStyleElement: DangerousStyleElement<'dom>;
    /// An associated type that refers to the concrete implementation of [`LayoutNode`]
    /// implemented in `script`.
    type ConcreteLayoutNode: LayoutNode<'dom>;

    /// Creates a new `LayoutElement` for the same `LayoutElement`
    /// with a different pseudo-element type.
    ///
    /// Returns `None` if this pseudo doesn't apply to the given element for one of
    /// the following reasons:
    ///
    ///  1. `pseudo` is eager and is not defined in the stylesheet. In this case, there
    ///     is not reason to process the pseudo element at all.
    ///  2. `pseudo` is for `::servo-details-content` and
    ///     it doesn't apply to this element, either because it isn't a details or is
    ///     in the wrong state.
    fn with_pseudo(&self, pseudo: PseudoElement) -> Option<Self>;

    /// Returns the [`PseudoElementChain`] for this [`LayoutElement`].
    fn pseudo_element_chain(&self) -> PseudoElementChain;

    /// Return this [`LayoutElement`] as a [`LayoutNode`], preserving the internal
    /// pseudo-element chain.
    fn as_node(&self) -> Self::ConcreteLayoutNode;

    /// Returns access to a version of this LayoutElement that can be used by stylo
    /// and selectors. This is dangerous as it allows more access to ancestor nodes
    /// that might be in the process of being read or written to in other threads.
    /// This should *only* be used when handing a node to stylo or selectors.
    ///
    /// # Safety
    ///
    /// This should only ever be called from the main script thread. It is never
    /// okay to explicitly create a node for style while any layout worker threads
    /// are running.
    unsafe fn dangerous_style_element(self) -> Self::ConcreteStyleElement;

    /// Initialize this node with empty style and opaque layout data.
    fn initialize_style_and_layout_data<RequestedLayoutDataType: LayoutDataTrait>(&self);

    /// Unset the snapshot flags on the underlying DOM object for this element.
    fn unset_snapshot_flags(&self);

    /// Set the snapshot flags on the underlying DOM object for this element.
    fn set_has_snapshot(&self);

    /// Get the [`StyleData`] for this [`LayoutElement`].
    fn style_data(self) -> Option<&'dom StyleData>;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<LayoutNodeType>;

    /// Get the local name of this element. See
    /// <https://dom.spec.whatwg.org/#concept-element-local-name>.
    fn local_name(&self) -> &LocalName;

    /// Get the attribute with the given `namespace` and `name` as an [`AttrValue`] if it
    /// exists, otherwise return `None`.
    fn attribute(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue>;

    /// Get the attribute with the given `namespace` and `name` as an [`AttrValue`] if it
    /// exists and converting the result to a `&str`, otherwise return `None`.
    fn attribute_as_str<'a>(&'a self, namespace: &Namespace, name: &LocalName) -> Option<&'a str>;

    /// Get a reference to the inner [`ElementDataRef`] for this element's [`StyleData`]. This will
    /// panic if the element is unstyled.
    fn element_data(&self) -> ElementDataRef<'dom>;

    /// Get a mutable reference to the inner [`ElementDataRef`] for this element's [`StyleData`].
    /// This will panic if the element is unstyled.
    fn element_data_mut(&self) -> ElementDataMut<'dom>;

    /// Returns the computed style for the given element, properly handling pseudo-elements.
    ///
    /// # Panics
    ///
    /// Calling this method will panic if the element has no style data, whether because styling has
    /// not run yet or was not run for this element.
    fn style(&self, context: &SharedStyleContext) -> Arc<ComputedValues>;

    /// Returns `true` if this [`LayoutElement`] is a shadow DOM host and `false` otherwise.
    fn is_shadow_host(&self) -> bool;

    /// Returns whether this node is a body element of an HTML element root
    /// in an HTML document.
    ///
    /// Note that this does require accessing the parent, which this interface
    /// technically forbids. But accessing the parent is only unsafe insofar as
    /// it can be used to reach siblings and cousins. A simple immutable borrow
    /// of the parent data is fine, since the bottom-up traversal will not process
    /// the parent until all the children have been processed.
    fn is_body_element_of_html_element_root(&self) -> bool;

    /// Returns `true` if this [`LayoutNode`] is any kind of HTML element inside an HTML document
    /// and `false` otherwise.
    fn is_html_element_in_html_document(&self) -> bool;

    /// Returns whether this node is the root element in an HTML document element.
    ///
    /// Note that, like `Self::is_body_element_of_html_element_root`, this accesses the parent.
    /// As in that case, since this is an immutable borrow, we do not violate thread safety.
    fn is_root(&self) -> bool;
}

/// An element that can be passed to `stylo` and `selectors` that allows accessing the
/// parent node. We consider this to be too dangerous for normal layout, so it is
/// reserved only for using `stylo` and `selectors`.
///
/// If you are not interfacing with `stylo` and `selectors` you *should not* use this
/// type, unless you know what you are doing.
pub trait DangerousStyleElement<'dom>:
    TElement + ::selectors::Element<Impl = SelectorImpl> + Send + Sync
{
    /// The concrete implementation of [`LayoutElement`] implemented in `script`.
    type ConcreteLayoutElement: LayoutElement<'dom>;
    /// The concrete implementation of [`LayoutNode`] implemented in `script`.
    /// Get a handle to the original "safe" version of this element, a [`LayoutElement`].
    fn layout_element(&self) -> Self::ConcreteLayoutElement;
}
