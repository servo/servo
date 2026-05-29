/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use embedder_traits::ViewportDetails;
use layout_api::IFrameSizes;
use paint_api::PinchZoomInfos;
use script_bindings::script_runtime::CanGc;
use servo_base::id::BrowsingContextId;
use servo_constellation_traits::{IFrameSizeMsg, ScriptToConstellationMessage, WindowSizeType};

use crate::dom::NodeTraits;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::Node;
use crate::dom::types::Window;
use crate::script_thread::with_script_thread;

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
    iframes: Vec<IFrame>,
}

impl IFrameCollection {
    pub(crate) fn new() -> Self {
        Self {
            iframes: Default::default(),
        }
    }

    pub(crate) fn add(&mut self, iframe_element: &HTMLIFrameElement) {
        let iframe_node = iframe_element.upcast::<Node>();

        // During `moveBefore`, nodes are attached to the tree again without detaching
        // them in order to preserve state. Here we remove any pre-existing entry for
        // this iframe element from the collection and preserve its old size.
        let size = self.remove(iframe_element);

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
                    .iter()
                    .position(|iframe| *iframe.element == *following_iframe)
            })
            .unwrap_or(self.iframes.len());

        self.iframes.insert(
            insertion_index,
            IFrame {
                element: Dom::from_ref(iframe_element),
                size,
            },
        );
    }

    pub(crate) fn remove(&mut self, iframe_element: &HTMLIFrameElement) -> Option<ViewportDetails> {
        self.iframes
            .iter()
            .position(|iframe| &*iframe.element == iframe_element)
            .and_then(|index| self.iframes.remove(index).size)
    }

    pub(crate) fn get(&self, browsing_context_id: BrowsingContextId) -> Option<&IFrame> {
        self.iframes
            .iter()
            .find(|iframe| iframe.element.browsing_context_id() == Some(browsing_context_id))
    }

    pub(crate) fn get_mut(
        &mut self,
        browsing_context_id: BrowsingContextId,
    ) -> Option<&mut IFrame> {
        self.iframes
            .iter_mut()
            .find(|iframe| iframe.element.browsing_context_id() == Some(browsing_context_id))
    }

    /// Set the size of an `<iframe>` in the collection given its `BrowsingContextId` and
    /// the new size. Returns the old size.
    pub(crate) fn set_viewport_details(
        &mut self,
        browsing_context_id: BrowsingContextId,
        new_size: ViewportDetails,
    ) -> Option<ViewportDetails> {
        self.get_mut(browsing_context_id)
            .expect("Tried to set a size for an unknown <iframe>")
            .size
            .replace(new_size)
    }

    /// Update the recorded iframe sizes of the contents of layout. Return a
    /// [`Vec<IFrameSizeMsg>`] containing the messages to send to the `Constellation`. A
    /// message is only sent when the size actually changes.
    pub(crate) fn handle_new_iframe_sizes_after_layout(
        &mut self,
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
                        iframe_size.pipeline_id,
                        PinchZoomInfos::new_from_viewport_size(viewport_details.size),
                        // Theoritically it wouldn't do GC since it is impossible to initialize
                        // the `VisualViewport` interface here.
                        CanGc::deprecated_note(),
                    )
                });

                let old_viewport_details =
                    self.set_viewport_details(browsing_context_id, viewport_details);
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
        self.iframes.iter().map(|iframe| iframe.element.as_rooted())
    }
}
