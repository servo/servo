/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use api::{ColorU, ColorF, ImageFormat, TextureTarget};
use api::units::*;
use crate::debug_font_data;
use crate::device::{Device, Program, Texture, TextureSlot, VertexDescriptor, ShaderError, VAO};
use crate::device::{TextureFilter, VertexAttribute, VertexAttributeKind, VertexUsageHint};
use euclid::{Point2D, Rect, Size2D, Transform3D, default};
use crate::internal_types::Swizzle;
use std::f32;

#[cfg_attr(feature = "capture", derive(Serialize))]
#[cfg_attr(feature = "replay", derive(Deserialize))]
pub enum DebugItem {
    Text {
        msg: String,
        color: ColorF,
        position: DevicePoint,
    },
    Rect {
        outer_color: ColorF,
        inner_color: ColorF,
        rect: DeviceRect,
    },
}

#[derive(Debug, Copy, Clone)]
enum DebugSampler {
    Font,
}

impl Into<TextureSlot> for DebugSampler {
    fn into(self) -> TextureSlot {
        match self {
            DebugSampler::Font => TextureSlot(0),
        }
    }
}

const DESC_FONT: VertexDescriptor = VertexDescriptor {
    vertex_attributes: &[
        VertexAttribute {
            name: "aPosition",
            count: 2,
            kind: VertexAttributeKind::F32,
        },
        VertexAttribute {
            name: "aColor",
            count: 4,
            kind: VertexAttributeKind::U8Norm,
        },
        VertexAttribute {
            name: "aColorTexCoord",
            count: 2,
            kind: VertexAttributeKind::F32,
        },
    ],
    instance_attributes: &[],
};

const DESC_COLOR: VertexDescriptor = VertexDescriptor {
    vertex_attributes: &[
        VertexAttribute {
            name: "aPosition",
            count: 2,
            kind: VertexAttributeKind::F32,
        },
        VertexAttribute {
            name: "aColor",
            count: 4,
            kind: VertexAttributeKind::U8Norm,
        },
    ],
    instance_attributes: &[],
};

#[repr(C)]
pub struct DebugFontVertex {
    pub x: f32,
    pub y: f32,
    pub color: ColorU,
    pub u: f32,
    pub v: f32,
}

impl DebugFontVertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32, color: ColorU) -> DebugFontVertex {
        DebugFontVertex { x, y, color, u, v }
    }
}

#[repr(C)]
pub struct DebugColorVertex {
    pub x: f32,
    pub y: f32,
    pub color: ColorU,
}

impl DebugColorVertex {
    pub fn new(x: f32, y: f32, color: ColorU) -> DebugColorVertex {
        DebugColorVertex { x, y, color }
    }
}

pub struct DebugRenderer {
    font_vertices: Vec<DebugFontVertex>,
    font_indices: Vec<u32>,
    font_program: Program,
    font_vao: VAO,
    font_texture: Texture,

    tri_vertices: Vec<DebugColorVertex>,
    tri_indices: Vec<u32>,
    tri_vao: VAO,
    line_vertices: Vec<DebugColorVertex>,
    line_vao: VAO,
    color_program: Program,
}

impl DebugRenderer {
    pub fn new(device: &mut Device) -> Result<Self, ShaderError> {
        let font_program = device.create_program_linked(
            "debug_font",
            &[],
            &DESC_FONT,
        )?;
        device.bind_program(&font_program);
        device.bind_shader_samplers(&font_program, &[("sColor0", DebugSampler::Font)]);

        let color_program = device.create_program_linked(
            "debug_color",
            &[],
            &DESC_COLOR,
        )?;

        let font_vao = device.create_vao(&DESC_FONT);
        let line_vao = device.create_vao(&DESC_COLOR);
        let tri_vao = device.create_vao(&DESC_COLOR);

        let font_texture = device.create_texture(
            TextureTarget::Array,
            ImageFormat::R8,
            debug_font_data::BMP_WIDTH,
            debug_font_data::BMP_HEIGHT,
            TextureFilter::Linear,
            None,
            1,
        );
        device.upload_texture_immediate(
            &font_texture,
            &debug_font_data::FONT_BITMAP
        );

        Ok(DebugRenderer {
            font_vertices: Vec::new(),
            font_indices: Vec::new(),
            line_vertices: Vec::new(),
            tri_vao,
            tri_vertices: Vec::new(),
            tri_indices: Vec::new(),
            font_program,
            color_program,
            font_vao,
            line_vao,
            font_texture,
        })
    }

    pub fn deinit(self, device: &mut Device) {
        device.delete_texture(self.font_texture);
        device.delete_program(self.font_program);
        device.delete_program(self.color_program);
        device.delete_vao(self.tri_vao);
        device.delete_vao(self.line_vao);
        device.delete_vao(self.font_vao);
    }

    pub fn line_height(&self) -> f32 {
        debug_font_data::FONT_SIZE as f32 * 1.1
    }

    /// Draws a line of text at the provided starting coordinates.
    ///
    /// If |bounds| is specified, glyphs outside the bounds are discarded.
    ///
    /// Y-coordinates is relative to screen top, along with everything else in
    /// this file.
    pub fn add_text(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        color: ColorU,
        bounds: Option<DeviceRect>,
    ) -> default::Rect<f32> {
        let mut x_start = x;
        let ipw = 1.0 / debug_font_data::BMP_WIDTH as f32;
        let iph = 1.0 / debug_font_data::BMP_HEIGHT as f32;

        let mut min_x = f32::MAX;
        let mut max_x = -f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_y = -f32::MAX;

        for c in text.chars() {
            let c = c as usize - debug_font_data::FIRST_GLYPH_INDEX as usize;
            if c < debug_font_data::GLYPHS.len() {
                let glyph = &debug_font_data::GLYPHS[c];

                let x0 = (x_start + glyph.xo + 0.5).floor();
                let y0 = (y + glyph.yo + 0.5).floor();

                let x1 = x0 + glyph.x1 as f32 - glyph.x0 as f32;
                let y1 = y0 + glyph.y1 as f32 - glyph.y0 as f32;

                // If either corner of the glyph will end up out of bounds, drop it.
                if let Some(b) = bounds {
                    let rect = DeviceRect::new(
                        DevicePoint::new(x0, y0),
                        DeviceSize::new(x1 - x0, y1 - y0),
                    );
                    if !b.contains_rect(&rect) {
                        continue;
                    }
                }

                let s0 = glyph.x0 as f32 * ipw;
                let t0 = glyph.y0 as f32 * iph;
                let s1 = glyph.x1 as f32 * ipw;
                let t1 = glyph.y1 as f32 * iph;

                x_start += glyph.xa;

                let vertex_count = self.font_vertices.len() as u32;

                self.font_vertices
                    .push(DebugFontVertex::new(x0, y0, s0, t0, color));
                self.font_vertices
                    .push(DebugFontVertex::new(x1, y0, s1, t0, color));
                self.font_vertices
                    .push(DebugFontVertex::new(x0, y1, s0, t1, color));
                self.font_vertices
                    .push(DebugFontVertex::new(x1, y1, s1, t1, color));

                self.font_indices.push(vertex_count + 0);
                self.font_indices.push(vertex_count + 1);
                self.font_indices.push(vertex_count + 2);
                self.font_indices.push(vertex_count + 2);
                self.font_indices.push(vertex_count + 1);
                self.font_indices.push(vertex_count + 3);

                min_x = min_x.min(x0);
                max_x = max_x.max(x1);
                min_y = min_y.min(y0);
                max_y = max_y.max(y1);
            }
        }

        Rect::new(
            Point2D::new(min_x, min_y),
            Size2D::new(max_x - min_x, max_y - min_y),
        )
    }

    pub fn add_quad(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color_top: ColorU,
        color_bottom: ColorU,
    ) {
        let vertex_count = self.tri_vertices.len() as u32;

        self.tri_vertices
            .push(DebugColorVertex::new(x0, y0, color_top));
        self.tri_vertices
            .push(DebugColorVertex::new(x1, y0, color_top));
        self.tri_vertices
            .push(DebugColorVertex::new(x0, y1, color_bottom));
        self.tri_vertices
            .push(DebugColorVertex::new(x1, y1, color_bottom));

        self.tri_indices.push(vertex_count + 0);
        self.tri_indices.push(vertex_count + 1);
        self.tri_indices.push(vertex_count + 2);
        self.tri_indices.push(vertex_count + 2);
        self.tri_indices.push(vertex_count + 1);
        self.tri_indices.push(vertex_count + 3);
    }

    #[allow(dead_code)]
    pub fn add_line(&mut self, x0: i32, y0: i32, color0: ColorU, x1: i32, y1: i32, color1: ColorU) {
        self.line_vertices
            .push(DebugColorVertex::new(x0 as f32, y0 as f32, color0));
        self.line_vertices
            .push(DebugColorVertex::new(x1 as f32, y1 as f32, color1));
    }


    pub fn add_rect(&mut self, rect: &DeviceIntRect, color: ColorU) {
        let p0 = rect.origin;
        let p1 = p0 + rect.size;
        self.add_line(p0.x, p0.y, color, p1.x, p0.y, color);
        self.add_line(p1.x, p0.y, color, p1.x, p1.y, color);
        self.add_line(p1.x, p1.y, color, p0.x, p1.y, color);
        self.add_line(p0.x, p1.y, color, p0.x, p0.y, color);
    }

    pub fn render(
        &mut self,
        device: &mut Device,
        viewport_size: Option<DeviceIntSize>,
        scale: f32,
        surface_origin_is_top_left: bool,
    ) {
        if let Some(viewport_size) = viewport_size {
            device.disable_depth();
            device.set_blend(true);
            device.set_blend_mode_premultiplied_alpha();

            let (bottom, top) = if surface_origin_is_top_left {
                (0.0, viewport_size.height as f32 * scale)
            } else {
                (viewport_size.height as f32 * scale, 0.0)
            };

            let projection = Transform3D::ortho(
                0.0,
                viewport_size.width as f32 * scale,
                bottom,
                top,
                device.ortho_near_plane(),
                device.ortho_far_plane(),
            );

            // Triangles
            if !self.tri_vertices.is_empty() {
                device.bind_program(&self.color_program);
                device.set_uniforms(&self.color_program, &projection);
                device.bind_vao(&self.tri_vao);
                device.update_vao_indices(&self.tri_vao, &self.tri_indices, VertexUsageHint::Dynamic);
                device.update_vao_main_vertices(
                    &self.tri_vao,
                    &self.tri_vertices,
                    VertexUsageHint::Dynamic,
                );
                device.draw_triangles_u32(0, self.tri_indices.len() as i32);
            }

            // Lines
            if !self.line_vertices.is_empty() {
                device.bind_program(&self.color_program);
                device.set_uniforms(&self.color_program, &projection);
                device.bind_vao(&self.line_vao);
                device.update_vao_main_vertices(
                    &self.line_vao,
                    &self.line_vertices,
                    VertexUsageHint::Dynamic,
                );
                device.draw_nonindexed_lines(0, self.line_vertices.len() as i32);
            }

            // Glyph
            if !self.font_indices.is_empty() {
                device.bind_program(&self.font_program);
                device.set_uniforms(&self.font_program, &projection);
                device.bind_texture(DebugSampler::Font, &self.font_texture, Swizzle::default());
                device.bind_vao(&self.font_vao);
                device.update_vao_indices(&self.font_vao, &self.font_indices, VertexUsageHint::Dynamic);
                device.update_vao_main_vertices(
                    &self.font_vao,
                    &self.font_vertices,
                    VertexUsageHint::Dynamic,
                );
                device.draw_triangles_u32(0, self.font_indices.len() as i32);
            }
        }

        self.font_indices.clear();
        self.font_vertices.clear();
        self.line_vertices.clear();
        self.tri_vertices.clear();
        self.tri_indices.clear();
    }
}
