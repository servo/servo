/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{FontInstanceFlags, FontKey, FontRenderMode, FontVariation};
use api::{ColorU, GlyphDimensions, NativeFontHandle};
use dwrote;
use crate::gamma_lut::ColorLut;
use crate::glyph_rasterizer::{FontInstance, FontTransform, GlyphKey};
use crate::internal_types::{FastHashMap, FastHashSet, ResourceCacheError};
use crate::glyph_rasterizer::{GlyphFormat, GlyphRasterError, GlyphRasterResult, RasterizedGlyph};
use crate::gamma_lut::GammaLut;
use std::borrow::Borrow;
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Arc, Mutex};
use api::FontInstancePlatformOptions;
use std::mem;

lazy_static! {
    static ref DEFAULT_FONT_DESCRIPTOR: dwrote::FontDescriptor = dwrote::FontDescriptor {
        family_name: "Arial".to_owned(),
        weight: dwrote::FontWeight::Regular,
        stretch: dwrote::FontStretch::Normal,
        style: dwrote::FontStyle::Normal,
    };
}

type CachedFontKey = Arc<Path>;

// A cached dwrote font file that is shared among all faces.
// Each face holds a CachedFontKey to keep track of how many users of the font there are.
struct CachedFont {
    key: CachedFontKey,
    file: dwrote::FontFile,
}

// FontFile contains a ComPtr<IDWriteFontFile>, but DWrite font files are threadsafe.
unsafe impl Send for CachedFont {}

impl PartialEq for CachedFont {
    fn eq(&self, other: &CachedFont) -> bool {
        self.key == other.key
    }
}
impl Eq for CachedFont {}

impl Hash for CachedFont {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl Borrow<Path> for CachedFont {
    fn borrow(&self) -> &Path {
        &*self.key
    }
}

lazy_static! {
    // This is effectively a weak map of dwrote FontFiles. CachedFonts are entered into the
    // cache when there are any FontFaces using them. CachedFonts are removed from the cache
    // when there are no more FontFaces using them at all.
    static ref FONT_CACHE: Mutex<FastHashSet<CachedFont>> = Mutex::new(FastHashSet::default());
}

struct FontFace {
    cached: Option<CachedFontKey>,
    file: dwrote::FontFile,
    index: u32,
    face: dwrote::FontFace,
}

pub struct FontContext {
    fonts: FastHashMap<FontKey, FontFace>,
    variations: FastHashMap<(FontKey, dwrote::DWRITE_FONT_SIMULATIONS, Vec<FontVariation>), dwrote::FontFace>,
    gamma_luts: FastHashMap<(u16, u8), GammaLut>,
}

// DirectWrite is safe to use on multiple threads and non-shareable resources are
// all hidden inside their font context.
unsafe impl Send for FontContext {}

fn dwrite_texture_type(render_mode: FontRenderMode) -> dwrote::DWRITE_TEXTURE_TYPE {
    match render_mode {
        FontRenderMode::Mono => dwrote::DWRITE_TEXTURE_ALIASED_1x1,
        FontRenderMode::Alpha |
        FontRenderMode::Subpixel => dwrote::DWRITE_TEXTURE_CLEARTYPE_3x1,
    }
}

fn dwrite_measure_mode(
    font: &FontInstance,
    bitmaps: bool,
) -> dwrote::DWRITE_MEASURING_MODE {
    if bitmaps || font.flags.contains(FontInstanceFlags::FORCE_GDI) {
        dwrote::DWRITE_MEASURING_MODE_GDI_CLASSIC
    } else {
      match font.render_mode {
          FontRenderMode::Mono => dwrote::DWRITE_MEASURING_MODE_GDI_CLASSIC,
          FontRenderMode::Alpha | FontRenderMode::Subpixel => dwrote::DWRITE_MEASURING_MODE_NATURAL,
      }
    }
}

fn dwrite_render_mode(
    font_face: &dwrote::FontFace,
    font: &FontInstance,
    em_size: f32,
    measure_mode: dwrote::DWRITE_MEASURING_MODE,
    bitmaps: bool,
) -> dwrote::DWRITE_RENDERING_MODE {
    let dwrite_render_mode = match font.render_mode {
        FontRenderMode::Mono => dwrote::DWRITE_RENDERING_MODE_ALIASED,
        FontRenderMode::Alpha | FontRenderMode::Subpixel => {
            if bitmaps || font.flags.contains(FontInstanceFlags::FORCE_GDI) {
                dwrote::DWRITE_RENDERING_MODE_GDI_CLASSIC
            } else if font.flags.contains(FontInstanceFlags::FORCE_SYMMETRIC) {
                dwrote::DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL_SYMMETRIC
            } else if font.flags.contains(FontInstanceFlags::NO_SYMMETRIC) {
                dwrote::DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL
            } else {
                font_face.get_recommended_rendering_mode_default_params(em_size, 1.0, measure_mode)
            }
        }
    };

    if dwrite_render_mode == dwrote::DWRITE_RENDERING_MODE_OUTLINE {
        // Outline mode is not supported
        return dwrote::DWRITE_RENDERING_MODE_CLEARTYPE_NATURAL_SYMMETRIC;
    }

    dwrite_render_mode
}

fn is_bitmap_font(font: &FontInstance) -> bool {
    // If bitmaps are requested, then treat as a bitmap font to disable transforms.
    // If mono AA is requested, let that take priority over using bitmaps.
    font.render_mode != FontRenderMode::Mono &&
        font.flags.contains(FontInstanceFlags::EMBEDDED_BITMAPS)
}

impl FontContext {
    pub fn new() -> Result<FontContext, ResourceCacheError> {
        Ok(FontContext {
            fonts: FastHashMap::default(),
            variations: FastHashMap::default(),
            gamma_luts: FastHashMap::default(),
        })
    }

    pub fn has_font(&self, font_key: &FontKey) -> bool {
        self.fonts.contains_key(font_key)
    }

    fn add_font_descriptor(&mut self, font_key: &FontKey, desc: &dwrote::FontDescriptor) {
        let system_fc = dwrote::FontCollection::get_system(false);
        if let Some(font) = system_fc.get_font_from_descriptor(desc) {
            let face = font.create_font_face();
            let file = face.get_files().pop().unwrap();
            let index = face.get_index();
            self.fonts.insert(*font_key, FontFace { cached: None, file, index, face });
        }
    }

    pub fn add_raw_font(&mut self, font_key: &FontKey, data: Arc<Vec<u8>>, index: u32) {
        if self.fonts.contains_key(font_key) {
            return;
        }

        if let Some(file) = dwrote::FontFile::new_from_data(data) {
            if let Ok(face) = file.create_face(index, dwrote::DWRITE_FONT_SIMULATIONS_NONE) {
                self.fonts.insert(*font_key, FontFace { cached: None, file, index, face });
                return;
            }
        }
        // XXX add_raw_font needs to have a way to return an error
        debug!("DWrite WR failed to load font from data, using Arial instead");
        self.add_font_descriptor(font_key, &DEFAULT_FONT_DESCRIPTOR);
    }

    pub fn add_native_font(&mut self, font_key: &FontKey, font_handle: NativeFontHandle) {
        if self.fonts.contains_key(font_key) {
            return;
        }

        let index = font_handle.index;
        let mut cache = FONT_CACHE.lock().unwrap();
        // Check to see if the font is already in the cache. If so, reuse it.
        if let Some(font) = cache.get(font_handle.path.as_path()) {
            if let Ok(face) = font.file.create_face(index, dwrote::DWRITE_FONT_SIMULATIONS_NONE) {
                self.fonts.insert(
                    *font_key,
                    FontFace { cached: Some(font.key.clone()), file: font.file.clone(), index, face },
                );
                return;
            }
        }
        if let Some(file) = dwrote::FontFile::new_from_path(&font_handle.path) {
            // The font is not in the cache yet, so try to create the font and insert it in the cache.
            if let Ok(face) = file.create_face(index, dwrote::DWRITE_FONT_SIMULATIONS_NONE) {
                let key: CachedFontKey = font_handle.path.into();
                self.fonts.insert(
                    *font_key,
                    FontFace { cached: Some(key.clone()), file: file.clone(), index, face },
                );
                cache.insert(CachedFont { key, file });
                return;
            }
        }

        // XXX add_native_font needs to have a way to return an error
        debug!("DWrite WR failed to load font from path, using Arial instead");
        self.add_font_descriptor(font_key, &DEFAULT_FONT_DESCRIPTOR);
    }

    pub fn delete_font(&mut self, font_key: &FontKey) {
        if let Some(face) = self.fonts.remove(font_key) {
            self.variations.retain(|k, _| k.0 != *font_key);
            // Check if this was a cached font.
            if let Some(key) = face.cached {
                let mut cache = FONT_CACHE.lock().unwrap();
                // If there are only two references left, that means only this face and
                // the cache are using the font. So remove it from the cache.
                if Arc::strong_count(&key) == 2 {
                    cache.remove(&*key);
                }
            }
        }
    }

    pub fn delete_font_instance(&mut self, instance: &FontInstance) {
        // Ensure we don't keep around excessive amounts of stale variations.
        if !instance.variations.is_empty() {
            let sims = if instance.flags.contains(FontInstanceFlags::SYNTHETIC_BOLD) {
                dwrote::DWRITE_FONT_SIMULATIONS_BOLD
            } else {
                dwrote::DWRITE_FONT_SIMULATIONS_NONE
            };
            self.variations.remove(&(instance.font_key, sims, instance.variations.clone()));
        }
    }

    // Assumes RGB format from dwrite, which is 3 bytes per pixel as dwrite
    // doesn't output an alpha value via GlyphRunAnalysis::CreateAlphaTexture
    #[allow(dead_code)]
    fn print_glyph_data(&self, data: &[u8], width: usize, height: usize) {
        // Rust doesn't have step_by support on stable :(
        for i in 0 .. height {
            let current_height = i * width * 3;

            for pixel in data[current_height .. current_height + (width * 3)].chunks(3) {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                print!("({}, {}, {}) ", r, g, b,);
            }
            println!();
        }
    }

    fn get_font_face(
        &mut self,
        font: &FontInstance,
    ) -> &dwrote::FontFace {
        if !font.flags.contains(FontInstanceFlags::SYNTHETIC_BOLD) &&
           font.variations.is_empty() {
            return &self.fonts.get(&font.font_key).unwrap().face;
        }
        let sims = if font.flags.contains(FontInstanceFlags::SYNTHETIC_BOLD) {
            dwrote::DWRITE_FONT_SIMULATIONS_BOLD
        } else {
            dwrote::DWRITE_FONT_SIMULATIONS_NONE
        };
        match self.variations.entry((font.font_key, sims, font.variations.clone())) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let normal_face = self.fonts.get(&font.font_key).unwrap();
                if !font.variations.is_empty() {
                    if let Some(var_face) = normal_face.face.create_font_face_with_variations(
                        sims,
                        &font.variations.iter().map(|var| {
                            dwrote::DWRITE_FONT_AXIS_VALUE {
                                // OpenType tags are big-endian, but DWrite wants little-endian.
                                axisTag: var.tag.swap_bytes(),
                                value: var.value,
                            }
                        }).collect::<Vec<_>>(),
                    ) {
                        return entry.insert(var_face);
                    }
                }
                let var_face = normal_face.file
                    .create_face(normal_face.index, sims)
                    .unwrap_or_else(|_| normal_face.face.clone());
                entry.insert(var_face)
            }
        }
    }

    fn create_glyph_analysis(
        &mut self,
        font: &FontInstance,
        key: &GlyphKey,
        size: f32,
        transform: Option<dwrote::DWRITE_MATRIX>,
        bitmaps: bool,
    ) -> Result<(dwrote::GlyphRunAnalysis, dwrote::DWRITE_TEXTURE_TYPE, dwrote::RECT), dwrote::HRESULT> {
        let face = self.get_font_face(font);
        let glyph = key.index() as u16;
        let advance = 0.0f32;
        let offset = dwrote::GlyphOffset {
            advanceOffset: 0.0,
            ascenderOffset: 0.0,
        };

        let glyph_run = dwrote::DWRITE_GLYPH_RUN {
            fontFace: unsafe { face.as_ptr() },
            fontEmSize: size, // size in DIPs (1/96", same as CSS pixels)
            glyphCount: 1,
            glyphIndices: &glyph,
            glyphAdvances: &advance,
            glyphOffsets: &offset,
            isSideways: 0,
            bidiLevel: 0,
        };

        let dwrite_measure_mode = dwrite_measure_mode(font, bitmaps);
        let dwrite_render_mode = dwrite_render_mode(
            face,
            font,
            size,
            dwrite_measure_mode,
            bitmaps,
        );

        let analysis = dwrote::GlyphRunAnalysis::create(
            &glyph_run,
            1.0,
            transform,
            dwrite_render_mode,
            dwrite_measure_mode,
            0.0,
            0.0,
        )?;
        let texture_type = dwrite_texture_type(font.render_mode);
        let bounds = analysis.get_alpha_texture_bounds(texture_type)?;
        // If the bounds are empty, then we might not be able to render the glyph with cleartype.
        // Try again with aliased rendering to check if that works instead.
        if font.render_mode != FontRenderMode::Mono &&
           (bounds.left == bounds.right || bounds.top == bounds.bottom) {
            let analysis2 = dwrote::GlyphRunAnalysis::create(
                &glyph_run,
                1.0,
                transform,
                dwrote::DWRITE_RENDERING_MODE_ALIASED,
                dwrite_measure_mode,
                0.0,
                0.0,
            )?;
            let bounds2 = analysis2.get_alpha_texture_bounds(dwrote::DWRITE_TEXTURE_ALIASED_1x1)?;
            if bounds2.left != bounds2.right && bounds2.top != bounds2.bottom {
                return Ok((analysis2, dwrote::DWRITE_TEXTURE_ALIASED_1x1, bounds2));
            }
        }
        Ok((analysis, texture_type, bounds))
    }

    pub fn get_glyph_index(&mut self, font_key: FontKey, ch: char) -> Option<u32> {
        let face = &self.fonts.get(&font_key).unwrap().face;
        let indices = face.get_glyph_indices(&[ch as u32]);
        indices.first().map(|idx| *idx as u32)
    }

    pub fn get_glyph_dimensions(
        &mut self,
        font: &FontInstance,
        key: &GlyphKey,
    ) -> Option<GlyphDimensions> {
        let (size, _, bitmaps, transform) = Self::get_glyph_parameters(font, key);
        let (_, _, bounds) = self.create_glyph_analysis(font, key, size, transform, bitmaps).ok()?;

        let width = (bounds.right - bounds.left) as i32;
        let height = (bounds.bottom - bounds.top) as i32;

        // Alpha texture bounds can sometimes return an empty rect
        // Such as for spaces
        if width == 0 || height == 0 {
            return None;
        }

        let face = self.get_font_face(font);
        face.get_design_glyph_metrics(&[key.index() as u16], false)
            .first()
            .map(|metrics| {
                let em_size = size / 16.;
                let design_units_per_pixel = face.metrics().metrics0().designUnitsPerEm as f32 / 16. as f32;
                let scaled_design_units_to_pixels = em_size / design_units_per_pixel;
                let advance = metrics.advanceWidth as f32 * scaled_design_units_to_pixels;

                GlyphDimensions {
                    left: bounds.left,
                    top: -bounds.top,
                    width,
                    height,
                    advance,
                }
            })
    }

    // DWrite ClearType gives us values in RGB, but WR expects BGRA.
    fn convert_to_bgra(
        &self,
        pixels: &[u8],
        width: usize,
        height: usize,
        texture_type: dwrote::DWRITE_TEXTURE_TYPE,
        render_mode: FontRenderMode,
        bitmaps: bool,
        subpixel_bgr: bool,
        texture_padding: bool,
    ) -> Vec<u8> {
        let (buffer_width, buffer_height, padding) = if texture_padding {
            (width + 2, height + 2, 1)
        } else {
            (width, height, 0)
        };

        let buffer_length = buffer_width * buffer_height * 4;
        let mut bgra_pixels: Vec<u8> = vec![0; buffer_length];

        match (texture_type, render_mode, bitmaps) {
            (dwrote::DWRITE_TEXTURE_ALIASED_1x1, _, _) => {
                assert!(width * height == pixels.len());
                let mut i = 0;
                for row in padding .. height + padding {
                    let row_offset = row * buffer_width;
                    for col in padding .. width + padding {
                        let offset = (row_offset + col) * 4;
                        let alpha = pixels[i];
                        i += 1;
                        bgra_pixels[offset + 0] = alpha;
                        bgra_pixels[offset + 1] = alpha;
                        bgra_pixels[offset + 2] = alpha;
                        bgra_pixels[offset + 3] = alpha;
                    }
                }
            }
            (_, FontRenderMode::Subpixel, false) => {
                assert!(width * height * 3 == pixels.len());
                let mut i = 0;
                for row in padding .. height + padding {
                    let row_offset = row * buffer_width;
                    for col in padding .. width + padding {
                        let offset = (row_offset + col) * 4;
                        let (mut r, g, mut b) = (pixels[i + 0], pixels[i + 1], pixels[i + 2]);
                        if subpixel_bgr {
                            mem::swap(&mut r, &mut b);
                        }
                        i += 3;
                        bgra_pixels[offset + 0] = b;
                        bgra_pixels[offset + 1] = g;
                        bgra_pixels[offset + 2] = r;
                        bgra_pixels[offset + 3] = 0xff;
                    }
                }
            }
            _ => {
                assert!(width * height * 3 == pixels.len());
                let mut i = 0;
                for row in padding .. height + padding {
                    let row_offset = row * buffer_width;
                    for col in padding .. width + padding {
                        let offset = (row_offset + col) * 4;
                        // Only take the G channel, as its closest to D2D
                        let alpha = pixels[i + 1] as u8;
                        i += 3;
                        bgra_pixels[offset + 0] = alpha;
                        bgra_pixels[offset + 1] = alpha;
                        bgra_pixels[offset + 2] = alpha;
                        bgra_pixels[offset + 3] = alpha;
                    }
                }
            }
        };
        bgra_pixels
    }

    pub fn prepare_font(font: &mut FontInstance) {
        match font.render_mode {
            FontRenderMode::Mono => {
                // In mono mode the color of the font is irrelevant.
                font.color = ColorU::new(255, 255, 255, 255);
                // Subpixel positioning is disabled in mono mode.
                font.disable_subpixel_position();
            }
            FontRenderMode::Alpha => {
                font.color = font.color.luminance_color().quantize();
            }
            FontRenderMode::Subpixel => {
                font.color = font.color.quantize();
            }
        }
    }

    fn get_glyph_parameters(font: &FontInstance, key: &GlyphKey) -> (f32, f64, bool, Option<dwrote::DWRITE_MATRIX>) {
        let (_, y_scale) = font.transform.compute_scale().unwrap_or((1.0, 1.0));
        let scaled_size = font.size.to_f64_px() * y_scale;
        let bitmaps = is_bitmap_font(font);
        let (mut shape, (mut x_offset, mut y_offset)) = if bitmaps {
            (FontTransform::identity(), (0.0, 0.0))
        } else {
            (font.transform.invert_scale(y_scale, y_scale), font.get_subpx_offset(key))
        };
        if font.flags.contains(FontInstanceFlags::FLIP_X) {
            shape = shape.flip_x();
        }
        if font.flags.contains(FontInstanceFlags::FLIP_Y) {
            shape = shape.flip_y();
        }
        if font.flags.contains(FontInstanceFlags::TRANSPOSE) {
            shape = shape.swap_xy();
        }
        let (mut tx, mut ty) = (0.0, 0.0);
        if font.synthetic_italics.is_enabled() {
            let (shape_, (tx_, ty_)) = font.synthesize_italics(shape, scaled_size);
            shape = shape_;
            tx = tx_;
            ty = ty_;
        };
        x_offset += tx;
        y_offset += ty;
        let transform = if !shape.is_identity() || (x_offset, y_offset) != (0.0, 0.0) {
            Some(dwrote::DWRITE_MATRIX {
                m11: shape.scale_x,
                m12: shape.skew_y,
                m21: shape.skew_x,
                m22: shape.scale_y,
                dx: x_offset as f32,
                dy: y_offset as f32,
            })
        } else {
            None
        };
        (scaled_size as f32, y_scale, bitmaps, transform)
    }

    pub fn rasterize_glyph(&mut self, font: &FontInstance, key: &GlyphKey) -> GlyphRasterResult {
        let (size, y_scale, bitmaps, transform) = Self::get_glyph_parameters(font, key);
        let (analysis, texture_type, bounds) = self.create_glyph_analysis(font, key, size, transform, bitmaps)
                                                   .or(Err(GlyphRasterError::LoadFailed))?;
        let width = (bounds.right - bounds.left) as i32;
        let height = (bounds.bottom - bounds.top) as i32;
        // Alpha texture bounds can sometimes return an empty rect
        // Such as for spaces
        if width == 0 || height == 0 {
            return Err(GlyphRasterError::LoadFailed);
        }

        let pixels = analysis.create_alpha_texture(texture_type, bounds).or(Err(GlyphRasterError::LoadFailed))?;
        let mut bgra_pixels = self.convert_to_bgra(&pixels, width as usize, height as usize,
                                                   texture_type, font.render_mode, bitmaps,
                                                   font.flags.contains(FontInstanceFlags::SUBPIXEL_BGR),
                                                   font.use_texture_padding());

        let FontInstancePlatformOptions { gamma, contrast, cleartype_level, .. } =
            font.platform_options.unwrap_or_default();
        let gamma_lut = self.gamma_luts
            .entry((gamma, contrast))
            .or_insert_with(||
                GammaLut::new(
                    contrast as f32 / 100.0,
                    gamma as f32 / 100.0,
                    gamma as f32 / 100.0,
                ));
        if bitmaps || texture_type == dwrote::DWRITE_TEXTURE_ALIASED_1x1 ||
           font.render_mode != FontRenderMode::Subpixel {
            gamma_lut.preblend(&mut bgra_pixels, font.color);
        } else {
            gamma_lut.preblend_scaled(&mut bgra_pixels, font.color, cleartype_level);
        }

        let format = if bitmaps {
            GlyphFormat::Bitmap
        } else if texture_type == dwrote::DWRITE_TEXTURE_ALIASED_1x1 {
            font.get_alpha_glyph_format()
        } else {
            font.get_glyph_format()
        };

        let padding = if font.use_texture_padding() { 1 } else { 0 };
        Ok(RasterizedGlyph {
            left: (bounds.left - padding) as f32,
            top: (-bounds.top + padding) as f32,
            width: width + padding * 2,
            height: height + padding * 2,
            scale: (if bitmaps { y_scale.recip() } else { 1.0 }) as f32,
            format,
            bytes: bgra_pixels,
        })
    }
}
