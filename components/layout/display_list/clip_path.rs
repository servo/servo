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
use webrender_api::ClipChainId;
use webrender_api::units::{LayoutRect, LayoutSideOffsets, LayoutSize};

use super::{BuilderForBoxFragment, DisplayList, compute_margin_box_radius, normalize_radii};

pub(super) fn build_clip_path_clip_chain_if_necessary(
    clip_path: ClipPath,
    display_list: &mut DisplayList,
    parent_scroll_node_id: &ScrollTreeNodeId,
    parent_clip_chain_id: &ClipChainId,
    fragment_builder: BuilderForBoxFragment,
) -> Option<ClipChainId> {
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
            BasicShape::Circle(_) | BasicShape::Ellipse(_) | BasicShape::Rect(_) => {
                build_simple_shape(
                    *shape,
                    layout_rect,
                    parent_scroll_node_id,
                    parent_clip_chain_id,
                    display_list,
                )
            },
            BasicShape::Polygon(_) | BasicShape::PathOrShape(_) => None,
        }
    } else {
        Some(create_rect_clip_chain(
            match geometry_box {
                ShapeBox::MarginBox => compute_margin_box_radius(
                    fragment_builder.border_radius,
                    layout_rect.size(),
                    fragment_builder.fragment,
                ),
                _ => fragment_builder.border_radius,
            },
            layout_rect,
            parent_scroll_node_id,
            parent_clip_chain_id,
            display_list,
        ))
    }
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "display_list::build_simple_shape",
        skip_all,
        fields(servo_profiling = true),
        level = "trace",
    )
)]
fn build_simple_shape(
    shape: BasicShape,
    layout_box: LayoutRect,
    parent_scroll_node_id: &ScrollTreeNodeId,
    parent_clip_chain_id: &ClipChainId,
    display_list: &mut DisplayList,
) -> Option<ClipChainId> {
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
            Some(create_rect_clip_chain(
                radii,
                shape_rect,
                parent_scroll_node_id,
                parent_clip_chain_id,
                display_list,
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

            let horizontal =
                compute_shape_radius(center.x, &circle.radius, layout_box.min.x, layout_box.max.x);
            let vertical =
                compute_shape_radius(center.y, &circle.radius, layout_box.min.y, layout_box.max.y);

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
            Some(create_rect_clip_chain(
                radii,
                rect,
                parent_scroll_node_id,
                parent_clip_chain_id,
                display_list,
            ))
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
            Some(create_rect_clip_chain(
                radii,
                rect,
                parent_scroll_node_id,
                parent_clip_chain_id,
                display_list,
            ))
        },
        _ => None,
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
fn create_rect_clip_chain(
    radii: webrender_api::BorderRadius,
    rect: LayoutRect,
    parent_scroll_node_id: &ScrollTreeNodeId,
    parent_clip_chain_id: &ClipChainId,
    display_list: &mut DisplayList,
) -> ClipChainId {
    let new_clip_id = if radii.is_zero() {
        display_list
            .wr
            .define_clip_rect(parent_scroll_node_id.spatial_id, rect)
    } else {
        display_list.wr.define_clip_rounded_rect(
            parent_scroll_node_id.spatial_id,
            webrender_api::ComplexClipRegion {
                rect,
                radii,
                mode: webrender_api::ClipMode::Clip,
            },
        )
    };
    display_list.define_clip_chain(*parent_clip_chain_id, [new_clip_id])
}
