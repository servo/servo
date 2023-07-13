/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{
    ColorU, MixBlendMode, FilterPrimitiveInput, FilterPrimitiveKind, ColorSpace,
    PropertyBinding, PropertyBindingId, CompositeOperator,
};
use api::units::{Au, LayoutVector2D};
use crate::scene_building::IsVisible;
use crate::filterdata::SFilterData;
use crate::intern::ItemUid;
use crate::intern::{Internable, InternDebug, Handle as InternHandle};
use crate::internal_types::{LayoutPrimitiveInfo, Filter};
use crate::picture::PictureCompositeMode;
use crate::prim_store::{
    PrimitiveInstanceKind, PrimitiveStore, VectorKey,
    InternablePrimitive,
};

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf, PartialEq, Hash, Eq)]
pub enum CompositeOperatorKey {
    Over,
    In,
    Out,
    Atop,
    Xor,
    Lighter,
    Arithmetic([Au; 4]),
}

impl From<CompositeOperator> for CompositeOperatorKey {
    fn from(operator: CompositeOperator) -> Self {
        match operator {
            CompositeOperator::Over => CompositeOperatorKey::Over,
            CompositeOperator::In => CompositeOperatorKey::In,
            CompositeOperator::Out => CompositeOperatorKey::Out,
            CompositeOperator::Atop => CompositeOperatorKey::Atop,
            CompositeOperator::Xor => CompositeOperatorKey::Xor,
            CompositeOperator::Lighter => CompositeOperatorKey::Lighter,
            CompositeOperator::Arithmetic(k_vals) => {
                let k_vals = [
                    Au::from_f32_px(k_vals[0]),
                    Au::from_f32_px(k_vals[1]),
                    Au::from_f32_px(k_vals[2]),
                    Au::from_f32_px(k_vals[3]),
                ];
                CompositeOperatorKey::Arithmetic(k_vals)
            }
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf, PartialEq, Hash, Eq)]
pub enum FilterPrimitiveKey {
    Identity(ColorSpace, FilterPrimitiveInput),
    Flood(ColorSpace, ColorU),
    Blend(ColorSpace, MixBlendMode, FilterPrimitiveInput, FilterPrimitiveInput),
    Blur(ColorSpace, Au, Au, FilterPrimitiveInput),
    Opacity(ColorSpace, Au, FilterPrimitiveInput),
    ColorMatrix(ColorSpace, [Au; 20], FilterPrimitiveInput),
    DropShadow(ColorSpace, (VectorKey, Au, ColorU), FilterPrimitiveInput),
    ComponentTransfer(ColorSpace, FilterPrimitiveInput, Vec<SFilterData>),
    Offset(ColorSpace, FilterPrimitiveInput, VectorKey),
    Composite(ColorSpace, FilterPrimitiveInput, FilterPrimitiveInput, CompositeOperatorKey),
}

/// Represents a hashable description of how a picture primitive
/// will be composited into its parent.
#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, MallocSizeOf, PartialEq, Hash, Eq)]
pub enum PictureCompositeKey {
    // No visual compositing effect
    Identity,

    // FilterOp
    Blur(Au, Au),
    Brightness(Au),
    Contrast(Au),
    Grayscale(Au),
    HueRotate(Au),
    Invert(Au),
    Opacity(Au),
    OpacityBinding(PropertyBindingId, Au),
    Saturate(Au),
    Sepia(Au),
    DropShadows(Vec<(VectorKey, Au, ColorU)>),
    ColorMatrix([Au; 20]),
    SrgbToLinear,
    LinearToSrgb,
    ComponentTransfer(ItemUid),
    Flood(ColorU),
    SvgFilter(Vec<FilterPrimitiveKey>),

    // MixBlendMode
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl From<Option<PictureCompositeMode>> for PictureCompositeKey {
    fn from(mode: Option<PictureCompositeMode>) -> Self {
        match mode {
            Some(PictureCompositeMode::MixBlend(mode)) => {
                match mode {
                    MixBlendMode::Normal => PictureCompositeKey::Identity,
                    MixBlendMode::Multiply => PictureCompositeKey::Multiply,
                    MixBlendMode::Screen => PictureCompositeKey::Screen,
                    MixBlendMode::Overlay => PictureCompositeKey::Overlay,
                    MixBlendMode::Darken => PictureCompositeKey::Darken,
                    MixBlendMode::Lighten => PictureCompositeKey::Lighten,
                    MixBlendMode::ColorDodge => PictureCompositeKey::ColorDodge,
                    MixBlendMode::ColorBurn => PictureCompositeKey::ColorBurn,
                    MixBlendMode::HardLight => PictureCompositeKey::HardLight,
                    MixBlendMode::SoftLight => PictureCompositeKey::SoftLight,
                    MixBlendMode::Difference => PictureCompositeKey::Difference,
                    MixBlendMode::Exclusion => PictureCompositeKey::Exclusion,
                    MixBlendMode::Hue => PictureCompositeKey::Hue,
                    MixBlendMode::Saturation => PictureCompositeKey::Saturation,
                    MixBlendMode::Color => PictureCompositeKey::Color,
                    MixBlendMode::Luminosity => PictureCompositeKey::Luminosity,
                }
            }
            Some(PictureCompositeMode::Filter(op)) => {
                match op {
                    Filter::Blur(width, height) =>
                        PictureCompositeKey::Blur(Au::from_f32_px(width), Au::from_f32_px(height)),
                    Filter::Brightness(value) => PictureCompositeKey::Brightness(Au::from_f32_px(value)),
                    Filter::Contrast(value) => PictureCompositeKey::Contrast(Au::from_f32_px(value)),
                    Filter::Grayscale(value) => PictureCompositeKey::Grayscale(Au::from_f32_px(value)),
                    Filter::HueRotate(value) => PictureCompositeKey::HueRotate(Au::from_f32_px(value)),
                    Filter::Invert(value) => PictureCompositeKey::Invert(Au::from_f32_px(value)),
                    Filter::Saturate(value) => PictureCompositeKey::Saturate(Au::from_f32_px(value)),
                    Filter::Sepia(value) => PictureCompositeKey::Sepia(Au::from_f32_px(value)),
                    Filter::SrgbToLinear => PictureCompositeKey::SrgbToLinear,
                    Filter::LinearToSrgb => PictureCompositeKey::LinearToSrgb,
                    Filter::Identity => PictureCompositeKey::Identity,
                    Filter::DropShadows(ref shadows) => {
                        PictureCompositeKey::DropShadows(
                            shadows.iter().map(|shadow| {
                                (shadow.offset.into(), Au::from_f32_px(shadow.blur_radius), shadow.color.into())
                            }).collect()
                        )
                    }
                    Filter::Opacity(binding, _) => {
                        match binding {
                            PropertyBinding::Value(value) => {
                                PictureCompositeKey::Opacity(Au::from_f32_px(value))
                            }
                            PropertyBinding::Binding(key, default) => {
                                PictureCompositeKey::OpacityBinding(key.id, Au::from_f32_px(default))
                            }
                        }
                    }
                    Filter::ColorMatrix(values) => {
                        let mut quantized_values: [Au; 20] = [Au(0); 20];
                        for (value, result) in values.iter().zip(quantized_values.iter_mut()) {
                            *result = Au::from_f32_px(*value);
                        }
                        PictureCompositeKey::ColorMatrix(quantized_values)
                    }
                    Filter::ComponentTransfer => unreachable!(),
                    Filter::Flood(color) => PictureCompositeKey::Flood(color.into()),
                }
            }
            Some(PictureCompositeMode::ComponentTransferFilter(handle)) => {
                PictureCompositeKey::ComponentTransfer(handle.uid())
            }
            Some(PictureCompositeMode::SvgFilter(filter_primitives, filter_data)) => {
                PictureCompositeKey::SvgFilter(filter_primitives.into_iter().map(|primitive| {
                    match primitive.kind {
                        FilterPrimitiveKind::Identity(identity) => FilterPrimitiveKey::Identity(primitive.color_space, identity.input),
                        FilterPrimitiveKind::Blend(blend) => FilterPrimitiveKey::Blend(primitive.color_space, blend.mode, blend.input1, blend.input2),
                        FilterPrimitiveKind::Flood(flood) => FilterPrimitiveKey::Flood(primitive.color_space, flood.color.into()),
                        FilterPrimitiveKind::Blur(blur) =>
                            FilterPrimitiveKey::Blur(primitive.color_space, Au::from_f32_px(blur.width), Au::from_f32_px(blur.height), blur.input),
                        FilterPrimitiveKind::Opacity(opacity) =>
                            FilterPrimitiveKey::Opacity(primitive.color_space, Au::from_f32_px(opacity.opacity), opacity.input),
                        FilterPrimitiveKind::ColorMatrix(color_matrix) => {
                            let mut quantized_values: [Au; 20] = [Au(0); 20];
                            for (value, result) in color_matrix.matrix.iter().zip(quantized_values.iter_mut()) {
                                *result = Au::from_f32_px(*value);
                            }
                            FilterPrimitiveKey::ColorMatrix(primitive.color_space, quantized_values, color_matrix.input)
                        }
                        FilterPrimitiveKind::DropShadow(drop_shadow) => {
                            FilterPrimitiveKey::DropShadow(
                                primitive.color_space,
                                (
                                    drop_shadow.shadow.offset.into(),
                                    Au::from_f32_px(drop_shadow.shadow.blur_radius),
                                    drop_shadow.shadow.color.into(),
                                ),
                                drop_shadow.input,
                            )
                        }
                        FilterPrimitiveKind::ComponentTransfer(component_transfer) =>
                            FilterPrimitiveKey::ComponentTransfer(primitive.color_space, component_transfer.input, filter_data.clone()),
                        FilterPrimitiveKind::Offset(info) =>
                            FilterPrimitiveKey::Offset(primitive.color_space, info.input, info.offset.into()),
                        FilterPrimitiveKind::Composite(info) =>
                            FilterPrimitiveKey::Composite(primitive.color_space, info.input1, info.input2, info.operator.into()),
                    }
                }).collect())
            }
            Some(PictureCompositeMode::Blit(_)) |
            Some(PictureCompositeMode::TileCache { .. }) |
            None => {
                PictureCompositeKey::Identity
            }
        }
    }
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct Picture {
    pub composite_mode_key: PictureCompositeKey,
}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(Debug, Clone, Eq, MallocSizeOf, PartialEq, Hash)]
pub struct PictureKey {
    pub composite_mode_key: PictureCompositeKey,
}

impl PictureKey {
    pub fn new(
        pic: Picture,
    ) -> Self {
        PictureKey {
            composite_mode_key: pic.composite_mode_key,
        }
    }
}

impl InternDebug for PictureKey {}

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct PictureData;

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
#[derive(MallocSizeOf)]
pub struct PictureTemplate;

impl From<PictureKey> for PictureTemplate {
    fn from(_: PictureKey) -> Self {
        PictureTemplate
    }
}

pub type PictureDataHandle = InternHandle<Picture>;

impl Internable for Picture {
    type Key = PictureKey;
    type StoreData = PictureTemplate;
    type InternData = ();
    const PROFILE_COUNTER: usize = crate::profiler::INTERNED_PICTURES;
}

impl InternablePrimitive for Picture {
    fn into_key(
        self,
        _: &LayoutPrimitiveInfo,
    ) -> PictureKey {
        PictureKey::new(self)
    }

    fn make_instance_kind(
        _key: PictureKey,
        _: PictureDataHandle,
        _: &mut PrimitiveStore,
        _reference_frame_relative_offset: LayoutVector2D,
    ) -> PrimitiveInstanceKind {
        // Should never be hit as this method should not be
        // called for pictures.
        unreachable!();
    }
}

impl IsVisible for Picture {
    fn is_visible(&self) -> bool {
        true
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
    assert_eq!(mem::size_of::<Picture>(), 88, "Picture size changed");
    assert_eq!(mem::size_of::<PictureTemplate>(), 0, "PictureTemplate size changed");
    assert_eq!(mem::size_of::<PictureKey>(), 88, "PictureKey size changed");
}
