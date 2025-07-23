/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use base::id::BrowsingContextId;
use constellation_traits::{IFrameSizeMsg, ScriptToConstellationMessage, WindowSizeType};
use embedder_traits::ViewportDetails;
use fnv::FnvHashMap;
use layout_api::IFrameSizes;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::types::{Document, Window};
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
    /// The `<iframe>`s in the collection.
    iframes: Vec<IFrame>,
    /// When true, the collection will need to be rebuilt.
    invalid: bool,
}

impl IFrameCollection {
    pub(crate) fn new() -> Self {
        Self {
            iframes: vec![],
            invalid: true,
        }
    }

    pub(crate) fn invalidate(&mut self) {
        self.invalid = true;
    }

    /// Validate that the collection is up-to-date with the given [`Document`]. If it isn't up-to-date
    /// rebuild it.
    pub(crate) fn validate(&mut self, document: &Document) {
        if !self.invalid {
            return;
        }
        let document_node = DomRoot::from_ref(document.upcast::<Node>());

        // Preserve any old sizes, but only for `<iframe>`s that already have a
        // BrowsingContextId and a set size.
        let mut old_sizes: FnvHashMap<_, _> = self
            .iframes
            .iter()
            .filter_map(
                |iframe| match (iframe.element.browsing_context_id(), iframe.size) {
                    (Some(browsing_context_id), Some(size)) => Some((browsing_context_id, size)),
                    _ => None,
                },
            )
            .collect();

        self.iframes = document_node
            .traverse_preorder(ShadowIncluding::Yes)
            .filter_map(DomRoot::downcast::<HTMLIFrameElement>)
            .map(|element| {
                let size = element
                    .browsing_context_id()
                    .and_then(|browsing_context_id| old_sizes.remove(&browsing_context_id));
                IFrame {
                    element: element.as_traced(),
                    size,
                }
            })
            .collect();
        self.invalid = false;
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
