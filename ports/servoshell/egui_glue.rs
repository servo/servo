/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A modified version of EguiGlow [from egui_glow 0.22.0][0] that retains its shapes,
//! allowing [`EguiGlow::paint`] to be called multiple times.
//!
//! [0]: https://github.com/emilk/egui/blob/0.22.0/crates/egui_glow/src/winit.rs

// Copyright (c) 2018-2021 Emil Ernerfeldt <emil.ernerfeldt@gmail.com>
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use egui::{ViewportId, ViewportOutput};
use egui_glow::ShaderVersion;
pub use egui_winit;
use egui_winit::winit;
pub use egui_winit::EventResponse;

/// Use [`egui`] from a [`glow`] app based on [`winit`].
pub struct EguiGlow {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub painter: egui_glow::Painter,

    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
}

impl EguiGlow {
    /// For automatic shader version detection set `shader_version` to `None`.
    pub fn new<E>(
        event_loop: &winit::event_loop::EventLoopWindowTarget<E>,
        gl: std::sync::Arc<glow::Context>,
        shader_version: Option<ShaderVersion>,
    ) -> Self {
        let painter = egui_glow::Painter::new(gl, "", shader_version)
            .map_err(|err| {
                log::error!("error occurred in initializing painter:\n{err}");
            })
            .unwrap();

        let egui_ctx = egui::Context::default();
        Self {
            egui_winit: egui_winit::State::new(
                egui_ctx.clone(),
                ViewportId::ROOT,
                event_loop,
                None,
                None,
            ),
            egui_ctx,
            painter,
            shapes: Default::default(),
            textures_delta: Default::default(),
        }
    }

    pub fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_winit.on_window_event(window, event)
    }

    /// Returns the `Duration` of the timeout after which egui should be repainted even if there's no new events.
    ///
    /// Call [`Self::paint`] later to paint.
    pub fn run(
        &mut self,
        window: &winit::window::Window,
        run_ui: impl FnMut(&egui::Context),
    ) -> std::time::Duration {
        let raw_input = self.egui_winit.take_egui_input(window);
        let egui::FullOutput {
            platform_output,
            viewport_output,
            textures_delta,
            shapes,
            pixels_per_point: _pixels_per_point,
        } = self.egui_ctx.run(raw_input, run_ui);

        self.egui_winit
            .handle_platform_output(window, platform_output);

        self.shapes = shapes;
        self.textures_delta.append(textures_delta);
        match viewport_output.get(&ViewportId::ROOT) {
            Some(&ViewportOutput { repaint_delay, .. }) => repaint_delay,
            None => std::time::Duration::ZERO,
        }
    }

    /// Paint the results of the last call to [`Self::run`].
    pub fn paint(&mut self, window: &winit::window::Window) {
        /////// let shapes = std::mem::take(&mut self.shapes);
        let shapes = &self.shapes;
        let mut textures_delta = std::mem::take(&mut self.textures_delta);

        for (id, image_delta) in textures_delta.set {
            self.painter.set_texture(id, &image_delta);
        }

        let pixels_per_point = self.egui_ctx.pixels_per_point();
        /////// let clipped_primitives = self.egui_ctx.tessellate(shapes);
        let clipped_primitives = self.egui_ctx.tessellate(shapes.clone(), pixels_per_point);
        let dimensions: [u32; 2] = window.inner_size().into();
        self.painter
            .paint_primitives(dimensions, pixels_per_point, &clipped_primitives);

        for id in textures_delta.free.drain(..) {
            self.painter.free_texture(id);
        }
    }

    /// Call to release the allocated graphics resources.
    pub fn destroy(&mut self) {
        self.painter.destroy();
    }
}
