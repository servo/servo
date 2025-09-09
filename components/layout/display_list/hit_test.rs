/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use base::id::ScrollTreeNodeId;
use embedder_traits::Cursor;
use euclid::{Box2D, Vector2D};
use kurbo::{Ellipse, Shape};
use layout_api::{ElementsFromPointFlags, ElementsFromPointResult};
use rustc_hash::FxHashMap;
use servo_geometry::FastLayoutTransform;
use style::computed_values::backface_visibility::T as BackfaceVisibility;
use style::computed_values::pointer_events::T as PointerEvents;
use style::computed_values::visibility::T as Visibility;
use style::properties::ComputedValues;
use style::values::computed::ui::CursorKind;
use webrender_api::BorderRadius;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutSize, RectExt};

use crate::display_list::clip::{Clip, ClipId};
use crate::display_list::stacking_context::StackingContextSection;
use crate::display_list::{
    StackingContext, StackingContextContent, StackingContextTree, ToWebRender,
};
use crate::fragment_tree::{Fragment, FragmentFlags};
use crate::geom::PhysicalRect;

pub(crate) struct HitTest<'a> {
    /// The flags which describe how to perform this [`HitTest`].
    flags: ElementsFromPointFlags,
    /// The point to test for this hit test, relative to the page.
    point_to_test: LayoutPoint,
    /// A cached version of [`Self::point_to_test`] projected to a spatial node, to avoid
    /// doing a lot of matrix math over and over.
    projected_point_to_test: Option<(ScrollTreeNodeId, LayoutPoint, FastLayoutTransform)>,
    /// The stacking context tree against which to perform the hit test.
    stacking_context_tree: &'a StackingContextTree,
    /// The resulting [`HitTestResultItems`] for this hit test.
    results: Vec<ElementsFromPointResult>,
    /// A cache of hit test results for shared clip nodes.
    clip_hit_test_results: FxHashMap<ClipId, bool>,
}

impl<'a> HitTest<'a> {
    pub(crate) fn run(
        stacking_context_tree: &'a StackingContextTree,
        point_to_test: LayoutPoint,
        flags: ElementsFromPointFlags,
    ) -> Vec<ElementsFromPointResult> {
        let mut hit_test = Self {
            flags,
            point_to_test,
            projected_point_to_test: None,
            stacking_context_tree,
            results: Vec::new(),
            clip_hit_test_results: FxHashMap::default(),
        };
        stacking_context_tree
            .root_stacking_context
            .hit_test(&mut hit_test);
        hit_test.results
    }

    /// Perform a hit test against a the clip node for the given [`ClipId`], returning
    /// true if it is not clipped out or false if is clipped out.
    fn hit_test_clip_id(&mut self, clip_id: ClipId) -> bool {
        if clip_id == ClipId::INVALID {
            return true;
        }

        if let Some(result) = self.clip_hit_test_results.get(&clip_id) {
            return *result;
        }

        let clip = self.stacking_context_tree.clip_store.get(clip_id);
        let result = self
            .location_in_spatial_node(clip.parent_scroll_node_id)
            .is_some_and(|(point, _)| {
                clip.contains(point) && self.hit_test_clip_id(clip.parent_clip_id)
            });
        self.clip_hit_test_results.insert(clip_id, result);
        result
    }

    /// Get the hit test location in the coordinate system of the given spatial node,
    /// returning `None` if the transformation is uninvertible or the point cannot be
    /// projected into the spatial node.
    fn location_in_spatial_node(
        &mut self,
        scroll_tree_node_id: ScrollTreeNodeId,
    ) -> Option<(LayoutPoint, FastLayoutTransform)> {
        match self.projected_point_to_test {
            Some((cached_scroll_tree_node_id, projected_point, transform))
                if cached_scroll_tree_node_id == scroll_tree_node_id =>
            {
                return Some((projected_point, transform));
            },
            _ => {},
        }

        let transform = self
            .stacking_context_tree
            .compositor_info
            .scroll_tree
            .cumulative_root_to_node_transform(&scroll_tree_node_id)?;

        let projected_point = transform.project_point2d(self.point_to_test)?;

        self.projected_point_to_test = Some((scroll_tree_node_id, projected_point, transform));
        Some((projected_point, transform))
    }
}

impl Clip {
    fn contains(&self, point: LayoutPoint) -> bool {
        rounded_rect_contains_point(self.rect, &self.radii, point)
    }
}

impl StackingContext {
    /// Perform a hit test against a [`StackingContext`]. Note that this is the reverse
    /// of the stacking context walk algorithm in `stacking_context.rs`. Any changes made
    /// here should be reflected in the forward version in that file.
    fn hit_test(&self, hit_test: &mut HitTest) -> bool {
        let mut contents = self.contents.iter().rev().peekable();

        // Step 10: Outlines
        while contents
            .peek()
            .is_some_and(|child| child.section() == StackingContextSection::Outline)
        {
            // The hit test will not hit the outline.
            let _ = contents.next().unwrap();
        }

        // Steps 8 and 9: Stacking contexts with non-negative ‘z-index’, and
        // positioned stacking containers (where ‘z-index’ is auto)
        let mut real_stacking_contexts_and_positioned_stacking_containers = self
            .real_stacking_contexts_and_positioned_stacking_containers
            .iter()
            .rev()
            .peekable();
        while real_stacking_contexts_and_positioned_stacking_containers
            .peek()
            .is_some_and(|child| child.z_index() >= 0)
        {
            let child = real_stacking_contexts_and_positioned_stacking_containers
                .next()
                .unwrap();
            if child.hit_test(hit_test) {
                return true;
            }
        }

        // Steps 7 and 8: Fragments and inline stacking containers
        while contents
            .peek()
            .is_some_and(|child| child.section() == StackingContextSection::Foreground)
        {
            let child = contents.next().unwrap();
            if self.hit_test_content(child, hit_test) {
                return true;
            }
        }

        // Step 6: Float stacking containers
        for child in self.float_stacking_containers.iter().rev() {
            if child.hit_test(hit_test) {
                return true;
            }
        }

        // Step 5: Block backgrounds and borders
        while contents.peek().is_some_and(|child| {
            child.section() == StackingContextSection::DescendantBackgroundsAndBorders
        }) {
            let child = contents.next().unwrap();
            if self.hit_test_content(child, hit_test) {
                return true;
            }
        }

        // Step 4: Stacking contexts with negative ‘z-index’
        for child in real_stacking_contexts_and_positioned_stacking_containers {
            if child.hit_test(hit_test) {
                return true;
            }
        }

        // Steps 2 and 3: Borders and background for the root
        while contents.peek().is_some_and(|child| {
            child.section() == StackingContextSection::OwnBackgroundsAndBorders
        }) {
            let child = contents.next().unwrap();
            if self.hit_test_content(child, hit_test) {
                return true;
            }
        }
        false
    }

    pub(crate) fn hit_test_content(
        &self,
        content: &StackingContextContent,
        hit_test: &mut HitTest<'_>,
    ) -> bool {
        match content {
            StackingContextContent::Fragment {
                scroll_node_id,
                clip_id,
                containing_block,
                fragment,
                ..
            } => {
                hit_test.hit_test_clip_id(*clip_id) &&
                    fragment.hit_test(hit_test, *scroll_node_id, containing_block)
            },
            StackingContextContent::AtomicInlineStackingContainer { index } => {
                self.atomic_inline_stacking_containers[*index].hit_test(hit_test)
            },
        }
    }
}

impl Fragment {
    pub(crate) fn hit_test(
        &self,
        hit_test: &mut HitTest,
        spatial_node_id: ScrollTreeNodeId,
        containing_block: &PhysicalRect<Au>,
    ) -> bool {
        let Some(tag) = self.tag() else {
            return false;
        };

        let mut hit_test_fragment_inner =
            |style: &ComputedValues,
             fragment_rect: PhysicalRect<Au>,
             border_radius: BorderRadius,
             fragment_flags: FragmentFlags,
             auto_cursor: Cursor| {
                let is_root_element = fragment_flags.contains(FragmentFlags::IS_ROOT_ELEMENT);

                if !is_root_element {
                    if style.get_inherited_ui().pointer_events == PointerEvents::None {
                        return false;
                    }
                    if style.get_inherited_box().visibility != Visibility::Visible {
                        return false;
                    }
                }

                let (point_in_spatial_node, transform) =
                    match hit_test.location_in_spatial_node(spatial_node_id) {
                        Some(point) => point,
                        None => return false,
                    };

                if !is_root_element &&
                    style.get_box().backface_visibility == BackfaceVisibility::Hidden &&
                    transform.is_backface_visible()
                {
                    return false;
                }

                let fragment_rect = fragment_rect.translate(containing_block.origin.to_vector());
                if is_root_element {
                    let viewport_size = hit_test
                        .stacking_context_tree
                        .compositor_info
                        .viewport_details
                        .size;
                    let viewport_rect = LayoutRect::from_origin_and_size(
                        Default::default(),
                        viewport_size.cast_unit(),
                    );
                    if !viewport_rect.contains(hit_test.point_to_test) {
                        return false;
                    }
                } else if !rounded_rect_contains_point(
                    fragment_rect.to_webrender(),
                    &border_radius,
                    point_in_spatial_node,
                ) {
                    return false;
                }

                let point_in_target = point_in_spatial_node.cast_unit() -
                    Vector2D::new(
                        fragment_rect.origin.x.to_f32_px(),
                        fragment_rect.origin.y.to_f32_px(),
                    );

                hit_test.results.push(ElementsFromPointResult {
                    node: tag.node,
                    point_in_target,
                    cursor: cursor(style.get_inherited_ui().cursor.keyword, auto_cursor),
                });
                !hit_test.flags.contains(ElementsFromPointFlags::FindAll)
            };

        match self {
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => {
                let box_fragment = box_fragment.borrow();
                hit_test_fragment_inner(
                    &box_fragment.style,
                    box_fragment.border_rect(),
                    box_fragment.border_radius(),
                    box_fragment.base.flags,
                    Cursor::Default,
                )
            },
            Fragment::Text(text) => {
                let text = &*text.borrow();
                hit_test_fragment_inner(
                    &text.inline_styles.style.borrow(),
                    text.rect,
                    BorderRadius::zero(),
                    FragmentFlags::empty(),
                    Cursor::Text,
                )
            },
            _ => false,
        }
    }
}

fn rounded_rect_contains_point(
    rect: LayoutRect,
    border_radius: &BorderRadius,
    point: LayoutPoint,
) -> bool {
    if !rect.contains(point) {
        return false;
    }

    if border_radius.is_zero() {
        return true;
    }

    let check_corner = |corner: LayoutPoint, radius: &LayoutSize, is_right, is_bottom| {
        let mut origin = corner;
        if is_right {
            origin.x -= radius.width;
        }
        if is_bottom {
            origin.y -= radius.height;
        }
        if !Box2D::from_origin_and_size(origin, *radius).contains(point) {
            return true;
        }
        let center = (
            if is_right {
                corner.x - radius.width
            } else {
                corner.x + radius.width
            },
            if is_bottom {
                corner.y - radius.height
            } else {
                corner.y + radius.height
            },
        );
        let radius = (radius.width as f64, radius.height as f64);
        Ellipse::new(center, radius, 0.0).contains((point.x, point.y).into())
    };

    check_corner(rect.top_left(), &border_radius.top_left, false, false) &&
        check_corner(rect.top_right(), &border_radius.top_right, true, false) &&
        check_corner(rect.bottom_right(), &border_radius.bottom_right, true, true) &&
        check_corner(rect.bottom_left(), &border_radius.bottom_left, false, true)
}

fn cursor(kind: CursorKind, auto_cursor: Cursor) -> Cursor {
    match kind {
        CursorKind::Auto => auto_cursor,
        CursorKind::None => Cursor::None,
        CursorKind::Default => Cursor::Default,
        CursorKind::Pointer => Cursor::Pointer,
        CursorKind::ContextMenu => Cursor::ContextMenu,
        CursorKind::Help => Cursor::Help,
        CursorKind::Progress => Cursor::Progress,
        CursorKind::Wait => Cursor::Wait,
        CursorKind::Cell => Cursor::Cell,
        CursorKind::Crosshair => Cursor::Crosshair,
        CursorKind::Text => Cursor::Text,
        CursorKind::VerticalText => Cursor::VerticalText,
        CursorKind::Alias => Cursor::Alias,
        CursorKind::Copy => Cursor::Copy,
        CursorKind::Move => Cursor::Move,
        CursorKind::NoDrop => Cursor::NoDrop,
        CursorKind::NotAllowed => Cursor::NotAllowed,
        CursorKind::Grab => Cursor::Grab,
        CursorKind::Grabbing => Cursor::Grabbing,
        CursorKind::EResize => Cursor::EResize,
        CursorKind::NResize => Cursor::NResize,
        CursorKind::NeResize => Cursor::NeResize,
        CursorKind::NwResize => Cursor::NwResize,
        CursorKind::SResize => Cursor::SResize,
        CursorKind::SeResize => Cursor::SeResize,
        CursorKind::SwResize => Cursor::SwResize,
        CursorKind::WResize => Cursor::WResize,
        CursorKind::EwResize => Cursor::EwResize,
        CursorKind::NsResize => Cursor::NsResize,
        CursorKind::NeswResize => Cursor::NeswResize,
        CursorKind::NwseResize => Cursor::NwseResize,
        CursorKind::ColResize => Cursor::ColResize,
        CursorKind::RowResize => Cursor::RowResize,
        CursorKind::AllScroll => Cursor::AllScroll,
        CursorKind::ZoomIn => Cursor::ZoomIn,
        CursorKind::ZoomOut => Cursor::ZoomOut,
    }
}
