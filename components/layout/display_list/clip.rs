/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use base::id::ScrollTreeNodeId;
use style::values::computed::basic_shape::{BasicShape, ClipPath};
use style::values::computed::length_percentage::NonNegativeLengthPercentage;
use style::values::computed::position::Position;
use style::values::generics::basic_shape::{GenericShapeRadius, ShapeBox, ShapeGeometryBox};
use style::values::generics::position::GenericPositionOrAuto;
use webrender_api::BorderRadius;
use webrender_api::units::{LayoutRect, LayoutSideOffsets, LayoutSize};

use super::{BuilderForBoxFragment, compute_margin_box_radius, normalize_radii};

/// An identifier for a clip used during StackingContextTree construction. This is a simple index in
/// a [`ClipStore`]s vector of clips.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ClipId(pub usize);

impl ClipId {
    /// Equivalent to [`ClipChainId::INVALID`]. This means "no clip."
    pub(crate) const INVALID: ClipId = ClipId(usize::MAX);
}

/// All the information needed to create a clip on a WebRender display list. These are created at
/// two times: during `StackingContextTree` creation and during WebRender display list construction.
/// Only the former are stored in a [`ClipStore`].
#[derive(Clone)]
pub(crate) struct Clip {
    pub id: ClipId,
    pub radii: BorderRadius,
    pub rect: LayoutRect,
    pub parent_scroll_node_id: ScrollTreeNodeId,
    pub parent_clip_id: ClipId,
}

/// A simple vector of [`Clip`] that is built during `StackingContextTree` construction.
/// These are later turned into WebRender clips and clip chains during WebRender display
/// list construction.
#[derive(Clone, Default)]
pub(crate) struct StackingContextTreeClipStore(pub Vec<Clip>);

impl StackingContextTreeClipStore {
    pub(crate) fn add(
        &mut self,
        radii: webrender_api::BorderRadius,
        rect: LayoutRect,
        parent_scroll_node_id: ScrollTreeNodeId,
        parent_clip_id: ClipId,
    ) -> ClipId {
        let id = ClipId(self.0.len());
        self.0.push(Clip {
            id,
            radii,
            rect,
            parent_scroll_node_id,
            parent_clip_id,
        });
        id
    }

    pub(super) fn add_for_clip_path(
        &mut self,
        clip_path: ClipPath,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_chain_id: &ClipId,
        fragment_builder: BuilderForBoxFragment,
    ) -> Option<ClipId> {
        let geometry_box = match clip_path {
            ClipPath::Shape(_, ShapeGeometryBox::ShapeBox(shape_box)) => shape_box,
            ClipPath::Shape(_, ShapeGeometryBox::ElementDependent) => ShapeBox::BorderBox,
            ClipPath::Box(ShapeGeometryBox::ShapeBox(shape_box)) => shape_box,
            ClipPath::Box(ShapeGeometryBox::ElementDependent) => ShapeBox::BorderBox,
            _ => return None,
        };
        let layout_rect = match geometry_box {
            ShapeBox::BorderBox => fragment_builder.border_rect,
            ShapeBox::ContentBox => *fragment_builder.content_rect(),
            ShapeBox::PaddingBox => *fragment_builder.padding_rect(),
            ShapeBox::MarginBox => *fragment_builder.margin_rect(),
        };
        if let ClipPath::Shape(shape, _) = clip_path {
            match *shape {
                BasicShape::Circle(_) | BasicShape::Ellipse(_) | BasicShape::Rect(_) => self
                    .add_for_basic_shape(
                        *shape,
                        layout_rect,
                        parent_scroll_node_id,
                        parent_clip_chain_id,
                    ),
                BasicShape::Polygon(_) | BasicShape::PathOrShape(_) => None,
            }
        } else {
            Some(self.add(
                match geometry_box {
                    ShapeBox::MarginBox => compute_margin_box_radius(
                        fragment_builder.border_radius,
                        layout_rect.size(),
                        fragment_builder.fragment,
                    ),
                    _ => fragment_builder.border_radius,
                },
                layout_rect,
                *parent_scroll_node_id,
                *parent_clip_chain_id,
            ))
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            name = "StackingContextClipStore::add_for_basic_shape",
            skip_all,
            fields(servo_profiling = true),
            level = "trace",
        )
    )]
    fn add_for_basic_shape(
        &mut self,
        shape: BasicShape,
        layout_box: LayoutRect,
        parent_scroll_node_id: &ScrollTreeNodeId,
        parent_clip_chain_id: &ClipId,
    ) -> Option<ClipId> {
        match shape {
            BasicShape::Rect(rect) => {
                let box_height = Au::from_f32_px(layout_box.height());
                let box_width = Au::from_f32_px(layout_box.width());
                let insets = LayoutSideOffsets::new(
                    rect.rect.0.to_used_value(box_height).to_f32_px(),
                    rect.rect.1.to_used_value(box_width).to_f32_px(),
                    rect.rect.2.to_used_value(box_height).to_f32_px(),
                    rect.rect.3.to_used_value(box_width).to_f32_px(),
                );

                // `inner_rect()` will cause an assertion failure if the insets are larger than the
                // rectangle dimension.
                let shape_rect = if insets.left + insets.right >= layout_box.width() ||
                    insets.top + insets.bottom > layout_box.height()
                {
                    LayoutRect::from_origin_and_size(layout_box.min, LayoutSize::zero())
                } else {
                    layout_box.to_rect().inner_rect(insets).to_box2d()
                };

                let corner = |corner: &style::values::computed::BorderCornerRadius| {
                    LayoutSize::new(
                        corner.0.width.0.to_used_value(box_width).to_f32_px(),
                        corner.0.height.0.to_used_value(box_height).to_f32_px(),
                    )
                };
                let mut radii = webrender_api::BorderRadius {
                    top_left: corner(&rect.round.top_left),
                    top_right: corner(&rect.round.top_right),
                    bottom_left: corner(&rect.round.bottom_left),
                    bottom_right: corner(&rect.round.bottom_right),
                };
                normalize_radii(&layout_box, &mut radii);
                Some(self.add(
                    radii,
                    shape_rect,
                    *parent_scroll_node_id,
                    *parent_clip_chain_id,
                ))
            },
            BasicShape::Circle(circle) => {
                let center = match circle.position {
                    GenericPositionOrAuto::Position(position) => position,
                    GenericPositionOrAuto::Auto => Position::center(),
                };
                let anchor_x = center
                    .horizontal
                    .to_used_value(Au::from_f32_px(layout_box.width()));
                let anchor_y = center
                    .vertical
                    .to_used_value(Au::from_f32_px(layout_box.height()));
                let center = layout_box
                    .min
                    .add_size(&LayoutSize::new(anchor_x.to_f32_px(), anchor_y.to_f32_px()));

                let horizontal = compute_shape_radius(
                    center.x,
                    &circle.radius,
                    layout_box.min.x,
                    layout_box.max.x,
                );
                let vertical = compute_shape_radius(
                    center.y,
                    &circle.radius,
                    layout_box.min.y,
                    layout_box.max.y,
                );

                // If the value is `Length` then both values should be the same at this point.
                let radius = match circle.radius {
                    GenericShapeRadius::FarthestSide => horizontal.max(vertical),
                    GenericShapeRadius::ClosestSide => horizontal.min(vertical),
                    GenericShapeRadius::Length(_) => horizontal,
                };
                let radius = LayoutSize::new(radius, radius);
                let mut radii = webrender_api::BorderRadius {
                    top_left: radius,
                    top_right: radius,
                    bottom_left: radius,
                    bottom_right: radius,
                };
                let start = center.add_size(&-radius);
                let rect = LayoutRect::from_origin_and_size(start, radius * 2.);
                normalize_radii(&layout_box, &mut radii);
                Some(self.add(radii, rect, *parent_scroll_node_id, *parent_clip_chain_id))
            },
            BasicShape::Ellipse(ellipse) => {
                let center = match ellipse.position {
                    GenericPositionOrAuto::Position(position) => position,
                    GenericPositionOrAuto::Auto => Position::center(),
                };
                let anchor_x = center
                    .horizontal
                    .to_used_value(Au::from_f32_px(layout_box.width()));
                let anchor_y = center
                    .vertical
                    .to_used_value(Au::from_f32_px(layout_box.height()));
                let center = layout_box
                    .min
                    .add_size(&LayoutSize::new(anchor_x.to_f32_px(), anchor_y.to_f32_px()));

                let width = compute_shape_radius(
                    center.x,
                    &ellipse.semiaxis_x,
                    layout_box.min.x,
                    layout_box.max.x,
                );
                let height = compute_shape_radius(
                    center.y,
                    &ellipse.semiaxis_y,
                    layout_box.min.y,
                    layout_box.max.y,
                );

                let mut radii = webrender_api::BorderRadius {
                    top_left: LayoutSize::new(width, height),
                    top_right: LayoutSize::new(width, height),
                    bottom_left: LayoutSize::new(width, height),
                    bottom_right: LayoutSize::new(width, height),
                };
                let size = LayoutSize::new(width, height);
                let start = center.add_size(&-size);
                let rect = LayoutRect::from_origin_and_size(start, size * 2.);
                normalize_radii(&rect, &mut radii);
                Some(self.add(radii, rect, *parent_scroll_node_id, *parent_clip_chain_id))
            },
            _ => None,
        }
    }
}

fn compute_shape_radius(
    center: f32,
    radius: &GenericShapeRadius<NonNegativeLengthPercentage>,
    min_edge: f32,
    max_edge: f32,
) -> f32 {
    let distance_from_min_edge = (min_edge - center).abs();
    let distance_from_max_edge = (max_edge - center).abs();
    match radius {
        GenericShapeRadius::FarthestSide => distance_from_min_edge.max(distance_from_max_edge),
        GenericShapeRadius::ClosestSide => distance_from_min_edge.min(distance_from_max_edge),
        GenericShapeRadius::Length(length) => length
            .0
            .to_used_value(Au::from_f32_px(max_edge - min_edge))
            .to_f32_px(),
    }
}
