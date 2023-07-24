/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{
    ColorF, ColorU,
    LineOrientation, LineStyle, PremultipliedColorF, Shadow,
};
use api::units::{Au, LayoutSizeAu, LayoutVector2D};
use crate::scene_building::{CreateShadow, IsVisible};
use crate::frame_builder::{FrameBuildingState};
use crate::gpu_cache::GpuDataRequest;
use crate::intern;
use crate::internal_types::LayoutPrimitiveInfo;
use crate::prim_store::{
    PrimKey, PrimTemplate, PrimTemplateCommonData,
    InternablePrimitive, PrimitiveStore,
};
use crate::prim_store::PrimitiveInstanceKind;

/// Maximum resolution in device pixels at which line decorations are rasterized.
pub const MAX_LINE_DECORATION_RESOLUTION: u32 = 4096;

#[derive(Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct LineDecorationCacheKey {
    pub style: LineStyle,
    pub orientation: LineOrientation,
    pub wavy_line_thickness: Au,
    pub size: LayoutSizeAu,
}

/// Identifying key for a line decoration.
#[derive(Clone, Debug, Hash, MallocSizeOf, PartialEq, Eq)]
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub struct LineDecoration {
    // If the cache_key is Some(..) it is a line decoration
    // that relies on a render task (e.g. wavy). If the
    // cache key is None, it uses a fast path to draw the
    // line decoration as a solid rect.
    pub cache_key: Option<LineDecorationCacheKey>,
    pub color: ColorU,
}

pub type LineDecorationKey = PrimKey<LineDecoration>;

impl LineDecorationKey {
    pub fn new(
        info: &LayoutPrimitiveInfo,
        line_dec: LineDecoration,
    ) -> Self {
        LineDecorationKey {
            common: info.into(),
            kind: line_dec,
        }
    }
}

impl intern::InternDebug for LineDecorationKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct LineDecorationData {
    pub cache_key: Option<LineDecorationCacheKey>,
    pub color: ColorF,
}

impl LineDecorationData {
    /// Update the GPU cache for a given primitive template. This may be called multiple
    /// times per frame, by each primitive reference that refers to this interned
    /// template. The initial request call to the GPU cache ensures that work is only
    /// done if the cache entry is invalid (due to first use or eviction).
    pub fn update(
        &mut self,
        common: &mut PrimTemplateCommonData,
        frame_state: &mut FrameBuildingState,
    ) {
        if let Some(ref mut request) = frame_state.gpu_cache.request(&mut common.gpu_cache_handle) {
            self.write_prim_gpu_blocks(request);
        }
    }

    fn write_prim_gpu_blocks(
        &self,
        request: &mut GpuDataRequest
    ) {
        match self.cache_key.as_ref() {
            Some(cache_key) => {
                request.push(self.color.premultiplied());
                request.push(PremultipliedColorF::WHITE);
                request.push([
                    cache_key.size.width.to_f32_px(),
                    cache_key.size.height.to_f32_px(),
                    0.0,
                    0.0,
                ]);
            }
            None => {
                request.push(self.color.premultiplied());
            }
        }
    }
}

pub type LineDecorationTemplate = PrimTemplate<LineDecorationData>;

impl From<LineDecorationKey> for LineDecorationTemplate {
    fn from(line_dec: LineDecorationKey) -> Self {
        let common = PrimTemplateCommonData::with_key_common(line_dec.common);
        LineDecorationTemplate {
            common,
            kind: LineDecorationData {
                cache_key: line_dec.kind.cache_key,
                color: line_dec.kind.color.into(),
            }
        }
    }
}

pub type LineDecorationDataHandle = intern::Handle<LineDecoration>;

impl intern::Internable for LineDecoration {
    type Key = LineDecorationKey;
    type StoreData = LineDecorationTemplate;
    type InternData = ();
}

impl InternablePrimitive for LineDecoration {
    fn into_key(
        self,
        info: &LayoutPrimitiveInfo,
    ) -> LineDecorationKey {
        LineDecorationKey::new(
            info,
            self,
        )
    }

    fn make_instance_kind(
        _key: LineDecorationKey,
        data_handle: LineDecorationDataHandle,
        _: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        PrimitiveInstanceKind::LineDecoration {
            data_handle,
            cache_handle: None,
        }
    }
}

impl CreateShadow for LineDecoration {
    fn create_shadow(&self, shadow: &Shadow) -> Self {
        LineDecoration {
            color: shadow.color.into(),
            cache_key: self.cache_key.clone(),
        }
    }
}

impl IsVisible for LineDecoration {
    fn is_visible(&self) -> bool {
        self.color.a > 0
    }
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
    assert_eq!(mem::size_of::<LineDecoration>(), 20, "LineDecoration size changed");
    assert_eq!(mem::size_of::<LineDecorationTemplate>(), 60, "LineDecorationTemplate size changed");
    assert_eq!(mem::size_of::<LineDecorationKey>(), 40, "LineDecorationKey size changed");
}
