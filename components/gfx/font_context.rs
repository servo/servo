/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::{Font, FontGroup};
use font::SpecifiedFontStyle;
use platform::font_context::FontContextHandle;
use style::computed_values::font_style;

use font_cache_task::FontCacheTask;
use font_template::FontTemplateDescriptor;
use platform::font_template::FontTemplateData;
use font::FontHandleMethods;
use platform::font::FontHandle;
use servo_util::cache::HashCache;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use sync::Arc;

use azure::AzFloat;
use azure::azure_hl::BackendType;
use azure::scaled_font::ScaledFont;

#[cfg(target_os="linux")]
#[cfg(target_os="android")]
use azure::scaled_font::FontData;

#[cfg(target_os="linux")]
#[cfg(target_os="android")]
fn create_scaled_font(backend: BackendType, template: &Arc<FontTemplateData>, pt_size: f64) -> ScaledFont {
    ScaledFont::new(backend, FontData(&template.bytes), pt_size as AzFloat)
}

#[cfg(target_os="macos")]
fn create_scaled_font(backend: BackendType, template: &Arc<FontTemplateData>, pt_size: f64) -> ScaledFont {
    let cgfont = template.ctfont.get_ref().copy_to_CGFont();
    ScaledFont::new(backend, &cgfont, pt_size as AzFloat)
}

/// A cached azure font (per render task) that
/// can be shared by multiple text runs.
struct RenderFontCacheEntry {
    pt_size: f64,
    identifier: String,
    font: Rc<RefCell<ScaledFont>>,
}

/// The FontContext represents the per-thread/task state necessary for
/// working with fonts. It is the public API used by the layout and
/// render code. It talks directly to the font cache task where
/// required.
pub struct FontContext {
    platform_handle: FontContextHandle,
    font_cache_task: FontCacheTask,

    /// Weak reference as the layout FontContext is persistent.
    layout_font_cache: Vec<Weak<RefCell<Font>>>,

    /// Strong reference as the render FontContext is (for now) recycled
    /// per frame. TODO: Make this weak when incremental redraw is done.
    render_font_cache: Vec<RenderFontCacheEntry>,
}

impl FontContext {
    pub fn new(font_cache_task: FontCacheTask) -> FontContext {
        let handle = FontContextHandle::new();
        FontContext {
            platform_handle: handle,
            font_cache_task: font_cache_task,
            layout_font_cache: vec!(),
            render_font_cache: vec!(),
        }
    }

    /// Create a font for use in layout calculations.
    fn create_layout_font(&self, template: Arc<FontTemplateData>,
                            descriptor: FontTemplateDescriptor, pt_size: f64) -> Font {

        let handle: FontHandle = FontHandleMethods::new_from_template(&self.platform_handle, template, Some(pt_size)).unwrap();
        let metrics = handle.get_metrics();

        Font {
            handle: handle,
            shaper: None,
            descriptor: descriptor,
            pt_size: pt_size,
            metrics: metrics,
            shape_cache: HashCache::new(),
            glyph_advance_cache: HashCache::new(),
        }
    }

    /// Create a group of fonts for use in layout calculations. May return
    /// a cached font if this font instance has already been used by
    /// this context.
    pub fn get_layout_font_group_for_style(&mut self, style: &SpecifiedFontStyle) -> FontGroup {
        // Remove all weak pointers that have been dropped.
        self.layout_font_cache.retain(|maybe_font| {
            maybe_font.upgrade().is_some()
        });

        let mut fonts: Vec<Rc<RefCell<Font>>> = vec!();

        for family in style.families.iter() {
            let desc = FontTemplateDescriptor::new(style.weight, style.style == font_style::italic);

            // GWTODO: Check on real pages if this is faster as Vec() or HashMap().
            let mut cache_hit = false;
            for maybe_cached_font in self.layout_font_cache.iter() {
                let cached_font = maybe_cached_font.upgrade().unwrap();
                if cached_font.borrow().descriptor == desc {
                    fonts.push(cached_font.clone());
                    cache_hit = true;
                    break;
                }
            }

            if !cache_hit {
                let font_template = self.font_cache_task.get_font_template(family.clone(), desc.clone());
                let layout_font = Rc::new(RefCell::new(self.create_layout_font(font_template, desc.clone(), style.pt_size)));
                self.layout_font_cache.push(layout_font.downgrade());
                fonts.push(layout_font);
            }
        }

        FontGroup::new(fonts)
    }

    /// Create a render font for use with azure. May return a cached
    /// reference if already used by this font context.
    pub fn get_render_font_from_template(&mut self, template: &Arc<FontTemplateData>, pt_size: f64, backend: BackendType) -> Rc<RefCell<ScaledFont>> {
        for cached_font in self.render_font_cache.iter() {
            if cached_font.pt_size == pt_size &&
               cached_font.identifier == template.identifier {
                return cached_font.font.clone();
            }
        }

        let render_font = Rc::new(RefCell::new(create_scaled_font(backend, template, pt_size)));
        self.render_font_cache.push(RenderFontCacheEntry{
            font: render_font.clone(),
            pt_size: pt_size,
            identifier: template.identifier.clone(),
        });
        render_font
    }
}
