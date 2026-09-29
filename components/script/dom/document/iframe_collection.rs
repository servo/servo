/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Ref;
use std::default::Default;
use std::iter;

use embedder_traits::ViewportDetails;
use js::context::{JSContext, NoGC};
use layout_api::IFrameSizes;
use paint_api::PinchZoomInfos;
use script_bindings::cell::DomRefCell;
use servo_base::id::BrowsingContextId;
use servo_constellation_traits::{IFrameSizeMsg, ScriptToConstellationMessage, WindowSizeType};

use crate::dom::NodeTraits;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::Node;
use crate::dom::types::Window;
use crate::event_loop::script_thread::with_script_thread;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct IFrame {
    pub(crate) element: Dom<HTMLIFrameElement>,
    #[no_trace]
    pub(crate) size: Option<ViewportDetails>,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct IFrameCollection {
    /// The `<iframe>`s in the collection. These are kept in DOM tree order to ensure that
    /// requestAnimationFrame callbacks respect that order.
    iframes: DomRefCell<Vec<IFrame>>,
    /// The same `<iframe>`s in [`Self::iframes`], but stored in insertion order for use
    /// in the `WindowProxy` subframe getter.
    iframes_in_insertion_order: DomRefCell<Vec<Dom<HTMLIFrameElement>>>,
}

impl IFrameCollection {
    pub(crate) fn new() -> Self {
        Self {
            iframes: Default::default(),
            iframes_in_insertion_order: Default::default(),
        }
    }

    pub(crate) fn add(&self, no_gc: &NoGC, iframe_element: &HTMLIFrameElement) {
        let iframe_node = iframe_element.upcast::<Node>();

        // During `moveBefore`, nodes are attached to the tree again without detaching
        // them in order to preserve state. Here we remove any pre-existing entry for
        // this iframe element from the collection and preserve its old size.
        let size = self.remove(no_gc, iframe_element);

        // Look forward for the next `<iframe>` in the document in order to find the new
        // insertion point in the DOM-ordered list of frames. This optimizes for the parser
        // case where the `<iframe>` is likely being inserted at the end of the DOM and there
        // are very few subsequent nodes.
        let insertion_index = iframe_node
            .following_nodes(
                iframe_element.owner_document().upcast::<Node>(),
                ShadowIncluding::Yes,
            )
            .find_map(DomRoot::downcast::<HTMLIFrameElement>)
            .and_then(|following_iframe| {
                self.iframes
                    .borrow()
                    .iter()
                    .position(|iframe| *iframe.element == *following_iframe)
            })
            .unwrap_or(self.iframes.borrow().len());

        self.iframes.safe_borrow_mut(no_gc).insert(
            insertion_index,
            IFrame {
                element: Dom::from_ref(iframe_element),
                size,
            },
        );

        self.iframes_in_insertion_order
            .safe_borrow_mut(no_gc)
            .push(Dom::from_ref(iframe_element));
    }

    pub(crate) fn remove(
        &self,
        no_gc: &NoGC,
        iframe_element: &HTMLIFrameElement,
    ) -> Option<ViewportDetails> {
        self.iframes_in_insertion_order
            .safe_borrow_mut(no_gc)
            .retain(|iframe| *iframe != iframe_element);
        let mut iframes = self.iframes.safe_borrow_mut(no_gc);
        iframes
            .iter()
            .position(|iframe| &*iframe.element == iframe_element)
            .and_then(|index| iframes.remove(index).size)
    }

    /// Get the [`BrowsingContextId`] of the `<iframe>` element at the given
    /// position in insertion order, filtering out `<iframe>`s that do not have
    /// a browsing context.
    pub(crate) fn at_insertion_index(&self, index: usize) -> Option<BrowsingContextId> {
        self.iframes_in_insertion_order
            .borrow()
            .iter()
            .filter_map(|iframe| iframe.browsing_context_id())
            .nth(index)
    }

    /// Get a count of the iframes in this [`IframeCollection`] that have active browsing
    /// contexts.
    pub(crate) fn active_iframe_count(&self) -> usize {
        self.iframes_in_insertion_order
            .borrow()
            .iter()
            .filter(|iframe| iframe.browsing_context_id().is_some())
            .count()
    }

    /// Get an iframe elment matching the provided browsing context id, if it exists.
    pub(crate) fn element(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<DomRoot<HTMLIFrameElement>> {
        self.iframes
            .borrow()
            .iter()
            .find(|iframe| iframe.element.browsing_context_id() == Some(browsing_context_id))
            .map(|iframe| iframe.element.as_rooted())
    }

    /// Get the viewport details for the iframe matching the provided browsing context id, if it exists.
    pub(crate) fn viewport_details(
        &self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<ViewportDetails> {
        self.iframes
            .borrow()
            .iter()
            .find(|iframe| iframe.element.browsing_context_id() == Some(browsing_context_id))
            .and_then(|iframe| iframe.size)
    }

    /// Set the size of an `<iframe>` in the collection given its `BrowsingContextId` and
    /// the new size. Returns the old size.
    fn set_viewport_details(
        &self,
        no_gc: &NoGC,
        browsing_context_id: BrowsingContextId,
        new_size: ViewportDetails,
    ) -> Option<ViewportDetails> {
        // Top-level document destruction can destroy an entire tree of frames, which
        // means that the the `<iframe>` we are targeting at this moment might not exist.
        self.iframes
            .safe_borrow_mut(no_gc)
            .iter_mut()
            .find(|iframe| iframe.element.browsing_context_id() == Some(browsing_context_id))
            .and_then(|iframe| iframe.size.replace(new_size))
    }

    /// Update the recorded iframe sizes of the contents of layout. Return a
    /// [`Vec<IFrameSizeMsg>`] containing the messages to send to the `Constellation`. A
    /// message is only sent when the size actually changes.
    pub(crate) fn handle_new_iframe_sizes_after_layout(
        &self,
        cx: &mut JSContext,
        window: &Window,
        new_iframe_sizes: IFrameSizes,
    ) {
        if new_iframe_sizes.is_empty() {
            return;
        }

        let size_messages: Vec<_> = new_iframe_sizes
            .into_iter()
            .filter_map(|(browsing_context_id, iframe_size)| {
                // Batch resize message to any local `Pipeline`s now, rather than waiting for them
                // to filter asynchronously through the `Constellation`. This allows the new value
                // to be reflected immediately in layout.
                let viewport_details = iframe_size.viewport_details;
                with_script_thread(|script_thread| {
                    script_thread.handle_resize_message(
                        iframe_size.pipeline_id,
                        viewport_details,
                        WindowSizeType::Resize,
                    );

                    // Additionally, update the `VisualViewport` of the `Iframe`. This allows us
                    // to process the resize for `VisualViewport` in the corrent timing. Note that
                    // `VisualViewport` for iframes would practically follow layout viewport.
                    script_thread.handle_update_pinch_zoom_infos(
                        cx,
                        iframe_size.pipeline_id,
                        PinchZoomInfos::new_from_viewport_size(viewport_details.size),
                    )
                });

                let old_viewport_details =
                    self.set_viewport_details(cx.no_gc(), browsing_context_id, viewport_details);
                // The `Constellation` should be up-to-date even when the in-ScriptThread pipelines
                // might not be.
                if old_viewport_details == Some(viewport_details) {
                    return None;
                }

                let size_type = match old_viewport_details {
                    Some(_) => WindowSizeType::Resize,
                    None => WindowSizeType::Initial,
                };

                Some(IFrameSizeMsg {
                    browsing_context_id,
                    size: viewport_details,
                    type_: size_type,
                })
            })
            .collect();

        if !size_messages.is_empty() {
            window.send_to_constellation(ScriptToConstellationMessage::IFrameSizes(size_messages));
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = DomRoot<HTMLIFrameElement>> + use<'_> {
        let mut items = Some(Ref::map(self.iframes.borrow(), |vec| &vec[..]));
        iter::from_fn(move || {
            let mut item = None;
            let rest = Ref::map(items.take()?, |items| {
                let mut iter = items.iter();
                item = iter.next().map(|item| item.element.as_rooted());
                iter.as_slice()
            });
            if item.is_some() {
                items = Some(rest);
            }
            item
        })
    }
}
