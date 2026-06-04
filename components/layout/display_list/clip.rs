/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use servo_base::id::ScrollTreeNodeId;
use style::values::computed::basic_shape::{BasicShape, ClipPath};
use style::values::computed::position::Position;
use style::values::computed::{Length, LengthPercentage};
use style::values::generics::basic_shape::{GenericShapeRadius, ShapeBox, ShapeGeometryBox};
use style::values::generics::position::GenericPositionOrAuto;
use webrender_api::BorderRadius;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutSideOffsets, LayoutSize};

use super::{BuilderForBoxFragment, compute_margin_box_radius, normalize_radii};
use crate::fragment_tree::BoxFragment;
use crate::geom::PhysicalPoint;

/// An identifier for a clip used during StackingContextTree construction. This is a simple index in
/// a [`ClipStore`]s vector of clips.
#[derive(Clone, Copy, Debug, Eq, Hash, MallocSizeOf, PartialEq)]
pub(crate) struct ClipId(pub usize);

impl ClipId {
    /// Equivalent to [`ClipChainId::INVALID`]. This means "no clip."
    pub(crate) const INVALID: ClipId = ClipId(usize::MAX);
}

impl Default for ClipId {
    fn default() -> Self {
        Self::INVALID
    }
}

/// All the information needed to create a clip on a WebRender display list. These are created at
/// two times: during `StackingContextTree` creation and during WebRender display list construction.
/// Only the former are stored in a [`ClipStore`].
#[derive(Clone, MallocSizeOf)]
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
#[derive(Clone, Default, MallocSizeOf)]
pub(crate) struct StackingContextTreeClipStore(pub Vec<Clip>);

impl StackingContextTreeClipStore {
    pub(super) fn get(&self, clip_id: ClipId) -> &Clip {
        &self.0[clip_id.0]
    }

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
        clip_path: &ClipPath,
        parent_scroll_node_id: ScrollTreeNodeId,
        parent_clip_chain_id: ClipId,
        box_fragment: &BoxFragment,
        containing_block_origin: PhysicalPoint<Au>,
    ) -> Option<ClipId> {
        let geometry_box = match clip_path {
            ClipPath::Shape(_, ShapeGeometryBox::ShapeBox(shape_box)) => *shape_box,
            ClipPath::Shape(_, ShapeGeometryBox::ElementDependent) => ShapeBox::BorderBox,
            ClipPath::Box(ShapeGeometryBox::ShapeBox(shape_box)) => *shape_box,
            ClipPath::Box(ShapeGeometryBox::ElementDependent) => ShapeBox::BorderBox,
            _ => return None,
        };
        let fragment_builder = BuilderForBoxFragment::new(box_fragment, containing_block_origin);
        let layout_rect = match geometry_box {
            ShapeBox::BorderBox => fragment_builder.border_rect,
            ShapeBox::ContentBox => *fragment_builder.content_rect(),
            ShapeBox::PaddingBox => *fragment_builder.padding_rect(),
            ShapeBox::MarginBox => *fragment_builder.margin_rect(),
        };
        if let ClipPath::Shape(shape, _) = clip_path {
            match **shape {
                BasicShape::Circle(_) | BasicShape::Ellipse(_) | BasicShape::Rect(_) => self
                    .add_for_basic_shape(
                        shape,
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
                        fragment_builder.border_radius(),
                        layout_rect.size(),
                        fragment_builder.fragment,
                    ),
                    _ => fragment_builder.border_radius(),
                },
                layout_rect,
                parent_scroll_node_id,
                parent_clip_chain_id,
            ))
        }
    }

    #[servo_tracing::instrument(name = "StackingContextClipStore::add_for_basic_shape", skip_all)]
    fn add_for_basic_shape(
        &mut self,
        shape: &BasicShape,
        layout_box: LayoutRect,
        parent_scroll_node_id: ScrollTreeNodeId,
        parent_clip_chain_id: ClipId,
    ) -> Option<ClipId> {
        match shape {
            BasicShape::Rect(rect) => {
                let box_height = Length::new(layout_box.height());
                let box_width = Length::new(layout_box.width());
                let insets = LayoutSideOffsets::new(
                    rect.rect.0.resolve(box_height).px(),
                    rect.rect.1.resolve(box_width).px(),
                    rect.rect.2.resolve(box_height).px(),
                    rect.rect.3.resolve(box_width).px(),
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
                        corner.0.width.0.resolve(box_width).px(),
                        corner.0.height.0.resolve(box_height).px(),
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
                    parent_scroll_node_id,
                    parent_clip_chain_id,
                ))
            },
            BasicShape::Circle(circle) => {
                let center = match &circle.position {
                    GenericPositionOrAuto::Position(position) => position.clone(),
                    GenericPositionOrAuto::Auto => Position::center(),
                };
                let anchor_x = center.horizontal.resolve(Length::new(layout_box.width()));
                let anchor_y = center.vertical.resolve(Length::new(layout_box.height()));
                let center = layout_box
                    .min
                    .add_size(&LayoutSize::new(anchor_x.px(), anchor_y.px()));

                let radius = compute_shape_radius_for_circle(
                    center,
                    &circle.radius,
                    layout_box.min,
                    layout_box.max,
                );
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
                Some(self.add(radii, rect, parent_scroll_node_id, parent_clip_chain_id))
            },
            BasicShape::Ellipse(ellipse) => {
                let center = match &ellipse.position {
                    GenericPositionOrAuto::Position(position) => position.clone(),
                    GenericPositionOrAuto::Auto => Position::center(),
                };
                let anchor_x = center.horizontal.resolve(Length::new(layout_box.width()));
                let anchor_y = center.vertical.resolve(Length::new(layout_box.height()));
                let center = layout_box
                    .min
                    .add_size(&LayoutSize::new(anchor_x.px(), anchor_y.px()));

                let radius_x = compute_shape_radius_for_ellipse_axis(
                    center.x,
                    &ellipse.semiaxis_x,
                    layout_box.min.x,
                    layout_box.max.x,
                );
                let radius_y = compute_shape_radius_for_ellipse_axis(
                    center.y,
                    &ellipse.semiaxis_y,
                    layout_box.min.y,
                    layout_box.max.y,
                );
                let radius = LayoutSize::new(radius_x, radius_y);

                let mut radii = webrender_api::BorderRadius {
                    top_left: radius,
                    top_right: radius,
                    bottom_left: radius,
                    bottom_right: radius,
                };
                let start = center.add_size(&-radius);
                let rect = LayoutRect::from_origin_and_size(start, radius * 2.);
                normalize_radii(&rect, &mut radii);
                Some(self.add(radii, rect, parent_scroll_node_id, parent_clip_chain_id))
            },
            _ => None,
        }
    }
}

fn compute_shape_radius_for_circle(
    center: LayoutPoint,
    radius: &GenericShapeRadius<LengthPercentage>,
    min_edge: LayoutPoint,
    max_edge: LayoutPoint,
) -> f32 {
    let distance_from_min_edge = (min_edge - center).abs();
    let distance_from_max_edge = (max_edge - center).abs();
    match radius {
        GenericShapeRadius::FarthestSide => {
            let x = distance_from_min_edge.x.max(distance_from_max_edge.x);
            let y = distance_from_min_edge.y.max(distance_from_max_edge.y);
            x.max(y)
        },
        GenericShapeRadius::ClosestSide => {
            let x = distance_from_min_edge.x.min(distance_from_max_edge.x);
            let y = distance_from_min_edge.y.min(distance_from_max_edge.y);
            x.min(y)
        },
        GenericShapeRadius::Length(length) => {
            // https://www.w3.org/TR/css-shapes/#direction-agnostic-size
            let size = max_edge - min_edge;
            let basis = size.x.hypot(size.y) / std::f32::consts::SQRT_2;
            length.0.resolve(Length::new(basis)).px()
        },
    }
}

fn compute_shape_radius_for_ellipse_axis(
    center: f32,
    radius: &GenericShapeRadius<LengthPercentage>,
    min_edge: f32,
    max_edge: f32,
) -> f32 {
    let distance_from_min_edge = (min_edge - center).abs();
    let distance_from_max_edge = (max_edge - center).abs();
    match radius {
        GenericShapeRadius::FarthestSide => distance_from_min_edge.max(distance_from_max_edge),
        GenericShapeRadius::ClosestSide => distance_from_min_edge.min(distance_from_max_edge),
        GenericShapeRadius::Length(length) => {
            length.0.resolve(Length::new(max_edge - min_edge)).px()
        },
    }
}
