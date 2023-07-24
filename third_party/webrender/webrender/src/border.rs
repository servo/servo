/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{BorderRadius, BorderSide, BorderStyle, ColorF, ColorU};
use api::{NormalBorder as ApiNormalBorder, RepeatMode, EdgeAaSegmentMask};
use api::units::*;
use crate::clip::ClipChainId;
use crate::ellipse::Ellipse;
use euclid::vec2;
use crate::scene_building::SceneBuilder;
use crate::spatial_tree::SpatialNodeIndex;
use crate::gpu_types::{BorderInstance, BorderSegment, BrushFlags};
use crate::prim_store::{BorderSegmentInfo, BrushSegment, NinePatchDescriptor};
use crate::prim_store::borders::{NormalBorderPrim, NormalBorderData};
use crate::util::{lerp, RectHelpers};
use crate::internal_types::LayoutPrimitiveInfo;

// Using 2048 as the maximum radius in device space before which we
// start stretching is up for debate.
// the value must be chosen so that the corners will not use an
// unreasonable amount of memory but should allow crisp corners in the
// common cases.

/// Maximum resolution in device pixels at which borders are rasterized.
pub const MAX_BORDER_RESOLUTION: u32 = 2048;
/// Maximum number of dots or dashes per segment to avoid freezing and filling up
/// memory with unreasonable inputs. It would be better to address this by not building
/// a list of per-dot information in the first place.
pub const MAX_DASH_COUNT: u32 = 2048;

// TODO(gw): Perhaps there is a better way to store
//           the border cache key than duplicating
//           all the border structs with hashable
//           variants...

#[derive(Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BorderRadiusAu {
    pub top_left: LayoutSizeAu,
    pub top_right: LayoutSizeAu,
    pub bottom_left: LayoutSizeAu,
    pub bottom_right: LayoutSizeAu,
}

impl From<BorderRadius> for BorderRadiusAu {
    fn from(radius: BorderRadius) -> BorderRadiusAu {
        BorderRadiusAu {
            top_left: radius.top_left.to_au(),
            top_right: radius.top_right.to_au(),
            bottom_right: radius.bottom_right.to_au(),
            bottom_left: radius.bottom_left.to_au(),
        }
    }
}

impl From<BorderRadiusAu> for BorderRadius {
    fn from(radius: BorderRadiusAu) -> Self {
        BorderRadius {
            top_left: LayoutSize::from_au(radius.top_left),
            top_right: LayoutSize::from_au(radius.top_right),
            bottom_right: LayoutSize::from_au(radius.bottom_right),
            bottom_left: LayoutSize::from_au(radius.bottom_left),
        }
    }
}

#[derive(Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BorderSideAu {
    pub color: ColorU,
    pub style: BorderStyle,
}

impl From<BorderSide> for BorderSideAu {
    fn from(side: BorderSide) -> Self {
        BorderSideAu {
            color: side.color.into(),
            style: side.style,
        }
    }
}

impl From<BorderSideAu> for BorderSide {
    fn from(side: BorderSideAu) -> Self {
        BorderSide {
            color: side.color.into(),
            style: side.style,
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Hash, Eq, MallocSizeOf, PartialEq)]
pub struct NormalBorderAu {
    pub left: BorderSideAu,
    pub right: BorderSideAu,
    pub top: BorderSideAu,
    pub bottom: BorderSideAu,
    pub radius: BorderRadiusAu,
    /// Whether to apply anti-aliasing on the border corners.
    ///
    /// Note that for this to be `false` and work, this requires the borders to
    /// be solid, and no border-radius.
    pub do_aa: bool,
}

impl NormalBorderAu {
    // Construct a border based upon self with color
    pub fn with_color(&self, color: ColorU) -> Self {
        let mut b = self.clone();
        b.left.color = color;
        b.right.color = color;
        b.top.color = color;
        b.bottom.color = color;
        b
    }
}

impl From<ApiNormalBorder> for NormalBorderAu {
    fn from(border: ApiNormalBorder) -> Self {
        NormalBorderAu {
            left: border.left.into(),
            right: border.right.into(),
            top: border.top.into(),
            bottom: border.bottom.into(),
            radius: border.radius.into(),
            do_aa: border.do_aa,
        }
    }
}

impl From<NormalBorderAu> for ApiNormalBorder {
    fn from(border: NormalBorderAu) -> Self {
        ApiNormalBorder {
            left: border.left.into(),
            right: border.right.into(),
            top: border.top.into(),
            bottom: border.bottom.into(),
            radius: border.radius.into(),
            do_aa: border.do_aa,
        }
    }
}

/// Cache key that uniquely identifies a border
/// segment in the render task cache.
#[derive(Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct BorderSegmentCacheKey {
    pub size: LayoutSizeAu,
    pub radius: LayoutSizeAu,
    pub side0: BorderSideAu,
    pub side1: BorderSideAu,
    pub segment: BorderSegment,
    pub do_aa: bool,
    pub h_adjacent_corner_outer: LayoutPointAu,
    pub h_adjacent_corner_radius: LayoutSizeAu,
    pub v_adjacent_corner_outer: LayoutPointAu,
    pub v_adjacent_corner_radius: LayoutSizeAu,
}

pub fn ensure_no_corner_overlap(
    radius: &mut BorderRadius,
    size: LayoutSize,
) {
    let mut ratio = 1.0;
    let top_left_radius = &mut radius.top_left;
    let top_right_radius = &mut radius.top_right;
    let bottom_right_radius = &mut radius.bottom_right;
    let bottom_left_radius = &mut radius.bottom_left;

    let sum = top_left_radius.width + top_right_radius.width;
    if size.width < sum {
        ratio = f32::min(ratio, size.width / sum);
    }

    let sum = bottom_left_radius.width + bottom_right_radius.width;
    if size.width < sum {
        ratio = f32::min(ratio, size.width / sum);
    }

    let sum = top_left_radius.height + bottom_left_radius.height;
    if size.height < sum {
        ratio = f32::min(ratio, size.height / sum);
    }

    let sum = top_right_radius.height + bottom_right_radius.height;
    if size.height < sum {
        ratio = f32::min(ratio, size.height / sum);
    }

    if ratio < 1. {
        top_left_radius.width *= ratio;
        top_left_radius.height *= ratio;

        top_right_radius.width *= ratio;
        top_right_radius.height *= ratio;

        bottom_left_radius.width *= ratio;
        bottom_left_radius.height *= ratio;

        bottom_right_radius.width *= ratio;
        bottom_right_radius.height *= ratio;
    }
}

impl<'a> SceneBuilder<'a> {
    pub fn add_normal_border(
        &mut self,
        info: &LayoutPrimitiveInfo,
        border: &ApiNormalBorder,
        widths: LayoutSideOffsets,
        spatial_node_index: SpatialNodeIndex,
        clip_chain_id: ClipChainId,
    ) {
        let mut border = *border;
        ensure_no_corner_overlap(&mut border.radius, info.rect.size);

        self.add_primitive(
            spatial_node_index,
            clip_chain_id,
            info,
            Vec::new(),
            NormalBorderPrim {
                border: border.into(),
                widths: widths.to_au(),
            },
        );
    }
}

pub trait BorderSideHelpers {
    fn border_color(&self, is_inner_border: bool) -> ColorF;
}

impl BorderSideHelpers for BorderSide {
    fn border_color(&self, is_inner_border: bool) -> ColorF {
        let lighter = match self.style {
            BorderStyle::Inset => is_inner_border,
            BorderStyle::Outset => !is_inner_border,
            _ => return self.color,
        };

        // The modulate colors below are not part of the specification. They are
        // derived from the Gecko source code and experimentation, and used to
        // modulate the colors in order to generate colors for the inset/outset
        // and groove/ridge border styles.
        //
        // NOTE(emilio): Gecko at least takes the background color into
        // account, should we do the same? Looks a bit annoying for this.
        //
        // NOTE(emilio): If you change this algorithm, do the same change on
        // get_colors_for_side in cs_border_segment.glsl.
        if self.color.r != 0.0 || self.color.g != 0.0 || self.color.b != 0.0 {
            let scale = if lighter { 1.0 } else { 2.0 / 3.0 };
            return self.color.scale_rgb(scale)
        }

        let black = if lighter { 0.7 } else { 0.3 };
        ColorF::new(black, black, black, self.color.a)
    }
}

/// The kind of border corner clip.
#[repr(C)]
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum BorderClipKind {
    DashCorner = 1,
    DashEdge = 2,
    Dot = 3,
}

fn compute_outer_and_clip_sign(
    corner_segment: BorderSegment,
    radius: DeviceSize,
) -> (DevicePoint, DeviceVector2D) {
    let outer_scale = match corner_segment {
        BorderSegment::TopLeft => DeviceVector2D::new(0.0, 0.0),
        BorderSegment::TopRight => DeviceVector2D::new(1.0, 0.0),
        BorderSegment::BottomRight => DeviceVector2D::new(1.0, 1.0),
        BorderSegment::BottomLeft => DeviceVector2D::new(0.0, 1.0),
        _ => panic!("bug: expected a corner segment"),
    };
    let outer = DevicePoint::new(
        outer_scale.x * radius.width,
        outer_scale.y * radius.height,
    );

    let clip_sign = DeviceVector2D::new(
        1.0 - 2.0 * outer_scale.x,
        1.0 - 2.0 * outer_scale.y,
    );

    (outer, clip_sign)
}

fn write_dashed_corner_instances(
    corner_radius: DeviceSize,
    widths: DeviceSize,
    segment: BorderSegment,
    base_instance: &BorderInstance,
    instances: &mut Vec<BorderInstance>,
) -> Result<(), ()> {
    let ellipse = Ellipse::new(corner_radius);

    let average_border_width = 0.5 * (widths.width + widths.height);

    let (_half_dash, num_half_dashes) =
        compute_half_dash(average_border_width, ellipse.total_arc_length);

    if num_half_dashes == 0 {
        return Err(());
    }

    let num_half_dashes = num_half_dashes.min(MAX_DASH_COUNT);

    let (outer, clip_sign) = compute_outer_and_clip_sign(segment, corner_radius);

    let instance_count = num_half_dashes / 4 + 1;
    instances.reserve(instance_count as usize);

    let half_dash_arc_length =
        ellipse.total_arc_length / num_half_dashes as f32;
    let dash_length = 2. * half_dash_arc_length;

    let mut current_length = 0.;
    for i in 0..instance_count {
        let arc_length0 = current_length;
        current_length += if i == 0 {
            half_dash_arc_length
        } else {
            dash_length
        };

        let arc_length1 = current_length;
        current_length += dash_length;

        let alpha = ellipse.find_angle_for_arc_length(arc_length0);
        let beta = ellipse.find_angle_for_arc_length(arc_length1);

        let (point0, tangent0) = ellipse.get_point_and_tangent(alpha);
        let (point1, tangent1) = ellipse.get_point_and_tangent(beta);

        let point0 = DevicePoint::new(
            outer.x + clip_sign.x * (corner_radius.width - point0.x),
            outer.y + clip_sign.y * (corner_radius.height - point0.y),
        );

        let tangent0 = DeviceVector2D::new(
            -tangent0.x * clip_sign.x,
            -tangent0.y * clip_sign.y,
        );

        let point1 = DevicePoint::new(
            outer.x + clip_sign.x * (corner_radius.width - point1.x),
            outer.y + clip_sign.y * (corner_radius.height - point1.y),
        );

        let tangent1 = DeviceVector2D::new(
            -tangent1.x * clip_sign.x,
            -tangent1.y * clip_sign.y,
        );

        instances.push(BorderInstance {
            flags: base_instance.flags | ((BorderClipKind::DashCorner as i32) << 24),
            clip_params: [
                point0.x,
                point0.y,
                tangent0.x,
                tangent0.y,
                point1.x,
                point1.y,
                tangent1.x,
                tangent1.y,
            ],
            .. *base_instance
        });
    }

    Ok(())
}

fn write_dotted_corner_instances(
    corner_radius: DeviceSize,
    widths: DeviceSize,
    segment: BorderSegment,
    base_instance: &BorderInstance,
    instances: &mut Vec<BorderInstance>,
) -> Result<(), ()> {
    let mut corner_radius = corner_radius;
    if corner_radius.width < (widths.width / 2.0) {
        corner_radius.width = 0.0;
    }
    if corner_radius.height < (widths.height / 2.0) {
        corner_radius.height = 0.0;
    }

    let (ellipse, max_dot_count) =
        if corner_radius.width == 0. && corner_radius.height == 0. {
            (Ellipse::new(corner_radius), 1)
        } else {
            // The centers of dots follow an ellipse along the middle of the
            // border radius.
            let inner_radius = (corner_radius - widths * 0.5).abs();
            let ellipse = Ellipse::new(inner_radius);

            // Allocate a "worst case" number of dot clips. This can be
            // calculated by taking the minimum edge radius, since that
            // will result in the maximum number of dots along the path.
            let min_diameter = widths.width.min(widths.height);

            // Get the number of circles (assuming spacing of one diameter
            // between dots).
            let max_dot_count = 0.5 * ellipse.total_arc_length / min_diameter;

            // Add space for one extra dot since they are centered at the
            // start of the arc.
            (ellipse, max_dot_count.ceil() as usize)
        };

    if max_dot_count == 0 {
        return Err(());
    }

    if max_dot_count == 1 {
        let dot_diameter = lerp(widths.width, widths.height, 0.5);
        instances.push(BorderInstance {
            flags: base_instance.flags | ((BorderClipKind::Dot as i32) << 24),
            clip_params: [
                widths.width / 2.0, widths.height / 2.0, 0.5 * dot_diameter, 0.,
                0., 0., 0., 0.,
            ],
            .. *base_instance
        });
        return Ok(());
    }

    let max_dot_count = max_dot_count.min(MAX_DASH_COUNT as usize);

    // FIXME(emilio): Should probably use SmallVec.
    let mut forward_dots = Vec::with_capacity(max_dot_count / 2 + 1);
    let mut back_dots = Vec::with_capacity(max_dot_count / 2 + 1);
    let mut leftover_arc_length = 0.0;

    // Alternate between adding dots at the start and end of the
    // ellipse arc. This ensures that we always end up with an exact
    // half dot at each end of the arc, to match up with the edges.
    forward_dots.push(DotInfo::new(widths.width, widths.width));
    back_dots.push(DotInfo::new(
        ellipse.total_arc_length - widths.height,
        widths.height,
    ));

    let (outer, clip_sign) = compute_outer_and_clip_sign(segment, corner_radius);
    for dot_index in 0 .. max_dot_count {
        let prev_forward_pos = *forward_dots.last().unwrap();
        let prev_back_pos = *back_dots.last().unwrap();

        // Select which end of the arc to place a dot from.
        // This just alternates between the start and end of
        // the arc, which ensures that there is always an
        // exact half-dot at each end of the ellipse.
        let going_forward = dot_index & 1 == 0;

        let (next_dot_pos, leftover) = if going_forward {
            let next_dot_pos =
                prev_forward_pos.arc_pos + 2.0 * prev_forward_pos.diameter;
            (next_dot_pos, prev_back_pos.arc_pos - next_dot_pos)
        } else {
            let next_dot_pos = prev_back_pos.arc_pos - 2.0 * prev_back_pos.diameter;
            (next_dot_pos, next_dot_pos - prev_forward_pos.arc_pos)
        };

        // Use a lerp between each edge's dot
        // diameter, based on the linear distance
        // along the arc to get the diameter of the
        // dot at this arc position.
        let t = next_dot_pos / ellipse.total_arc_length;
        let dot_diameter = lerp(widths.width, widths.height, t);

        // If we can't fit a dot, bail out.
        if leftover < dot_diameter {
            leftover_arc_length = leftover;
            break;
        }

        // We can place a dot!
        let dot = DotInfo::new(next_dot_pos, dot_diameter);
        if going_forward {
            forward_dots.push(dot);
        } else {
            back_dots.push(dot);
        }
    }

    // Now step through the dots, and distribute any extra
    // leftover space on the arc between them evenly. Once
    // the final arc position is determined, generate the correct
    // arc positions and angles that get passed to the clip shader.
    let number_of_dots = forward_dots.len() + back_dots.len();
    let extra_space_per_dot = leftover_arc_length / (number_of_dots - 1) as f32;

    let create_dot_data = |arc_length: f32, dot_radius: f32| -> [f32; 8] {
        // Represents the GPU data for drawing a single dot to a clip mask. The order
        // these are specified must stay in sync with the way this data is read in the
        // dot clip shader.
        let theta = ellipse.find_angle_for_arc_length(arc_length);
        let (center, _) = ellipse.get_point_and_tangent(theta);

        let center = DevicePoint::new(
            outer.x + clip_sign.x * (corner_radius.width - center.x),
            outer.y + clip_sign.y * (corner_radius.height - center.y),
        );

        [center.x, center.y, dot_radius, 0.0, 0.0, 0.0, 0.0, 0.0]
    };

    instances.reserve(number_of_dots);
    for (i, dot) in forward_dots.iter().enumerate() {
        let extra_dist = i as f32 * extra_space_per_dot;
        instances.push(BorderInstance {
            flags: base_instance.flags | ((BorderClipKind::Dot as i32) << 24),
            clip_params: create_dot_data(dot.arc_pos + extra_dist, 0.5 * dot.diameter),
            .. *base_instance
        });
    }

    for (i, dot) in back_dots.iter().enumerate() {
        let extra_dist = i as f32 * extra_space_per_dot;
        instances.push(BorderInstance {
            flags: base_instance.flags | ((BorderClipKind::Dot as i32) << 24),
            clip_params: create_dot_data(dot.arc_pos - extra_dist, 0.5 * dot.diameter),
            .. *base_instance
        });
    }

    Ok(())
}

#[derive(Copy, Clone, Debug)]
struct DotInfo {
    arc_pos: f32,
    diameter: f32,
}

impl DotInfo {
    fn new(arc_pos: f32, diameter: f32) -> DotInfo {
        DotInfo { arc_pos, diameter }
    }
}

/// Information needed to place and draw a border edge.
#[derive(Debug)]
struct EdgeInfo {
    /// Offset in local space to place the edge from origin.
    local_offset: f32,
    /// Size of the edge in local space.
    local_size: f32,
    /// Local stretch size for this edge (repeat past this).
    stretch_size: f32,
}

impl EdgeInfo {
    fn new(
        local_offset: f32,
        local_size: f32,
        stretch_size: f32,
    ) -> Self {
        Self {
            local_offset,
            local_size,
            stretch_size,
        }
    }
}

// Given a side width and the available space, compute the half-dash (half of
// the 'on' segment) and the count of them for a given segment.
fn compute_half_dash(side_width: f32, total_size: f32) -> (f32, u32) {
    let half_dash = side_width * 1.5;
    let num_half_dashes = (total_size / half_dash).ceil() as u32;

    if num_half_dashes == 0 {
        return (0., 0);
    }

    // TODO(emilio): Gecko has some other heuristics here to start with a full
    // dash when the border side is zero, for example. We might consider those
    // in the future.
    let num_half_dashes = if num_half_dashes % 4 != 0 {
        num_half_dashes + 4 - num_half_dashes % 4
    } else {
        num_half_dashes
    };

    let half_dash = total_size / num_half_dashes as f32;
    (half_dash, num_half_dashes)
}


// Get the needed size in device pixels for an edge,
// based on the border style of that edge. This is used
// to determine how big the render task should be.
fn get_edge_info(
    style: BorderStyle,
    side_width: f32,
    avail_size: f32,
) -> EdgeInfo {
    // To avoid division by zero below.
    if side_width <= 0.0 || avail_size <= 0.0 {
        return EdgeInfo::new(0.0, 0.0, 0.0);
    }

    match style {
        BorderStyle::Dashed => {
            // Basically, two times the dash size.
            let (half_dash, _num_half_dashes) =
                compute_half_dash(side_width, avail_size);
            let stretch_size = 2.0 * 2.0 * half_dash;
            EdgeInfo::new(0., avail_size, stretch_size)
        }
        BorderStyle::Dotted => {
            let dot_and_space_size = 2.0 * side_width;
            if avail_size < dot_and_space_size * 0.75 {
                return EdgeInfo::new(0.0, 0.0, 0.0);
            }
            let approx_dot_count = avail_size / dot_and_space_size;
            let dot_count = approx_dot_count.floor().max(1.0);
            let used_size = dot_count * dot_and_space_size;
            let extra_space = avail_size - used_size;
            let stretch_size = dot_and_space_size;
            let offset = (extra_space * 0.5).round();
            EdgeInfo::new(offset, used_size, stretch_size)
        }
        _ => {
            EdgeInfo::new(0.0, avail_size, 8.0)
        }
    }
}

/// Create the set of border segments and render task
/// cache keys for a given CSS border.
pub fn create_border_segments(
    size: LayoutSize,
    border: &ApiNormalBorder,
    widths: &LayoutSideOffsets,
    border_segments: &mut Vec<BorderSegmentInfo>,
    brush_segments: &mut Vec<BrushSegment>,
) {
    let rect = LayoutRect::new(
        LayoutPoint::zero(),
        size,
    );

    let overlap = LayoutSize::new(
        (widths.left + widths.right - size.width).max(0.0),
        (widths.top + widths.bottom - size.height).max(0.0),
    );
    let non_overlapping_widths = LayoutSideOffsets::new(
        widths.top - overlap.height / 2.0,
        widths.right - overlap.width / 2.0,
        widths.bottom - overlap.height / 2.0,
        widths.left - overlap.width / 2.0,
    );

    let local_size_tl = LayoutSize::new(
        border.radius.top_left.width.max(widths.left),
        border.radius.top_left.height.max(widths.top),
    );
    let local_size_tr = LayoutSize::new(
        border.radius.top_right.width.max(widths.right),
        border.radius.top_right.height.max(widths.top),
    );
    let local_size_br = LayoutSize::new(
        border.radius.bottom_right.width.max(widths.right),
        border.radius.bottom_right.height.max(widths.bottom),
    );
    let local_size_bl = LayoutSize::new(
        border.radius.bottom_left.width.max(widths.left),
        border.radius.bottom_left.height.max(widths.bottom),
    );

    let top_edge_info = get_edge_info(
        border.top.style,
        widths.top,
        rect.size.width - local_size_tl.width - local_size_tr.width,
    );
    let bottom_edge_info = get_edge_info(
        border.bottom.style,
        widths.bottom,
        rect.size.width - local_size_bl.width - local_size_br.width,
    );

    let left_edge_info = get_edge_info(
        border.left.style,
        widths.left,
        rect.size.height - local_size_tl.height - local_size_bl.height,
    );
    let right_edge_info = get_edge_info(
        border.right.style,
        widths.right,
        rect.size.height - local_size_tr.height - local_size_br.height,
    );

    add_edge_segment(
        LayoutRect::from_floats(
            rect.origin.x,
            rect.origin.y + local_size_tl.height + left_edge_info.local_offset,
            rect.origin.x + non_overlapping_widths.left,
            rect.origin.y + local_size_tl.height + left_edge_info.local_offset + left_edge_info.local_size,
        ),
        &left_edge_info,
        border.left,
        non_overlapping_widths.left,
        BorderSegment::Left,
        EdgeAaSegmentMask::LEFT | EdgeAaSegmentMask::RIGHT,
        brush_segments,
        border_segments,
        border.do_aa,
    );
    add_edge_segment(
        LayoutRect::from_floats(
            rect.origin.x + local_size_tl.width + top_edge_info.local_offset,
            rect.origin.y,
            rect.origin.x + local_size_tl.width + top_edge_info.local_offset + top_edge_info.local_size,
            rect.origin.y + non_overlapping_widths.top,
        ),
        &top_edge_info,
        border.top,
        non_overlapping_widths.top,
        BorderSegment::Top,
        EdgeAaSegmentMask::TOP | EdgeAaSegmentMask::BOTTOM,
        brush_segments,
        border_segments,
        border.do_aa,
    );
    add_edge_segment(
        LayoutRect::from_floats(
            rect.origin.x + rect.size.width - non_overlapping_widths.right,
            rect.origin.y + local_size_tr.height + right_edge_info.local_offset,
            rect.origin.x + rect.size.width,
            rect.origin.y + local_size_tr.height + right_edge_info.local_offset + right_edge_info.local_size,
        ),
        &right_edge_info,
        border.right,
        non_overlapping_widths.right,
        BorderSegment::Right,
        EdgeAaSegmentMask::RIGHT | EdgeAaSegmentMask::LEFT,
        brush_segments,
        border_segments,
        border.do_aa,
    );
    add_edge_segment(
        LayoutRect::from_floats(
            rect.origin.x + local_size_bl.width + bottom_edge_info.local_offset,
            rect.origin.y + rect.size.height - non_overlapping_widths.bottom,
            rect.origin.x + local_size_bl.width + bottom_edge_info.local_offset + bottom_edge_info.local_size,
            rect.origin.y + rect.size.height,
        ),
        &bottom_edge_info,
        border.bottom,
        non_overlapping_widths.bottom,
        BorderSegment::Bottom,
        EdgeAaSegmentMask::BOTTOM | EdgeAaSegmentMask::TOP,
        brush_segments,
        border_segments,
        border.do_aa,
    );

    add_corner_segment(
        LayoutRect::from_floats(
            rect.origin.x,
            rect.origin.y,
            rect.origin.x + local_size_tl.width,
            rect.origin.y + local_size_tl.height,
        ),
        LayoutRect::from_floats(
            rect.origin.x,
            rect.origin.y,
            rect.max_x() - non_overlapping_widths.right,
            rect.max_y() - non_overlapping_widths.bottom
        ),
        border.left,
        border.top,
        LayoutSize::new(widths.left, widths.top),
        border.radius.top_left,
        BorderSegment::TopLeft,
        EdgeAaSegmentMask::TOP | EdgeAaSegmentMask::LEFT,
        rect.top_right(),
        border.radius.top_right,
        rect.bottom_left(),
        border.radius.bottom_left,
        brush_segments,
        border_segments,
        border.do_aa,
    );
    add_corner_segment(
        LayoutRect::from_floats(
            rect.origin.x + rect.size.width - local_size_tr.width,
            rect.origin.y,
            rect.origin.x + rect.size.width,
            rect.origin.y + local_size_tr.height,
        ),
        LayoutRect::from_floats(
            rect.origin.x + non_overlapping_widths.left,
            rect.origin.y,
            rect.max_x(),
            rect.max_y() - non_overlapping_widths.bottom,
        ),
        border.top,
        border.right,
        LayoutSize::new(widths.right, widths.top),
        border.radius.top_right,
        BorderSegment::TopRight,
        EdgeAaSegmentMask::TOP | EdgeAaSegmentMask::RIGHT,
        rect.origin,
        border.radius.top_left,
        rect.bottom_right(),
        border.radius.bottom_right,
        brush_segments,
        border_segments,
        border.do_aa,
    );
    add_corner_segment(
        LayoutRect::from_floats(
            rect.origin.x + rect.size.width - local_size_br.width,
            rect.origin.y + rect.size.height - local_size_br.height,
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        ),
        LayoutRect::from_floats(
            rect.origin.x + non_overlapping_widths.left,
            rect.origin.y + non_overlapping_widths.top,
            rect.max_x(),
            rect.max_y(),
        ),
        border.right,
        border.bottom,
        LayoutSize::new(widths.right, widths.bottom),
        border.radius.bottom_right,
        BorderSegment::BottomRight,
        EdgeAaSegmentMask::BOTTOM | EdgeAaSegmentMask::RIGHT,
        rect.bottom_left(),
        border.radius.bottom_left,
        rect.top_right(),
        border.radius.top_right,
        brush_segments,
        border_segments,
        border.do_aa,
    );
    add_corner_segment(
        LayoutRect::from_floats(
            rect.origin.x,
            rect.origin.y + rect.size.height - local_size_bl.height,
            rect.origin.x + local_size_bl.width,
            rect.origin.y + rect.size.height,
        ),
        LayoutRect::from_floats(
            rect.origin.x,
            rect.origin.y + non_overlapping_widths.top,
            rect.max_x() - non_overlapping_widths.right,
            rect.max_y(),
        ),
        border.bottom,
        border.left,
        LayoutSize::new(widths.left, widths.bottom),
        border.radius.bottom_left,
        BorderSegment::BottomLeft,
        EdgeAaSegmentMask::BOTTOM | EdgeAaSegmentMask::LEFT,
        rect.bottom_right(),
        border.radius.bottom_right,
        rect.origin,
        border.radius.top_left,
        brush_segments,
        border_segments,
        border.do_aa,
    );
}

/// Computes the maximum scale that we allow for this set of border parameters.
/// capping the scale will result in rendering very large corners at a lower
/// resolution and stretching them, so they will have the right shape, but
/// blurrier.
pub fn get_max_scale_for_border(
    border_data: &NormalBorderData,
) -> LayoutToDeviceScale {
    let mut r = 1.0;
    for segment in &border_data.border_segments {
        let size = segment.local_task_size;
        r = size.width.max(size.height.max(r));
    }

    LayoutToDeviceScale::new(MAX_BORDER_RESOLUTION as f32 / r)
}

fn add_segment(
    task_rect: DeviceRect,
    style0: BorderStyle,
    style1: BorderStyle,
    color0: ColorF,
    color1: ColorF,
    segment: BorderSegment,
    instances: &mut Vec<BorderInstance>,
    widths: DeviceSize,
    radius: DeviceSize,
    do_aa: bool,
    h_adjacent_corner_outer: DevicePoint,
    h_adjacent_corner_radius: DeviceSize,
    v_adjacent_corner_outer: DevicePoint,
    v_adjacent_corner_radius: DeviceSize,
) {
    let base_flags = (segment as i32) |
                     ((style0 as i32) << 8) |
                     ((style1 as i32) << 16) |
                     ((do_aa as i32) << 28);

    let base_instance = BorderInstance {
        task_origin: DevicePoint::zero(),
        local_rect: task_rect,
        flags: base_flags,
        color0: color0.premultiplied(),
        color1: color1.premultiplied(),
        widths,
        radius,
        clip_params: [0.0; 8],
    };

    match segment {
        BorderSegment::TopLeft |
        BorderSegment::TopRight |
        BorderSegment::BottomLeft |
        BorderSegment::BottomRight => {
            // TODO(gw): Similarly to the old border code, we don't correctly handle a a corner
            //           that is dashed on one edge, and dotted on another. We can handle this
            //           in the future by submitting two instances, each one with one side
            //           color set to have an alpha of 0.
            if (style0 == BorderStyle::Dotted && style1 == BorderStyle::Dashed) ||
               (style0 == BorderStyle::Dashed && style0 == BorderStyle::Dotted) {
                warn!("TODO: Handle a corner with dotted / dashed transition.");
            }

            let dashed_or_dotted_corner = match style0 {
                BorderStyle::Dashed => {
                    write_dashed_corner_instances(
                        radius,
                        widths,
                        segment,
                        &base_instance,
                        instances,
                    )
                }
                BorderStyle::Dotted => {
                    write_dotted_corner_instances(
                        radius,
                        widths,
                        segment,
                        &base_instance,
                        instances,
                    )
                }
                _ => Err(()),
            };

            if dashed_or_dotted_corner.is_err() {
                let clip_params = [
                    h_adjacent_corner_outer.x,
                    h_adjacent_corner_outer.y,
                    h_adjacent_corner_radius.width,
                    h_adjacent_corner_radius.height,
                    v_adjacent_corner_outer.x,
                    v_adjacent_corner_outer.y,
                    v_adjacent_corner_radius.width,
                    v_adjacent_corner_radius.height,
                ];

                instances.push(BorderInstance {
                    clip_params,
                    ..base_instance
                });
            }
        }
        BorderSegment::Top |
        BorderSegment::Bottom |
        BorderSegment::Right |
        BorderSegment::Left => {
            let is_vertical = segment == BorderSegment::Left ||
                              segment == BorderSegment::Right;

            match style0 {
                BorderStyle::Dashed => {
                    let (x, y) = if is_vertical {
                        let half_dash_size = task_rect.size.height * 0.25;
                        (0., half_dash_size)
                    } else {
                        let half_dash_size = task_rect.size.width * 0.25;
                        (half_dash_size, 0.)
                    };

                    instances.push(BorderInstance {
                        flags: base_flags | ((BorderClipKind::DashEdge as i32) << 24),
                        clip_params: [
                            x, y, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                        ],
                        ..base_instance
                    });
                }
                BorderStyle::Dotted => {
                    let (x, y, r) = if is_vertical {
                        (widths.width * 0.5,
                         widths.width,
                         widths.width * 0.5)
                    } else {
                        (widths.height,
                         widths.height * 0.5,
                         widths.height * 0.5)
                    };

                    instances.push(BorderInstance {
                        flags: base_flags | ((BorderClipKind::Dot as i32) << 24),
                        clip_params: [
                            x, y, r, 0.0, 0.0, 0.0, 0.0, 0.0,
                        ],
                        ..base_instance
                    });
                }
                _ => {
                    instances.push(base_instance);
                }
            }
        }
    }
}

/// Add a corner segment (if valid) to the list of
/// border segments for this primitive.
fn add_corner_segment(
    image_rect: LayoutRect,
    non_overlapping_rect: LayoutRect,
    side0: BorderSide,
    side1: BorderSide,
    widths: LayoutSize,
    radius: LayoutSize,
    segment: BorderSegment,
    edge_flags: EdgeAaSegmentMask,
    h_adjacent_corner_outer: LayoutPoint,
    h_adjacent_corner_radius: LayoutSize,
    v_adjacent_corner_outer: LayoutPoint,
    v_adjacent_corner_radius: LayoutSize,
    brush_segments: &mut Vec<BrushSegment>,
    border_segments: &mut Vec<BorderSegmentInfo>,
    do_aa: bool,
) {
    if side0.color.a <= 0.0 && side1.color.a <= 0.0 {
        return;
    }

    if widths.width <= 0.0 && widths.height <= 0.0 {
        return;
    }

    if side0.style.is_hidden() && side1.style.is_hidden() {
        return;
    }

    let segment_rect = match image_rect.intersection(&non_overlapping_rect) {
        Some(rect) => rect,
        None => {
            return;
        }
    };

    let texture_rect = segment_rect
        .translate(-image_rect.origin.to_vector())
        .scale(1.0 / image_rect.size.width, 1.0 / image_rect.size.height);

    brush_segments.push(
        BrushSegment::new(
            segment_rect,
            /* may_need_clip_mask = */ true,
            edge_flags,
            [texture_rect.min_x(), texture_rect.min_y(), texture_rect.max_x(), texture_rect.max_y()],
            BrushFlags::SEGMENT_RELATIVE | BrushFlags::SEGMENT_TEXEL_RECT,
        )
    );

    // If the radii of the adjacent corners do not overlap with this segment,
    // then set the outer position to this segment's corner and the radii to zero.
    // That way the cache key is unaffected by non-overlapping corners, resulting
    // in fewer misses.
    let (h_corner_outer, h_corner_radius) = match segment {
        BorderSegment::TopLeft => {
            if h_adjacent_corner_outer.x - h_adjacent_corner_radius.width < image_rect.max_x() {
                (h_adjacent_corner_outer, h_adjacent_corner_radius)
            } else {
                (LayoutPoint::new(image_rect.max_x(), image_rect.min_y()), LayoutSize::zero())
            }
        }
        BorderSegment::TopRight => {
            if h_adjacent_corner_outer.x + h_adjacent_corner_radius.width > image_rect.min_x() {
                (h_adjacent_corner_outer, h_adjacent_corner_radius)
            } else {
                (LayoutPoint::new(image_rect.min_x(), image_rect.min_y()), LayoutSize::zero())
            }
        }
        BorderSegment::BottomRight => {
            if h_adjacent_corner_outer.x + h_adjacent_corner_radius.width > image_rect.min_x() {
                (h_adjacent_corner_outer, h_adjacent_corner_radius)
            } else {
                (LayoutPoint::new(image_rect.min_x(), image_rect.max_y()), LayoutSize::zero())
            }
        }
        BorderSegment::BottomLeft => {
            if h_adjacent_corner_outer.x - h_adjacent_corner_radius.width < image_rect.max_x() {
                (h_adjacent_corner_outer, h_adjacent_corner_radius)
            } else {
                (image_rect.bottom_right(), LayoutSize::zero())
            }
        }
        _ => unreachable!()
    };

    let (v_corner_outer, v_corner_radius) = match segment {
        BorderSegment::TopLeft => {
            if v_adjacent_corner_outer.y - v_adjacent_corner_radius.height < image_rect.max_y() {
                (v_adjacent_corner_outer, v_adjacent_corner_radius)
            } else {
                (LayoutPoint::new(image_rect.min_x(), image_rect.max_y()), LayoutSize::zero())
            }
        }
        BorderSegment::TopRight => {
            if v_adjacent_corner_outer.y - v_adjacent_corner_radius.height < image_rect.max_y() {
                (v_adjacent_corner_outer, v_adjacent_corner_radius)
            } else {
                (image_rect.bottom_right(), LayoutSize::zero())
            }
        }
        BorderSegment::BottomRight => {
            if v_adjacent_corner_outer.y + v_adjacent_corner_radius.height > image_rect.min_y() {
                (v_adjacent_corner_outer, v_adjacent_corner_radius)
            } else {
                (LayoutPoint::new(image_rect.max_x(), image_rect.min_y()), LayoutSize::zero())
            }
        }
        BorderSegment::BottomLeft => {
            if v_adjacent_corner_outer.y + v_adjacent_corner_radius.height > image_rect.min_y() {
                (v_adjacent_corner_outer, v_adjacent_corner_radius)
            } else {
                (LayoutPoint::new(image_rect.min_x(), image_rect.min_y()), LayoutSize::zero())
            }
        }
        _ => unreachable!()
    };

    border_segments.push(BorderSegmentInfo {
        local_task_size: image_rect.size,
        cache_key: BorderSegmentCacheKey {
            do_aa,
            side0: side0.into(),
            side1: side1.into(),
            segment,
            radius: radius.to_au(),
            size: widths.to_au(),
            h_adjacent_corner_outer: (h_corner_outer - image_rect.origin).to_point().to_au(),
            h_adjacent_corner_radius: h_corner_radius.to_au(),
            v_adjacent_corner_outer: (v_corner_outer - image_rect.origin).to_point().to_au(),
            v_adjacent_corner_radius: v_corner_radius.to_au(),
        },
    });
}

/// Add an edge segment (if valid) to the list of
/// border segments for this primitive.
fn add_edge_segment(
    image_rect: LayoutRect,
    edge_info: &EdgeInfo,
    side: BorderSide,
    width: f32,
    segment: BorderSegment,
    edge_flags: EdgeAaSegmentMask,
    brush_segments: &mut Vec<BrushSegment>,
    border_segments: &mut Vec<BorderSegmentInfo>,
    do_aa: bool,
) {
    if side.color.a <= 0.0 {
        return;
    }

    if side.style.is_hidden() {
        return;
    }

    let (size, brush_flags) = match segment {
        BorderSegment::Left | BorderSegment::Right => {
            (LayoutSize::new(width, edge_info.stretch_size), BrushFlags::SEGMENT_REPEAT_Y)
        }
        BorderSegment::Top | BorderSegment::Bottom => {
            (LayoutSize::new(edge_info.stretch_size, width), BrushFlags::SEGMENT_REPEAT_X)
        }
        _ => {
            unreachable!();
        }
    };

    if image_rect.size.width <= 0. || image_rect.size.height <= 0. {
        return;
    }

    brush_segments.push(
        BrushSegment::new(
            image_rect,
            /* may_need_clip_mask = */ true,
            edge_flags,
            [0.0, 0.0, size.width, size.height],
            BrushFlags::SEGMENT_RELATIVE | brush_flags,
        )
    );

    border_segments.push(BorderSegmentInfo {
        local_task_size: size,
        cache_key: BorderSegmentCacheKey {
            do_aa,
            side0: side.into(),
            side1: side.into(),
            radius: LayoutSizeAu::zero(),
            size: size.to_au(),
            segment,
            h_adjacent_corner_outer: LayoutPointAu::zero(),
            h_adjacent_corner_radius: LayoutSizeAu::zero(),
            v_adjacent_corner_outer: LayoutPointAu::zero(),
            v_adjacent_corner_radius: LayoutSizeAu::zero(),
        },
    });
}

/// Build the set of border instances needed to draw a border
/// segment into the render task cache.
pub fn build_border_instances(
    cache_key: &BorderSegmentCacheKey,
    cache_size: DeviceIntSize,
    border: &ApiNormalBorder,
    scale: LayoutToDeviceScale,
) -> Vec<BorderInstance> {
    let mut instances = Vec::new();

    let (side0, side1, flip0, flip1) = match cache_key.segment {
        BorderSegment::Left => (&border.left, &border.left, false, false),
        BorderSegment::Top => (&border.top, &border.top, false, false),
        BorderSegment::Right => (&border.right, &border.right, true, true),
        BorderSegment::Bottom => (&border.bottom, &border.bottom, true, true),
        BorderSegment::TopLeft => (&border.left, &border.top, false, false),
        BorderSegment::TopRight => (&border.top, &border.right, false, true),
        BorderSegment::BottomRight => (&border.right, &border.bottom, true, true),
        BorderSegment::BottomLeft => (&border.bottom, &border.left, true, false),
    };

    let style0 = if side0.style.is_hidden() {
        side1.style
    } else {
        side0.style
    };
    let style1 = if side1.style.is_hidden() {
        side0.style
    } else {
        side1.style
    };

    let color0 = side0.border_color(flip0);
    let color1 = side1.border_color(flip1);

    let widths = (LayoutSize::from_au(cache_key.size) * scale).ceil();
    let radius = (LayoutSize::from_au(cache_key.radius) * scale).ceil();

    let h_corner_outer = (LayoutPoint::from_au(cache_key.h_adjacent_corner_outer) * scale).round();
    let h_corner_radius = (LayoutSize::from_au(cache_key.h_adjacent_corner_radius) * scale).ceil();
    let v_corner_outer = (LayoutPoint::from_au(cache_key.v_adjacent_corner_outer) * scale).round();
    let v_corner_radius = (LayoutSize::from_au(cache_key.v_adjacent_corner_radius) * scale).ceil();

    add_segment(
        DeviceRect::new(DevicePoint::zero(), cache_size.to_f32()),
        style0,
        style1,
        color0,
        color1,
        cache_key.segment,
        &mut instances,
        widths,
        radius,
        border.do_aa,
        h_corner_outer,
        h_corner_radius,
        v_corner_outer,
        v_corner_radius,
    );

    instances
}

impl NinePatchDescriptor {
    pub fn create_segments(
        &self,
        size: LayoutSize,
    ) -> Vec<BrushSegment> {
        let rect = LayoutRect::new(
            LayoutPoint::zero(),
            size,
        );

        // Calculate the modified rect as specific by border-image-outset
        let origin = LayoutPoint::new(
            rect.origin.x - self.outset.left,
            rect.origin.y - self.outset.top,
        );
        let size = LayoutSize::new(
            rect.size.width + self.outset.left + self.outset.right,
            rect.size.height + self.outset.top + self.outset.bottom,
        );
        let rect = LayoutRect::new(origin, size);

        // Calculate the local texel coords of the slices.
        let px0 = 0.0;
        let px1 = self.slice.left as f32 / self.width as f32;
        let px2 = (self.width as f32 - self.slice.right as f32) / self.width as f32;
        let px3 = 1.0;

        let py0 = 0.0;
        let py1 = self.slice.top as f32 / self.height as f32;
        let py2 = (self.height as f32 - self.slice.bottom as f32) / self.height as f32;
        let py3 = 1.0;

        let tl_outer = LayoutPoint::new(rect.origin.x, rect.origin.y);
        let tl_inner = tl_outer + vec2(self.widths.left, self.widths.top);

        let tr_outer = LayoutPoint::new(rect.origin.x + rect.size.width, rect.origin.y);
        let tr_inner = tr_outer + vec2(-self.widths.right, self.widths.top);

        let bl_outer = LayoutPoint::new(rect.origin.x, rect.origin.y + rect.size.height);
        let bl_inner = bl_outer + vec2(self.widths.left, -self.widths.bottom);

        let br_outer = LayoutPoint::new(
            rect.origin.x + rect.size.width,
            rect.origin.y + rect.size.height,
        );
        let br_inner = br_outer - vec2(self.widths.right, self.widths.bottom);

        fn add_segment(
            segments: &mut Vec<BrushSegment>,
            rect: LayoutRect,
            uv_rect: TexelRect,
            repeat_horizontal: RepeatMode,
            repeat_vertical: RepeatMode,
            extra_flags: BrushFlags,
        ) {
            if uv_rect.uv1.x <= uv_rect.uv0.x || uv_rect.uv1.y <= uv_rect.uv0.y {
                return;
            }

            // Use segment relative interpolation for all
            // instances in this primitive.
            let mut brush_flags =
                BrushFlags::SEGMENT_RELATIVE |
                BrushFlags::SEGMENT_TEXEL_RECT |
                extra_flags;

            // Enable repeat modes on the segment.
            if repeat_horizontal == RepeatMode::Repeat {
                brush_flags |= BrushFlags::SEGMENT_REPEAT_X;
            } else if repeat_horizontal == RepeatMode::Round {
                brush_flags |= BrushFlags::SEGMENT_REPEAT_X | BrushFlags::SEGMENT_REPEAT_X_ROUND;
            }

            if repeat_vertical == RepeatMode::Repeat {
                brush_flags |= BrushFlags::SEGMENT_REPEAT_Y;
            } else if repeat_vertical == RepeatMode::Round {
                brush_flags |= BrushFlags::SEGMENT_REPEAT_Y | BrushFlags::SEGMENT_REPEAT_Y_ROUND;
            }

            let segment = BrushSegment::new(
                rect,
                true,
                EdgeAaSegmentMask::empty(),
                [
                    uv_rect.uv0.x,
                    uv_rect.uv0.y,
                    uv_rect.uv1.x,
                    uv_rect.uv1.y,
                ],
                brush_flags,
            );

            segments.push(segment);
        }

        // Build the list of image segments
        let mut segments = Vec::new();

        // Top left
        add_segment(
            &mut segments,
            LayoutRect::from_floats(tl_outer.x, tl_outer.y, tl_inner.x, tl_inner.y),
            TexelRect::new(px0, py0, px1, py1),
            RepeatMode::Stretch,
            RepeatMode::Stretch,
            BrushFlags::empty(),
        );
        // Top right
        add_segment(
            &mut segments,
            LayoutRect::from_floats(tr_inner.x, tr_outer.y, tr_outer.x, tr_inner.y),
            TexelRect::new(px2, py0, px3, py1),
            RepeatMode::Stretch,
            RepeatMode::Stretch,
            BrushFlags::empty(),
        );
        // Bottom right
        add_segment(
            &mut segments,
            LayoutRect::from_floats(br_inner.x, br_inner.y, br_outer.x, br_outer.y),
            TexelRect::new(px2, py2, px3, py3),
            RepeatMode::Stretch,
            RepeatMode::Stretch,
            BrushFlags::empty(),
        );
        // Bottom left
        add_segment(
            &mut segments,
            LayoutRect::from_floats(bl_outer.x, bl_inner.y, bl_inner.x, bl_outer.y),
            TexelRect::new(px0, py2, px1, py3),
            RepeatMode::Stretch,
            RepeatMode::Stretch,
            BrushFlags::empty(),
        );

        // Center
        if self.fill {
            add_segment(
                &mut segments,
                LayoutRect::from_floats(tl_inner.x, tl_inner.y, tr_inner.x, bl_inner.y),
                TexelRect::new(px1, py1, px2, py2),
                self.repeat_horizontal,
                self.repeat_vertical,
                BrushFlags::SEGMENT_NINEPATCH_MIDDLE,
            );
        }

        // Add edge segments.

        // Top
        add_segment(
            &mut segments,
            LayoutRect::from_floats(tl_inner.x, tl_outer.y, tr_inner.x, tl_inner.y),
            TexelRect::new(px1, py0, px2, py1),
            self.repeat_horizontal,
            RepeatMode::Stretch,
            BrushFlags::empty(),
        );
        // Bottom
        add_segment(
            &mut segments,
            LayoutRect::from_floats(bl_inner.x, bl_inner.y, br_inner.x, bl_outer.y),
            TexelRect::new(px1, py2, px2, py3),
            self.repeat_horizontal,
            RepeatMode::Stretch,
            BrushFlags::empty(),
        );
        // Left
        add_segment(
            &mut segments,
            LayoutRect::from_floats(tl_outer.x, tl_inner.y, tl_inner.x, bl_inner.y),
            TexelRect::new(px0, py1, px1, py2),
            RepeatMode::Stretch,
            self.repeat_vertical,
            BrushFlags::empty(),
        );
        // Right
        add_segment(
            &mut segments,
            LayoutRect::from_floats(tr_inner.x, tr_inner.y, br_outer.x, br_inner.y),
            TexelRect::new(px2, py1, px3, py2),
            RepeatMode::Stretch,
            self.repeat_vertical,
            BrushFlags::empty(),
        );

        segments
    }
}
