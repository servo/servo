/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorF, ColorU, GradientStop, PremultipliedColorF};
use api::units::{LayoutRect, LayoutSize, LayoutVector2D};
use crate::gpu_cache::GpuDataRequest;
use std::hash;

mod linear;
mod radial;
mod conic;

pub use linear::*;
pub use radial::*;
pub use conic::*;

/// A hashable gradient stop that can be used in primitive keys.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Copy, Clone, MallocSizeOf, PartialEq)]
pub struct GradientStopKey {
    pub offset: f32,
    pub color: ColorU,
}

impl GradientStopKey {
    pub fn empty() -> Self {
        GradientStopKey {
            offset: 0.0,
            color: ColorU::new(0, 0, 0, 0),
        }
    }
}

impl Into<GradientStopKey> for GradientStop {
    fn into(self) -> GradientStopKey {
        GradientStopKey {
            offset: self.offset,
            color: self.color.into(),
        }
    }
}

// Convert `stop_keys` into a vector of `GradientStop`s, which is a more
// convenient representation for the current gradient builder. Compute the
// minimum stop alpha along the way.
fn stops_and_min_alpha(stop_keys: &[GradientStopKey]) -> (Vec<GradientStop>, f32) {
    let mut min_alpha: f32 = 1.0;
    let stops = stop_keys.iter().map(|stop_key| {
        let color: ColorF = stop_key.color.into();
        min_alpha = min_alpha.min(color.a);

        GradientStop {
            offset: stop_key.offset,
            color,
        }
    }).collect();

    (stops, min_alpha)
}

impl Eq for GradientStopKey {}

impl hash::Hash for GradientStopKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.offset.to_bits().hash(state);
        self.color.hash(state);
    }
}

// The gradient entry index for the first color stop
pub const GRADIENT_DATA_FIRST_STOP: usize = 0;
// The gradient entry index for the last color stop
pub const GRADIENT_DATA_LAST_STOP: usize = GRADIENT_DATA_SIZE - 1;

// The start of the gradient data table
pub const GRADIENT_DATA_TABLE_BEGIN: usize = GRADIENT_DATA_FIRST_STOP + 1;
// The exclusive bound of the gradient data table
pub const GRADIENT_DATA_TABLE_END: usize = GRADIENT_DATA_LAST_STOP;
// The number of entries in the gradient data table.
pub const GRADIENT_DATA_TABLE_SIZE: usize = 128;

// The number of entries in a gradient data: GRADIENT_DATA_TABLE_SIZE + first stop entry + last stop entry
pub const GRADIENT_DATA_SIZE: usize = GRADIENT_DATA_TABLE_SIZE + 2;

/// An entry in a gradient data table representing a segment of the gradient
/// color space.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct GradientDataEntry {
    start_color: PremultipliedColorF,
    end_step: PremultipliedColorF,
}

impl GradientDataEntry {
    fn white() -> Self {
        Self {
            start_color: PremultipliedColorF::WHITE,
            end_step: PremultipliedColorF::TRANSPARENT,
        }
    }
}

// TODO(gw): Tidy this up to be a free function / module?
struct GradientGpuBlockBuilder {}

impl GradientGpuBlockBuilder {
    /// Generate a color ramp filling the indices in [start_idx, end_idx) and interpolating
    /// from start_color to end_color.
    fn fill_colors(
        start_idx: usize,
        end_idx: usize,
        start_color: &PremultipliedColorF,
        end_color: &PremultipliedColorF,
        entries: &mut [GradientDataEntry; GRADIENT_DATA_SIZE],
        prev_step: &PremultipliedColorF,
    ) -> PremultipliedColorF {
        // Calculate the color difference for individual steps in the ramp.
        let inv_steps = 1.0 / (end_idx - start_idx) as f32;
        let mut step = PremultipliedColorF {
            r: (end_color.r - start_color.r) * inv_steps,
            g: (end_color.g - start_color.g) * inv_steps,
            b: (end_color.b - start_color.b) * inv_steps,
            a: (end_color.a - start_color.a) * inv_steps,
        };
        // As a subtle form of compression, we ensure that the step values for
        // each stop range are the same if and only if they belong to the same
        // stop range. However, if two different stop ranges have the same step,
        // we need to modify the steps so they compare unequally between ranges.
        // This allows to quickly compare if two adjacent stops belong to the
        // same range by comparing their steps.
        if step == *prev_step {
            // Modify the step alpha value as if by nextafter(). The difference
            // here should be so small as to be unnoticeable, but yet allow it
            // to compare differently.
            step.a = f32::from_bits(if step.a == 0.0 { 1 } else { step.a.to_bits() + 1 });
        }

        let mut cur_color = *start_color;

        // Walk the ramp writing start and end colors for each entry.
        for index in start_idx .. end_idx {
            let entry = &mut entries[index];
            entry.start_color = cur_color;
            cur_color.r += step.r;
            cur_color.g += step.g;
            cur_color.b += step.b;
            cur_color.a += step.a;
            entry.end_step = step;
        }

        step
    }

    /// Compute an index into the gradient entry table based on a gradient stop offset. This
    /// function maps offsets from [0, 1] to indices in [GRADIENT_DATA_TABLE_BEGIN, GRADIENT_DATA_TABLE_END].
    #[inline]
    fn get_index(offset: f32) -> usize {
        (offset.max(0.0).min(1.0) * GRADIENT_DATA_TABLE_SIZE as f32 +
            GRADIENT_DATA_TABLE_BEGIN as f32)
            .round() as usize
    }

    // Build the gradient data from the supplied stops, reversing them if necessary.
    fn build(
        reverse_stops: bool,
        request: &mut GpuDataRequest,
        src_stops: &[GradientStop],
    ) {
        // Preconditions (should be ensured by DisplayListBuilder):
        // * we have at least two stops
        // * first stop has offset 0.0
        // * last stop has offset 1.0
        let mut src_stops = src_stops.into_iter();
        let mut cur_color = match src_stops.next() {
            Some(stop) => {
                debug_assert_eq!(stop.offset, 0.0);
                stop.color.premultiplied()
            }
            None => {
                error!("Zero gradient stops found!");
                PremultipliedColorF::BLACK
            }
        };

        // A table of gradient entries, with two colors per entry, that specify the start and end color
        // within the segment of the gradient space represented by that entry. To lookup a gradient result,
        // first the entry index is calculated to determine which two colors to interpolate between, then
        // the offset within that entry bucket is used to interpolate between the two colors in that entry.
        // This layout is motivated by the fact that if one naively tries to store a single color per entry
        // and interpolate directly between entries, then hard stops will become softened because the end
        // color of an entry actually differs from the start color of the next entry, even though they fall
        // at the same edge offset in the gradient space. Instead, the two-color-per-entry layout preserves
        // hard stops, as the end color for a given entry can differ from the start color for the following
        // entry.
        // Colors are stored in RGBA32F format (in the GPU cache). This table requires the gradient color
        // stops to be normalized to the range [0, 1]. The first and last entries hold the first and last
        // color stop colors respectively, while the entries in between hold the interpolated color stop
        // values for the range [0, 1].
        // As a further optimization, rather than directly storing the end color, the difference of the end
        // color from the start color is stored instead, so that an entry can be evaluated more cheaply
        // with start+diff*offset instead of mix(start,end,offset). Further, the color difference in two
        // adjacent entries will always be the same if they were generated from the same set of stops/run.
        // To allow fast searching of the table, if two adjacent entries generated from different sets of
        // stops (a boundary) have the same difference, the floating-point bits of the stop will be nudged
        // so that they compare differently without perceptibly altering the interpolation result. This way,
        // one can quickly scan the table and recover runs just by comparing the color differences of the
        // current and next entry.
        // For example, a table with 2 inside entries (startR,startG,startB):(diffR,diffG,diffB) might look
        // like so:
        //     first           | 0.0              | 0.5              | last
        //     (0,0,0):(0,0,0) | (1,0,0):(-1,1,0) | (0,0,1):(0,1,-1) | (1,1,1):(0,0,0)
        //     ^ solid black     ^ red to green     ^ blue to green    ^ solid white
        let mut entries = [GradientDataEntry::white(); GRADIENT_DATA_SIZE];
        let mut prev_step = cur_color;
        if reverse_stops {
            // Fill in the first entry (for reversed stops) with the first color stop
            prev_step = GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_LAST_STOP,
                GRADIENT_DATA_LAST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
                &prev_step,
            );

            // Fill in the center of the gradient table, generating a color ramp between each consecutive pair
            // of gradient stops. Each iteration of a loop will fill the indices in [next_idx, cur_idx). The
            // loop will then fill indices in [GRADIENT_DATA_TABLE_BEGIN, GRADIENT_DATA_TABLE_END).
            let mut cur_idx = GRADIENT_DATA_TABLE_END;
            for next in src_stops {
                let next_color = next.color.premultiplied();
                let next_idx = Self::get_index(1.0 - next.offset);

                if next_idx < cur_idx {
                    prev_step = GradientGpuBlockBuilder::fill_colors(
                        next_idx,
                        cur_idx,
                        &next_color,
                        &cur_color,
                        &mut entries,
                        &prev_step,
                    );
                    cur_idx = next_idx;
                }

                cur_color = next_color;
            }
            if cur_idx != GRADIENT_DATA_TABLE_BEGIN {
                error!("Gradient stops abruptly at {}, auto-completing to white", cur_idx);
            }

            // Fill in the last entry (for reversed stops) with the last color stop
            GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_FIRST_STOP,
                GRADIENT_DATA_FIRST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
                &prev_step,
            );
        } else {
            // Fill in the first entry with the first color stop
            prev_step = GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_FIRST_STOP,
                GRADIENT_DATA_FIRST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
                &prev_step,
            );

            // Fill in the center of the gradient table, generating a color ramp between each consecutive pair
            // of gradient stops. Each iteration of a loop will fill the indices in [cur_idx, next_idx). The
            // loop will then fill indices in [GRADIENT_DATA_TABLE_BEGIN, GRADIENT_DATA_TABLE_END).
            let mut cur_idx = GRADIENT_DATA_TABLE_BEGIN;
            for next in src_stops {
                let next_color = next.color.premultiplied();
                let next_idx = Self::get_index(next.offset);

                if next_idx > cur_idx {
                    prev_step = GradientGpuBlockBuilder::fill_colors(
                        cur_idx,
                        next_idx,
                        &cur_color,
                        &next_color,
                        &mut entries,
                        &prev_step,
                    );
                    cur_idx = next_idx;
                }

                cur_color = next_color;
            }
            if cur_idx != GRADIENT_DATA_TABLE_END {
                error!("Gradient stops abruptly at {}, auto-completing to white", cur_idx);
            }

            // Fill in the last entry with the last color stop
            GradientGpuBlockBuilder::fill_colors(
                GRADIENT_DATA_LAST_STOP,
                GRADIENT_DATA_LAST_STOP + 1,
                &cur_color,
                &cur_color,
                &mut entries,
                &prev_step,
            );
        }

        for entry in entries.iter() {
            request.push(entry.start_color);
            request.push(entry.end_step);
        }
    }
}

// If the gradient is not tiled we know that any content outside of the clip will not
// be shown. Applying the clip early reduces how much of the gradient we
// render and cache. We do this optimization separately on each axis.
// Returns the offset between the new and old primitive rect origin, to apply to the
// gradient parameters that are relative to the primitive origin.
pub fn apply_gradient_local_clip(
    prim_rect: &mut LayoutRect,
    stretch_size: &LayoutSize,
    tile_spacing: &LayoutSize,
    clip_rect: &LayoutRect,
) -> LayoutVector2D {
    let w = prim_rect.max_x().min(clip_rect.max_x()) - prim_rect.min_x();
    let h = prim_rect.max_y().min(clip_rect.max_y()) - prim_rect.min_y();
    let is_tiled_x = w > stretch_size.width + tile_spacing.width;
    let is_tiled_y = h > stretch_size.height + tile_spacing.height;

    let mut offset = LayoutVector2D::new(0.0, 0.0);

    if !is_tiled_x {
        let diff = (clip_rect.min_x() - prim_rect.min_x()).min(prim_rect.size.width);
        if diff > 0.0 {
            prim_rect.origin.x += diff;
            prim_rect.size.width -= diff;
            offset.x = -diff;
        }

        let diff = prim_rect.max_x() - clip_rect.max_x();
        if diff > 0.0 {
            prim_rect.size.width -= diff;
        }
    }

    if !is_tiled_y {
        let diff = (clip_rect.min_y() - prim_rect.min_y()).min(prim_rect.size.height);
        if diff > 0.0 {
            prim_rect.origin.y += diff;
            prim_rect.size.height -= diff;
            offset.y = -diff;
        }

        let diff = prim_rect.max_y() - clip_rect.max_y();
        if diff > 0.0 {
            prim_rect.size.height -= diff;
        }
    }

    offset
}

#[test]
#[cfg(target_pointer_width = "64")]
fn test_struct_sizes() {
    use std::mem;
    // The sizes of these structures are critical for performance on a number of
    // talos stress tests. If you get a failure here on CI, there's two possibilities:
    // (a) You made a structure smaller than it currently is. Great work! Update the
    //     test expectations and move on.
    // (b) You made a structure larger. This is not necessarily a problem, but should only
    //     be done with care, and after checking if talos performance regresses badly.
    assert_eq!(mem::size_of::<LinearGradient>(), 72, "LinearGradient size changed");
    assert_eq!(mem::size_of::<LinearGradientTemplate>(), 144, "LinearGradientTemplate size changed");
    assert_eq!(mem::size_of::<LinearGradientKey>(), 88, "LinearGradientKey size changed");

    assert_eq!(mem::size_of::<RadialGradient>(), 72, "RadialGradient size changed");
    assert_eq!(mem::size_of::<RadialGradientTemplate>(), 144, "RadialGradientTemplate size changed");
    assert_eq!(mem::size_of::<RadialGradientKey>(), 96, "RadialGradientKey size changed");

    assert_eq!(mem::size_of::<ConicGradient>(), 72, "ConicGradient size changed");
    assert_eq!(mem::size_of::<ConicGradientTemplate>(), 144, "ConicGradientTemplate size changed");
    assert_eq!(mem::size_of::<ConicGradientKey>(), 96, "ConicGradientKey size changed");
}
