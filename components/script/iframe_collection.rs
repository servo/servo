/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use base::id::BrowsingContextId;
use euclid::{Scale, Size2D};
use fnv::FnvHashMap;
use script_layout_interface::IFrameSizes;
use script_traits::{IFrameSizeMsg, WindowSizeData, WindowSizeType};
use style_traits::CSSPixel;
use webrender_api::units::DevicePixel;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::htmliframeelement::HTMLIFrameElement;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::types::Document;
use crate::script_thread::with_script_thread;

#[derive(JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub(crate) struct IFrame {
    pub element: Dom<HTMLIFrameElement>,
    #[no_trace]
    pub size: Option<Size2D<f32, CSSPixel>>,
}

#[derive(Default, JSTraceable, MallocSizeOf)]
#[crown::unrooted_must_root_lint::must_root]
pub(crate) struct IFrameCollection {
    /// The version of the [`Document`] that this collection refers to. When the versions
    /// do not match, the collection will need to be rebuilt.
    document_version: Cell<u64>,
    /// The `<iframe>`s in the collection.
    iframes: Vec<IFrame>,
}

impl IFrameCollection {
    /// Validate that the collection is up-to-date with the given [`Document`]. If it isn't up-to-date
    /// rebuild it.
    pub(crate) fn validate(&mut self, document: &Document) {
        // TODO: Whether the DOM has changed at all can lead to rebuilding this collection
        // when it isn't necessary. A better signal might be if any `<iframe>` nodes have
        // been connected or disconnected.
        let document_node = DomRoot::from_ref(document.upcast::<Node>());
        let document_version = document_node.inclusive_descendants_version();
        if document_version == self.document_version.get() {
            return;
        }

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
        self.document_version.set(document_version);
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
    pub(crate) fn set_size(
        &mut self,
        browsing_context_id: BrowsingContextId,
        new_size: Size2D<f32, CSSPixel>,
    ) -> Option<Size2D<f32, CSSPixel>> {
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
        new_iframe_sizes: IFrameSizes,
        device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
    ) -> Vec<IFrameSizeMsg> {
        if new_iframe_sizes.is_empty() {
            return vec![];
        }

        new_iframe_sizes
            .into_iter()
            .filter_map(|(browsing_context_id, size)| {
                // Batch resize message to any local `Pipeline`s now, rather than waiting for them
                // to filter asynchronously through the `Constellation`. This allows the new value
                // to be reflected immediately in layout.
                let new_size = size.size;
                with_script_thread(|script_thread| {
                    script_thread.handle_resize_message(
                        size.pipeline_id,
                        WindowSizeData {
                            initial_viewport: new_size,
                            device_pixel_ratio,
                        },
                        WindowSizeType::Resize,
                    );
                });

                let old_size = self.set_size(browsing_context_id, new_size);
                // The `Constellation` should be up-to-date even when the in-ScriptThread pipelines
                // might not be.
                if old_size == Some(size.size) {
                    return None;
                }

                let size_type = match old_size {
                    Some(_) => WindowSizeType::Resize,
                    None => WindowSizeType::Initial,
                };

                Some(IFrameSizeMsg {
                    browsing_context_id,
                    size: new_size,
                    type_: size_type,
                })
            })
            .collect()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = DomRoot<HTMLIFrameElement>> + use<'_> {
        self.iframes.iter().map(|iframe| iframe.element.as_rooted())
    }
}
