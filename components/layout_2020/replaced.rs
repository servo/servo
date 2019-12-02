/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom_traversal::NodeExt;
use crate::formatting_contexts::IndependentLayout;
use crate::fragments::{Fragment, ImageFragment};
use crate::geom::{flow_relative, physical};
use crate::ContainingBlock;
use net_traits::image::base::Image;
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::properties::ComputedValues;
use style::values::computed::Length;

#[derive(Debug)]
pub(crate) struct ReplacedContent {
    pub kind: ReplacedContentKind,
    pub intrinsic_size: physical::Vec2<Length>,
}

#[derive(Debug)]
pub(crate) enum ReplacedContentKind {
    Image(Option<Arc<Image>>),
}

impl ReplacedContent {
    pub fn for_element<'dom>(element: impl NodeExt<'dom>) -> Option<Self> {
        if let Some((image, intrinsic_size)) = element.as_image() {
            return Some(Self {
                kind: ReplacedContentKind::Image(image),
                intrinsic_size,
            });
        }
        None
    }

    pub fn layout<'a>(
        &'a self,
        style: &ServoArc<ComputedValues>,
        containing_block: &ContainingBlock,
    ) -> IndependentLayout {
        let (fragments, content_block_size) = match self.kind {
            ReplacedContentKind::Image(ref image) => {
                // FIXME(nox): We should not assume block size is known.
                let block_size = containing_block.block_size.non_auto().unwrap();
                let fragments = image
                    .as_ref()
                    .and_then(|image| image.id)
                    .map(|image_key| {
                        Fragment::Image(ImageFragment {
                            style: style.clone(),
                            content_rect: flow_relative::Rect {
                                start_corner: flow_relative::Vec2::zero(),
                                size: flow_relative::Vec2 {
                                    inline: containing_block.inline_size,
                                    block: block_size,
                                },
                            },
                            image_key,
                        })
                    })
                    .into_iter()
                    .collect::<Vec<_>>();
                (fragments, block_size)
            },
        };
        IndependentLayout {
            fragments,
            content_block_size,
        }
    }
}
