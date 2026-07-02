/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use app_units::Au;
use embedder_traits::Cursor;
use euclid::{Box2D, Vector2D};
use kurbo::{Ellipse, Shape};
use layout_api::ElementsFromPointResult;
use rustc_hash::FxHashMap;
use servo_base::id::ScrollTreeNodeId;
use servo_geometry::FastLayoutTransform;
use style::computed_values::backface_visibility::T as BackfaceVisibility;
use style::computed_values::pointer_events::T as PointerEvents;
use style::computed_values::visibility::T as Visibility;
use style::properties::ComputedValues;
use style::values::computed::ui::CursorKind;
use webrender_api::BorderRadius;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutSize, RectExt};

use crate::display_list::clip::{Clip, ClipId};
use crate::display_list::paint_traversal::{PaintTraversal, PaintTraversalHandler};
use crate::display_list::{StackingContext, StackingContextTree, ToWebRender, TraversalState};
use crate::fragment_tree::{BoxFragmentWithStyle, Fragment, FragmentFlags, TextFragment};
use crate::geom::PhysicalRect;

pub(crate) struct HitTest<'a> {
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
    ) -> Vec<ElementsFromPointResult> {
        let mut hit_test = Self {
            point_to_test,
            projected_point_to_test: None,
            stacking_context_tree,
            results: Vec::new(),
            clip_hit_test_results: FxHashMap::default(),
        };

        PaintTraversal::traverse(&stacking_context_tree.root_stacking_context, &mut hit_test);

        // PaintTraversal::traverse walks forward through all fragments via the stacking
        // context tree, so results will be in back-to-front order. We want results to be
        // front-to-back order, so reverse them.
        //
        // TODO: Eventually PaintTraversal should support walking backward through
        // fragments.
        hit_test.results.reverse();

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
            .paint_info
            .scroll_tree
            .cumulative_root_to_node_transform(scroll_tree_node_id)?;

        let projected_point = transform.project_point2d(self.point_to_test)?;

        self.projected_point_to_test = Some((scroll_tree_node_id, projected_point, transform));
        Some((projected_point, transform))
    }
}

impl PaintTraversalHandler for HitTest<'_> {
    type StackingContextState = ();

    fn visit_stacking_context(&mut self, _: &StackingContext) -> Self::StackingContextState {}
    fn leave_stacking_context(&mut self, _: &TraversalState, _: Self::StackingContextState) {}
    fn visit_box(&mut self, state: &TraversalState, fragment: &BoxFragmentWithStyle<'_>) {
        Fragment::Box(fragment.box_fragment.clone()).hit_test(state, self);
    }
    fn visit_text(
        &mut self,
        state: &TraversalState,
        _: PhysicalRect<Au>,
        fragment: &Arc<TextFragment>,
    ) {
        Fragment::Text(fragment.clone()).hit_test(state, self);
    }
}

impl Clip {
    fn contains(&self, point: LayoutPoint) -> bool {
        rounded_rect_contains_point(self.rect, &self.radii, point)
    }
}

impl Fragment {
    pub(crate) fn hit_test(&self, state: &TraversalState, hit_test: &mut HitTest) -> bool {
        let Some(tag) = self.tag() else {
            return false;
        };
        if !hit_test.hit_test_clip_id(state.clip_id) {
            return false;
        }

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
                    match hit_test.location_in_spatial_node(state.spatial_id) {
                        Some(point) => point,
                        None => return false,
                    };

                if !is_root_element &&
                    style.get_box().backface_visibility == BackfaceVisibility::Hidden &&
                    transform.is_backface_visible()
                {
                    return false;
                }

                let fragment_rect = fragment_rect.translate(state.origin.to_vector());
                if is_root_element {
                    let viewport_size = hit_test
                        .stacking_context_tree
                        .paint_info
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

                // Since there is no reverse PaintTraversal, hit testing always searches
                // the entire fragment tree (in stacking context order), which is why this
                // is always returning `false` (keep looking). Once PaintTraversal can
                // walk backward through fragments, this can return `true` if FindAll
                // isn't specified.
                false
            };

        match self {
            Fragment::LayoutRoot(layout_root_fragment) => {
                layout_root_fragment.inner().hit_test(state, hit_test)
            },
            Fragment::Box(box_fragment) | Fragment::Float(box_fragment) => hit_test_fragment_inner(
                &box_fragment.style(),
                box_fragment.border_rect(),
                box_fragment.border_radius(),
                box_fragment.base.flags,
                Cursor::Default,
            ),
            Fragment::Text(text) => hit_test_fragment_inner(
                &text.base.style(),
                text.base.rect(),
                BorderRadius::zero(),
                FragmentFlags::empty(),
                Cursor::Text,
            ),
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
