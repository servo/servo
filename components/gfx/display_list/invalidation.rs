/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computes the differences between two display lists.

use display_list::{BaseDisplayItem, BorderDisplayItem, BoxShadowDisplayItem, DisplayItem};
use display_list::{DisplayItemIterator, DisplayList, GradientDisplayItem, GradientStop};
use display_list::{ImageDisplayItem, LineDisplayItem, SolidColorDisplayItem, StackingContext};
use display_list::{TextDisplayItem};
use text::TextRun;

use euclid::point::Point2D;
use euclid::rect::Rect;
use fnv::FnvHasher;
use msg::compositor_msg::{LayerId, LayerKind};
use net_traits::image::base::Image;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::collections::hash_state::DefaultState;
use std::collections::linked_list::{self, LinkedList};
use std::sync::Arc;
use util::geometry::{Au, ZERO_RECT};

pub fn compute_invalid_regions(old_stacking_context: &StackingContext,
                               new_stacking_context: &StackingContext,
                               layer_kind: LayerKind)
                               -> HashMap<LayerId, Rect<Au>, DefaultState<FnvHasher>> {
    // TODO(pcwalton): This is a hack to avoid running DLBI on 3d transformed tiles. We should have
    // a better solution than just disabling DLBI here.
    if layer_kind == LayerKind::Layer3D {
        return invalidate_entire_stacking_context(new_stacking_context)
    }

    let mut invalidation_computer = DisplayListInvalidationComputer {
        invalid_rects: HashMap::with_hash_state(Default::default()),
    };
    let position_in_layer = old_stacking_context.overflow.origin -
        old_stacking_context.bounds.origin;
    invalidation_computer.compute_invalid_regions(old_stacking_context,
                                                  new_stacking_context,
                                                  old_stacking_context.layer.as_ref().unwrap().id,
                                                  &position_in_layer);
    invalidation_computer.invalid_rects
}

pub fn invalidate_entire_stacking_context(stacking_context: &StackingContext)
                                          -> HashMap<LayerId, Rect<Au>, DefaultState<FnvHasher>> {
    let mut invalidation_computer = DisplayListInvalidationComputer {
        invalid_rects: HashMap::with_hash_state(Default::default()),
    };
    let position_in_layer = stacking_context.overflow.origin - stacking_context.bounds.origin;
    invalidation_computer.invalidate_entire_stacking_context(
        stacking_context,
        stacking_context.layer.as_ref().unwrap().id,
        &position_in_layer);
    invalidation_computer.invalid_rects
}

struct DisplayListInvalidationComputer {
    invalid_rects: HashMap<LayerId, Rect<Au>, DefaultState<FnvHasher>>,
}

impl DisplayListInvalidationComputer {
    /// This function assumes that the two stacking contexts are positioned at the same position in
    /// the parent layer.
    fn compute_invalid_regions(&mut self,
                               old_stacking_context: &StackingContext,
                               new_stacking_context: &StackingContext,
                               layer_id: LayerId,
                               position_in_layer: &Point2D<Au>) {
        let mut invalid_rect = ZERO_RECT;

        // Compute differences between display items.
        compute_differences(&*old_stacking_context.display_list,
                            &*new_stacking_context.display_list,
                            |result| {
                                if let DisplayItemComparisonResult::Different(display_item) =
                                        result {
                                    display_item.invalidate(&mut invalid_rect);
                                }
                            });

        // Write in the resulting invalid rect.
        self.invalidate_rect(&invalid_rect, layer_id, position_in_layer);

        // Compute differences between child stacking contexts.
        compute_differences(&*old_stacking_context.display_list.children,
                            &*new_stacking_context.display_list.children,
                            |result| {
                                match result {
                                    DisplayItemComparisonResult::Same(old_child, new_child) => {
                                        // This child stacking context is in the same position.
                                        // Perform DLBI on its display list.
                                        let (layer_id_for_child, position_in_layer_for_child) =
                                        layer_id_and_position_in_layer_for_stacking_context_child(
                                                old_child,
                                                layer_id,
                                                position_in_layer);
                                        self.compute_invalid_regions(old_child,
                                                                     new_child,
                                                                     layer_id_for_child,
                                                                     &position_in_layer_for_child);
                                    }
                                    DisplayItemComparisonResult::Different(
                                            child_stacking_context) => {
                                        // This child stacking context is in a different position
                                        // entirely. Invalidate the entire thing.
                                        let (layer_id_for_child, position_in_layer_for_child) =
                                        layer_id_and_position_in_layer_for_stacking_context_child(
                                                child_stacking_context,
                                                layer_id,
                                                position_in_layer);
                                        self.invalidate_entire_stacking_context(
                                            child_stacking_context,
                                            layer_id_for_child,
                                            &position_in_layer_for_child)
                                    }
                                }
                            });
    }

    fn invalidate_entire_stacking_context(&mut self,
                                          stacking_context: &StackingContext,
                                          layer_id: LayerId,
                                          position_in_layer: &Point2D<Au>) {
        self.invalidate_rect(&stacking_context.overflow.translate(&stacking_context.bounds.origin),
                             layer_id,
                             position_in_layer);

        for child_stacking_context in stacking_context.display_list.children.iter() {
            let (layer_id_for_child, position_in_layer_for_child) =
                layer_id_and_position_in_layer_for_stacking_context_child(child_stacking_context,
                                                                          layer_id,
                                                                          position_in_layer);
            self.invalidate_entire_stacking_context(child_stacking_context,
                                                    layer_id_for_child,
                                                    &position_in_layer_for_child);
        }
    }

    fn invalidate_rect(&mut self,
                       invalid_rect: &Rect<Au>,
                       layer_id: LayerId,
                       position_in_layer: &Point2D<Au>) {
        let invalid_rect = invalid_rect.translate(position_in_layer);

        match self.invalid_rects.entry(layer_id) {
            Entry::Vacant(entry) => {
                entry.insert(invalid_rect);
            }
            Entry::Occupied(ref mut entry) => {
                *entry.get_mut() = entry.get().union(&invalid_rect)
            }
        }
    }
}

fn layer_id_and_position_in_layer_for_stacking_context_child(
        child_stacking_context: &StackingContext,
        parent_layer_id: LayerId,
        parent_position_in_layer: &Point2D<Au>)
        -> (LayerId, Point2D<Au>) {
    match child_stacking_context.layer {
        Some(ref layer) => {
            (layer.id,
             child_stacking_context.overflow.origin - child_stacking_context.bounds.origin)
        }
        None => {
            (parent_layer_id,
             *parent_position_in_layer + child_stacking_context.overflow.origin -
             child_stacking_context.bounds.origin)
        }
    }
}

/// A simple O(n) approximation algorithm to compute the differences between two sets of objects.
/// We simply throw out equivalent items from the front and back of each list and invalidate the
/// remainder. The full diff(1) algorithm (longest common subsequence) is O(n^2), which is likely
/// too expensive for lots of display items.
fn compute_differences<'a,T,I,B,F>(old_objects: B, new_objects: B, mut callback: F)
                                   where T: DisplayItemComparison + DisplayItemPrint + 'a,
                                         I: Iterator<Item=&'a T>,
                                         B: Iterable<ConcreteIterator=I> + Copy,
                                         F: FnMut(DisplayItemComparisonResult<&'a T>) {
    // Find the first display item that differs.
    let (mut old_objects_iter, mut new_objects_iter) = (old_objects.iter(), new_objects.iter());
    let (mut old_object_list, mut new_object_list) = (Vec::new(), Vec::new());
    loop {
        let old_object = match old_objects_iter.next() {
            Some(old_object) => old_object,
            None => break,
        };
        let new_object = match new_objects_iter.next() {
            Some(new_object) => new_object,
            None => break,
        };
        if old_object.must_be_identical_to(new_object) {
            callback(DisplayItemComparisonResult::Same(old_object, new_object));
            continue
        }
        old_object_list.push(old_object);
        new_object_list.push(new_object);
        break
    }

    // Gather up the remaining old objects and new objects into a list.
    for old_object in old_objects_iter {
        old_object_list.push(old_object)
    }
    for new_object in new_objects_iter {
        new_object_list.push(new_object)
    }

    // Throw out identical old and new objects from the end of the list.
    loop {
        match (old_object_list.last(), new_object_list.last()) {
            (Some(old_object), Some(new_object)) if
                    old_object.must_be_identical_to(new_object) => {
                callback(DisplayItemComparisonResult::Same(old_object, new_object));
            }
            _ => break,
        }
        old_object_list.pop();
        new_object_list.pop();
    }


    // Invalidate differences.
    for old_object in old_object_list.iter() {
        callback(DisplayItemComparisonResult::Different(old_object));
    }
    for new_object in old_object_list.iter() {
        callback(DisplayItemComparisonResult::Different(new_object));
    }
}

enum DisplayItemComparisonResult<T> {
    Same(T, T),
    Different(T),
}

trait Iterable {
    type ConcreteIterator;

    fn iter(self) -> Self::ConcreteIterator;
}

impl<'a> Iterable for &'a DisplayList {
    type ConcreteIterator = DisplayItemIterator<'a>;

    fn iter(self) -> DisplayItemIterator<'a> {
        self.iter()
    }
}

impl<'a,T> Iterable for &'a LinkedList<T> {
    type ConcreteIterator = linked_list::Iter<'a,T>;

    fn iter(self) -> linked_list::Iter<'a,T> {
        self.iter()
    }
}

/// A trait for marking areas as invalid.
trait DisplayItemInvalidation {
    /// Modifies the given rectangle to include the area that this display item occupies.
    fn invalidate(&self, invalid_rect: &mut Rect<Au>);
}

impl DisplayItemInvalidation for DisplayItem {
    fn invalidate(&self, invalid_rect: &mut Rect<Au>) {
        *invalid_rect = invalid_rect.union(&self.base().bounds)
    }
}

/// A trait for quick display item comparison.
trait DisplayItemComparison {
    /// Returns true if this display item must have precisely the same appearance as the given
    /// display item. This is optimized for speed above all else; the comparison can have false
    /// negatives.
    fn must_be_identical_to(&self, other: &Self) -> bool;
}

impl DisplayItemComparison for DisplayItem {
    fn must_be_identical_to(&self, other: &DisplayItem) -> bool {
        match (self, other) {
            (&DisplayItem::SolidColorClass(ref this_solid_color_display_item),
             &DisplayItem::SolidColorClass(ref other_solid_color_display_item)) => {
                this_solid_color_display_item.must_be_identical_to(other_solid_color_display_item)
            }
            (&DisplayItem::TextClass(ref this_text_display_item),
             &DisplayItem::TextClass(ref other_text_display_item)) => {
                this_text_display_item.must_be_identical_to(other_text_display_item)
            }
            (&DisplayItem::ImageClass(ref this_image_display_item),
             &DisplayItem::ImageClass(ref other_image_display_item)) => {
                this_image_display_item.must_be_identical_to(other_image_display_item)
            }
            (&DisplayItem::BorderClass(ref this_border_display_item),
             &DisplayItem::BorderClass(ref other_border_display_item)) => {
                this_border_display_item.must_be_identical_to(other_border_display_item)
            }
            (&DisplayItem::GradientClass(ref this_gradient_display_item),
             &DisplayItem::GradientClass(ref other_gradient_display_item)) => {
                this_gradient_display_item.must_be_identical_to(other_gradient_display_item)
            }
            (&DisplayItem::LineClass(ref this_line_display_item),
             &DisplayItem::LineClass(ref other_line_display_item)) => {
                this_line_display_item.must_be_identical_to(other_line_display_item)
            }
            (&DisplayItem::BoxShadowClass(ref this_box_shadow_display_item),
             &DisplayItem::BoxShadowClass(ref other_box_shadow_display_item)) => {
                this_box_shadow_display_item.must_be_identical_to(other_box_shadow_display_item)
            }
            _ => false,
        }
    }
}

impl DisplayItemComparison for BaseDisplayItem {
    fn must_be_identical_to(&self, other: &BaseDisplayItem) -> bool {
        self.bounds == other.bounds && self.clip == other.clip
    }
}

impl DisplayItemComparison for SolidColorDisplayItem {
    fn must_be_identical_to(&self, other: &SolidColorDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &SolidColorDisplayItem {
            base: ref this_base,
            color: ref this_color,
        } = self;
        let &SolidColorDisplayItem {
            base: ref other_base,
            color: ref other_color
        } = other;
        this_base.must_be_identical_to(other_base) && this_color == other_color
    }
}

impl DisplayItemComparison for TextDisplayItem {
    fn must_be_identical_to(&self, other: &TextDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &TextDisplayItem {
            base: ref this_base,
            text_run: ref this_text_run,
            range: ref this_range,
            text_color: ref this_text_color,
            baseline_origin: ref this_baseline_origin,
            orientation: ref this_orientation,
            blur_radius: ref this_blur_radius,
        } = self;
        let &TextDisplayItem {
            base: ref other_base,
            text_run: ref other_text_run,
            range: ref other_range,
            text_color: ref other_text_color,
            baseline_origin: ref other_baseline_origin,
            orientation: ref other_orientation,
            blur_radius: ref other_blur_radius,
        } = other;
        this_base.must_be_identical_to(other_base) &&
            (&***this_text_run as *const TextRun) == (&***other_text_run as *const TextRun) &&
            this_range == other_range &&
            this_text_color == other_text_color &&
            this_baseline_origin == other_baseline_origin &&
            this_orientation == other_orientation &&
            this_blur_radius == other_blur_radius
    }
}

impl DisplayItemComparison for ImageDisplayItem {
    fn must_be_identical_to(&self, other: &ImageDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &ImageDisplayItem {
            base: ref this_base,
            image: ref this_image,
            stretch_size: ref this_stretch_size,
            image_rendering: ref this_image_rendering,
        } = self;
        let &ImageDisplayItem {
            base: ref other_base,
            image: ref other_image,
            stretch_size: ref other_stretch_size,
            image_rendering: ref other_image_rendering,
        } = other;
        this_base.must_be_identical_to(other_base) &&
            (&**this_image as *const Image) == (&**other_image as *const Image) &&
            this_stretch_size == other_stretch_size &&
            this_image_rendering == other_image_rendering
    }
}

impl DisplayItemComparison for GradientDisplayItem {
    fn must_be_identical_to(&self, other: &GradientDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &GradientDisplayItem {
            base: ref this_base,
            start_point: ref this_start_point,
            end_point: ref this_end_point,
            stops: ref this_stops,
        } = self;
        let &GradientDisplayItem {
            base: ref other_base,
            start_point: ref other_start_point,
            end_point: ref other_end_point,
            stops: ref other_stops,
        } = other;
        this_base.must_be_identical_to(other_base) &&
            this_start_point == other_start_point &&
            this_end_point == other_end_point &&
            this_stops.len() == other_stops.len() &&
            this_stops.iter().zip(other_stops.iter()).all(|(this_stop, other_stop)| {
                this_stop.must_be_identical_to(other_stop)
            })
    }
}

impl DisplayItemComparison for BorderDisplayItem {
    fn must_be_identical_to(&self, other: &BorderDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &BorderDisplayItem {
            base: ref this_base,
            border_widths: ref this_border_widths,
            color: ref this_color,
            style: ref this_style,
            radius: ref this_radius,
        } = self;
        let &BorderDisplayItem {
            base: ref other_base,
            border_widths: ref other_border_widths,
            color: ref other_color,
            style: ref other_style,
            radius: ref other_radius,
        } = other;
        this_base.must_be_identical_to(other_base) &&
            this_border_widths == other_border_widths &&
            this_color == other_color &&
            this_style == other_style &&
            this_radius == other_radius
    }
}

impl DisplayItemComparison for LineDisplayItem {
    fn must_be_identical_to(&self, other: &LineDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &LineDisplayItem {
            base: ref this_base,
            color: ref this_color,
            style: ref this_style,
        } = self;
        let &LineDisplayItem {
            base: ref other_base,
            color: ref other_color,
            style: ref other_style,
        } = other;
        this_base.must_be_identical_to(other_base) &&
            this_color == other_color &&
            this_style == other_style
    }
}

impl DisplayItemComparison for BoxShadowDisplayItem {
    fn must_be_identical_to(&self, other: &BoxShadowDisplayItem) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        let &BoxShadowDisplayItem {
            base: ref this_base,
            box_bounds: ref this_box_bounds,
            offset: ref this_offset,
            color: ref this_color,
            blur_radius: ref this_blur_radius,
            spread_radius: ref this_spread_radius,
            clip_mode: ref this_clip_mode,
        } = self;
        let &BoxShadowDisplayItem {
            base: ref other_base,
            box_bounds: ref other_box_bounds,
            offset: ref other_offset,
            color: ref other_color,
            blur_radius: ref other_blur_radius,
            spread_radius: ref other_spread_radius,
            clip_mode: ref other_clip_mode,
        } = other;
        this_base.must_be_identical_to(other_base) &&
            this_box_bounds == other_box_bounds &&
            this_offset == other_offset &&
            this_color == other_color &&
            this_blur_radius == other_blur_radius &&
            this_spread_radius == other_spread_radius &&
            this_clip_mode == other_clip_mode
    }
}

impl DisplayItemComparison for GradientStop {
    fn must_be_identical_to(&self, other: &GradientStop) -> bool {
        self.offset == other.offset && self.color == other.color
    }
}

impl DisplayItemComparison for Arc<StackingContext> {
    fn must_be_identical_to(&self, other: &Arc<StackingContext>) -> bool {
        // We use destructuring here so that if any new structure fields are added the compiler
        // will require that we update this method.
        //
        // NB: This one is not actually "must-be-identical-to", but it's OK because we only use
        // differences here to determine which stacking contexts to perform full DLBI on.
        let &StackingContext {
            bounds: ref this_bounds,
            z_index: this_z_index,
            filters: ref this_filters,
            blend_mode: ref this_blend_mode,
            transform: ref this_transform,
            perspective: ref this_perspective,
            establishes_3d_context: ref this_establishes_3d_context,
            display_list: _,
            layer: _,
            overflow: _,
        } = &**self;
        let &StackingContext {
            bounds: ref other_bounds,
            z_index: other_z_index,
            filters: ref other_filters,
            blend_mode: ref other_blend_mode,
            transform: ref other_transform,
            perspective: ref other_perspective,
            establishes_3d_context: ref other_establishes_3d_context,
            display_list: _,
            layer: _,
            overflow: _,
        } = &**other;
        this_bounds == other_bounds &&
            this_z_index == other_z_index &&
            this_filters == other_filters &&
            this_blend_mode == other_blend_mode &&
            this_transform == other_transform &&
            this_perspective == other_perspective &&
            this_establishes_3d_context == other_establishes_3d_context
    }
}

/// A trait for debug printing of display items.
trait DisplayItemPrint {
    fn print(&self, indentation: &str);
}

impl DisplayItemPrint for DisplayItem {
    fn print(&self, indentation: &str) {
        DisplayItem::print(self, indentation)
    }
}

impl DisplayItemPrint for Arc<StackingContext> {
    fn print(&self, indentation: &str) {
        self.display_list.print_items(indentation.to_owned())
    }
}

