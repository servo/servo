/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use app_units::Au;
use embedder_traits::Cursor;
use euclid::{Box2D, Point2D, Vector2D};
use kurbo::{Ellipse, Shape};
use layout_api::{HitTestFlags, HitTestResult, HitTestResultItem};
use rustc_hash::FxHashMap;
use servo_base::id::ScrollTreeNodeId;
use servo_base::text::Utf32CodeUnits;
use servo_geometry::FastLayoutTransform;
use style::computed_values::backface_visibility::T as BackfaceVisibility;
use style::computed_values::pointer_events::T as PointerEvents;
use style::computed_values::visibility::T as Visibility;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::values::computed::ui::CursorKind;
use style_traits::CSSPixel;
use webrender_api::BorderRadius;
use webrender_api::units::{LayoutPoint, LayoutRect, LayoutSize, RectExt};

use crate::display_list::clip::{Clip, ClipId};
use crate::display_list::paint_traversal::{PaintTraversal, PaintTraversalHandler};
use crate::display_list::{StackingContext, StackingContextTree, ToWebRender, TraversalState};
use crate::fragment_tree::{BoxFragmentWithStyle, Fragment, FragmentFlags, TextFragment};
use crate::geom::PhysicalRect;

struct DomPositionCandidate {
    fragment: Fragment,
    node: OpaqueNode,
    point_in_target: Point2D<f32, CSSPixel>,
}

pub(crate) struct HitTest<'a> {
    /// The flags which describe how to perform this [`HitTest`]
    flags: HitTestFlags,
    /// The point to test for this hit test, relative to the page.
    point_to_test: LayoutPoint,
    /// A cached version of [`Self::point_to_test`] projected to a spatial node, to avoid
    /// doing a lot of matrix math over and over.
    projected_point_to_test: Option<(ScrollTreeNodeId, LayoutPoint, FastLayoutTransform)>,
    /// The stacking context tree against which to perform the hit test.
    stacking_context_tree: &'a StackingContextTree,
    /// The resulting [`HitTestResultItems`] for this hit test.
    items: Vec<HitTestResultItem>,
    /// Candidate for `HitTestResult::dom_position_for_selection`
    dom_position_candidate: Option<DomPositionCandidate>,
    /// A cache of hit test results for shared clip nodes.
    clip_hit_test_results: FxHashMap<ClipId, bool>,
    /// Collected reference frame clips. For painting, reference frame clips are handled
    /// by enclosing reference frames in stacking contexts, but we don't have that option
    /// here, so we must handle them manually.
    collected_reference_frame_clips: Vec<ClipId>,
}

impl<'a> HitTest<'a> {
    pub(crate) fn run(
        flags: HitTestFlags,
        stacking_context_tree: &'a StackingContextTree,
        point_to_test: LayoutPoint,
    ) -> HitTestResult {
        let mut hit_test = Self {
            flags,
            point_to_test,
            projected_point_to_test: None,
            stacking_context_tree,
            items: Vec::new(),
            dom_position_candidate: None,
            clip_hit_test_results: FxHashMap::default(),
            collected_reference_frame_clips: Default::default(),
        };

        PaintTraversal::traverse(&stacking_context_tree.root_stacking_context, &mut hit_test);

        // PaintTraversal::traverse walks forward through all fragments via the stacking
        // context tree, so results will be in back-to-front order. We want results to be
        // front-to-back order, so reverse them.
        //
        // TODO: Eventually PaintTraversal should support walking backward through
        // fragments.
        hit_test.items.reverse();

        HitTestResult {
            dom_position_for_selection: hit_test.dom_position(),
            items: hit_test.items,
        }
    }

    fn dom_position(&self) -> Option<(OpaqueNode, Utf32CodeUnits)> {
        let hit = self.dom_position_candidate.as_ref()?;
        if let Fragment::Text(text_fragment) = &hit.fragment {
            let character_offset =
                text_fragment.character_offset(hit.point_in_target.map(Au::from_f32_px))?;
            return Some((hit.node, character_offset));
        }

        let mut search = ClosestFragmentSearch::default();
        if let Some(point_in_fragment) = self.stacking_context_tree.offset_in_fragment(
            &hit.fragment,
            self.point_to_test.map(Au::from_f32_px).cast_unit(),
        ) {
            search.collect_relevant_children(&hit.fragment, point_in_fragment);
        }
        search.into_dom_position()
    }

    /// Perform a hit test against the clip node for the given [`ClipId`], returning
    /// true if it is not clipped out or false if is clipped out.
    fn hit_test_clip_id(&mut self, clip_id: ClipId) -> bool {
        // Using the index here is necessary to avoid a double borrow of `self`.
        for index in 0..self.collected_reference_frame_clips.len() {
            if !self.hit_test_individual_clip_id(self.collected_reference_frame_clips[index]) {
                return false;
            }
        }
        self.hit_test_individual_clip_id(clip_id)
    }

    fn hit_test_individual_clip_id(&mut self, clip_id: ClipId) -> bool {
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
                clip.contains(point) && self.hit_test_individual_clip_id(clip.parent_clip_id)
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
    /// `true` if we pushed a reference frame clip and `false` otherwise.
    type StackingContextState = bool;

    fn visit_stacking_context(
        &mut self,
        stacking_context: &StackingContext,
    ) -> Self::StackingContextState {
        if let Some(reference_frame_info) = stacking_context.reference_frame_info.as_ref() &&
            reference_frame_info.captured_clip_id != ClipId::INVALID
        {
            self.collected_reference_frame_clips
                .push(reference_frame_info.captured_clip_id);
            return true;
        }
        false
    }

    fn leave_stacking_context(
        &mut self,
        _: &TraversalState,
        pushed_reference_frame_clip: Self::StackingContextState,
    ) {
        if pushed_reference_frame_clip {
            self.collected_reference_frame_clips.pop();
        }
    }

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

                hit_test.items.push(HitTestResultItem {
                    node: tag.node,
                    point_in_target,
                    cursor: cursor(style.get_inherited_ui().cursor.keyword, auto_cursor),
                });

                if hit_test.flags.intersects(HitTestFlags::IncludeDomPosition) {
                    hit_test.dom_position_candidate = Some(DomPositionCandidate {
                        fragment: self.clone(),
                        node: tag.node,
                        point_in_target,
                    });
                }

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
                &text.style(),
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

pub(crate) struct ClosestFragment {
    fragment: Arc<TextFragment>,
    node: OpaqueNode,
    point_in_fragment: Point2D<Au, CSSPixel>,
    distance: Au,
    point_in_vertical_bounds: bool,
}

impl ClosestFragment {
    fn should_replace(&self, new_distance: Au, point_in_vertical_bounds: bool) -> bool {
        if point_in_vertical_bounds && !self.point_in_vertical_bounds {
            return true;
        }
        if self.point_in_vertical_bounds && !point_in_vertical_bounds {
            return false;
        }
        new_distance <= self.distance
    }

    pub(crate) fn dom_position(&self) -> Option<(OpaqueNode, Utf32CodeUnits)> {
        let character_offset = self.fragment.character_offset(self.point_in_fragment)?;
        Some((self.node, character_offset))
    }
}

#[derive(Default)]
pub(crate) struct ClosestFragmentSearch {
    closest: Option<ClosestFragment>,
}

impl ClosestFragmentSearch {
    pub(crate) fn into_dom_position(self) -> Option<(OpaqueNode, Utf32CodeUnits)> {
        self.closest?.dom_position()
    }

    fn maybe_update(&mut self, fragment: &Fragment, point_in_fragment: Point2D<Au, CSSPixel>) {
        let Fragment::Text(text_fragment) = fragment else {
            return;
        };

        let (distance, point_in_vertical_bounds) = {
            (
                text_fragment.distance_to_point_for_glyph_offset(point_in_fragment),
                text_fragment.point_is_within_vertical_boundaries(point_in_fragment),
            )
        };

        if let Some(tag) = text_fragment.base.tag.as_ref() &&
            self.closest.as_ref().is_none_or(|closest_fragment| {
                closest_fragment.should_replace(distance, point_in_vertical_bounds)
            })
        {
            self.closest = Some(ClosestFragment {
                fragment: text_fragment.clone(),
                node: tag.node,
                point_in_fragment,
                distance,
                point_in_vertical_bounds,
            });
        }
    }

    pub(crate) fn collect_relevant_children(
        &mut self,
        fragment: &Fragment,
        point_in_fragment: Point2D<Au, CSSPixel>,
    ) {
        self.maybe_update(fragment, point_in_fragment);
        if let Some(children) = fragment.children() {
            for child in children.iter() {
                let offset = child
                    .base()
                    .map(|base| base.rect().origin)
                    .unwrap_or_default();
                let point = point_in_fragment - offset.to_vector();
                self.collect_relevant_children(child, point);
            }
        }
    }
}
