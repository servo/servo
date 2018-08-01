/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use azure::azure::AzFloat;
use azure::azure_hl::SurfacePattern;
use azure::azure_hl::{AntialiasMode, AsAzurePoint, CapStyle, CompositionOp, JoinStyle};
use azure::azure_hl::{
    BackendType, DrawOptions, DrawTarget, Pattern, StrokeOptions, SurfaceFormat,
};
use azure::azure_hl::{Color, ColorPattern, DrawSurfaceOptions, Filter, PathBuilder, Path};
use azure::azure_hl::{ExtendMode, GradientStop, LinearGradientPattern, RadialGradientPattern};
use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use num_traits::ToPrimitive;
use std::mem;
use std::sync::Arc;
use webrender::api::DirtyRect;

pub struct CanvasData<'a> {
    drawtarget: DrawTarget,
    /// User-space path builder.
    path_builder: Option<PathBuilder>,
    /// Device-space path builder, if transforms are added during path building.
    device_space_path_builder: Option<PathBuilder>,
    /// Transformation required to move between user-space and device-space.
    path_to_device_space: Option<Transform2D<AzFloat>>,
    /// The user-space path.
    path: Option<Path>,
    state: CanvasPaintState<'a>,
    saved_states: Vec<CanvasPaintState<'a>>,
    webrender_api: webrender_api::RenderApi,
    image_key: Option<webrender_api::ImageKey>,
    /// An old webrender image key that can be deleted when the next epoch ends.
    old_image_key: Option<webrender_api::ImageKey>,
    /// An old webrender image key that can be deleted when the current epoch ends.
    very_old_image_key: Option<webrender_api::ImageKey>,
    pub canvas_id: CanvasId,
}

impl<'a> CanvasData<'a> {
    pub fn new(
        size: Size2D<u32>,
        webrender_api_sender: webrender_api::RenderApiSender,
        antialias: AntialiasMode,
        canvas_id: CanvasId,
    ) -> CanvasData<'a> {
        let draw_target = CanvasData::create(size);
        let webrender_api = webrender_api_sender.create_api();
        CanvasData {
            drawtarget: draw_target,
            path_builder: None,
            device_space_path_builder: None,
            path_to_device_space: None,
            path: None,
            state: CanvasPaintState::new(antialias),
            saved_states: vec![],
            webrender_api: webrender_api,
            image_key: None,
            old_image_key: None,
            very_old_image_key: None,
            canvas_id: canvas_id,
        }
    }

    pub fn draw_image(
        &self,
        image_data: Vec<u8>,
        image_size: Size2D<f64>,
        dest_rect: Rect<f64>,
        source_rect: Rect<f64>,
        smoothing_enabled: bool,
    ) {
        // We round up the floating pixel values to draw the pixels
        let source_rect = source_rect.ceil();
        // It discards the extra pixels (if any) that won't be painted
        let image_data = if Rect::from_size(image_size).contains_rect(&source_rect) {
            pixels::rgba8_get_rect(&image_data, image_size.to_u32(), source_rect.to_u32()).into()
        } else {
            image_data.into()
        };

        let writer = |draw_target: &DrawTarget| {
            write_image(
                &draw_target,
                image_data,
                source_rect.size,
                dest_rect,
                smoothing_enabled,
                self.state.draw_options.composition,
                self.state.draw_options.alpha,
            );
        };

        if self.need_to_draw_shadow() {
            let rect = Rect::new(
                Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32),
            );

            self.draw_with_shadow(&rect, writer);
        } else {
            writer(&self.drawtarget);
        }
    }

    pub fn save_context_state(&mut self) {
        self.saved_states.push(self.state.clone());
    }

    pub fn restore_context_state(&mut self) {
        if let Some(state) = self.saved_states.pop() {
            mem::replace(&mut self.state, state);
            self.drawtarget.set_transform(&self.state.transform);
            self.drawtarget.pop_clip();
        }
    }

    pub fn fill_text(&self, text: String, x: f64, y: f64, max_width: Option<f64>) {
        error!(
            "Unimplemented canvas2d.fillText. Values received: {}, {}, {}, {:?}.",
            text, x, y, max_width
        );
    }

    pub fn fill_rect(&self, rect: &Rect<f32>) {
        if is_zero_size_gradient(&self.state.fill_style) {
            return; // Paint nothing if gradient size is zero.
        }

        let draw_rect = Rect::new(
            rect.origin,
            match self.state.fill_style {
                Pattern::Surface(ref surface) => {
                    let surface_size = surface.size();
                    match (surface.repeat_x, surface.repeat_y) {
                        (true, true) => rect.size,
                        (true, false) => Size2D::new(rect.size.width, surface_size.height as f32),
                        (false, true) => Size2D::new(surface_size.width as f32, rect.size.height),
                        (false, false) => {
                            Size2D::new(surface_size.width as f32, surface_size.height as f32)
                        },
                    }
                },
                _ => rect.size,
            },
        );

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&draw_rect, |new_draw_target: &DrawTarget| {
                new_draw_target.fill_rect(
                    &draw_rect,
                    self.state.fill_style.to_pattern_ref(),
                    Some(&self.state.draw_options),
                );
            });
        } else {
            self.drawtarget.fill_rect(
                &draw_rect,
                self.state.fill_style.to_pattern_ref(),
                Some(&self.state.draw_options),
            );
        }
    }

    pub fn clear_rect(&self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    pub fn stroke_rect(&self, rect: &Rect<f32>) {
        if is_zero_size_gradient(&self.state.stroke_style) {
            return; // Paint nothing if gradient size is zero.
        }

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&rect, |new_draw_target: &DrawTarget| {
                new_draw_target.stroke_rect(
                    rect,
                    self.state.stroke_style.to_pattern_ref(),
                    &self.state.stroke_opts,
                    &self.state.draw_options,
                );
            });
        } else if rect.size.width == 0. || rect.size.height == 0. {
            let cap = match self.state.stroke_opts.line_join {
                JoinStyle::Round => CapStyle::Round,
                _ => CapStyle::Butt,
            };

            let stroke_opts = StrokeOptions::new(
                self.state.stroke_opts.line_width,
                self.state.stroke_opts.line_join,
                cap,
                self.state.stroke_opts.miter_limit,
                self.state.stroke_opts.mDashPattern,
            );
            self.drawtarget.stroke_line(
                rect.origin,
                rect.bottom_right(),
                self.state.stroke_style.to_pattern_ref(),
                &stroke_opts,
                &self.state.draw_options,
            );
        } else {
            self.drawtarget.stroke_rect(
                rect,
                self.state.stroke_style.to_pattern_ref(),
                &self.state.stroke_opts,
                &self.state.draw_options,
            );
        }
    }

    pub fn begin_path(&mut self) {
        // Erase any traces of previous paths that existed before this.
        self.path_builder = None;
        self.device_space_path_builder = None;
        self.path = None;
        self.path_to_device_space = None;
    }

    pub fn close_path(&mut self) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) |
            (None, Some(builder)) => builder.close(),
            _ => unreachable!(),
        }
    }

    fn ensure_path(&mut self) {
        // If there's no record of any path yet, create a new builder in user-space.
        if self.path.is_none() && self.path_builder.is_none() && self.device_space_path_builder.is_none() {
            self.path_builder = Some(self.drawtarget.create_path_builder());
        }

        // If a user-space builder exists, create a finished path from it.
        if let Some(path_builder) = self.path_builder.take() {
            self.path = Some(path_builder.finish());
        }

        // If a user-space path exists, create a device-space builder based on it if
        // any transform is present.
        if self.path.is_some() {
            if let Some(transform) = self.path_to_device_space.take() {
                let path = self.path.take().unwrap();
                self.device_space_path_builder = Some(path.transformed_copy_to_builder(&transform));
            }
        }

        // If a device-space builder is present, create a user-space path from its
        // finished path by inverting the initial transformation.
        if let Some(path_builder) = self.device_space_path_builder.take() {
            let path = path_builder.finish();
            let inverse = match self.drawtarget.get_transform().inverse() {
                Some(m) => m,
                None => {
                    warn!("Couldn't invert canvas transformation.");
                    return;
                }
            };
            let builder = path.transformed_copy_to_builder(&inverse);
            self.path = Some(builder.finish());
        }

        assert!(self.path.is_some());
    }

    fn path(&self) -> &Path {
        self.path.as_ref().expect("Should have called ensure_path()")
    }

    pub fn fill(&mut self) {
        if is_zero_size_gradient(&self.state.fill_style) {
            return; // Paint nothing if gradient size is zero.
        }

        self.ensure_path();
        self.drawtarget.fill(
            &self.path(),
            self.state.fill_style.to_pattern_ref(),
            &self.state.draw_options,
        );
    }

    pub fn stroke(&mut self) {
        if is_zero_size_gradient(&self.state.stroke_style) {
            return; // Paint nothing if gradient size is zero.
        }

        self.ensure_path();
        self.drawtarget.stroke(
            &self.path(),
            self.state.stroke_style.to_pattern_ref(),
            &self.state.stroke_opts,
            &self.state.draw_options,
        );
    }

    pub fn clip(&mut self) {
        self.ensure_path();
        self.drawtarget.push_clip(&self.path());
    }

    pub fn is_point_in_path(
        &mut self,
        x: f64,
        y: f64,
        _fill_rule: FillRule,
        chan: IpcSender<bool>,
    ) {
        self.ensure_path();
        let (path_builder, result) = {
            let path = self.path();
            let transform = match self.path_to_device_space {
                Some(ref transform) => transform,
                None => &self.state.transform,
            };
            let result = path.contains_point(x, y, transform);
            (path.copy_to_builder(), result)
        };
        self.path_builder = Some(path_builder);
        chan.send(result).unwrap();
    }

    pub fn move_to(&mut self, point: &Point2D<AzFloat>) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.move_to(*point),
            (None, Some(builder)) => {
                let xform = self.drawtarget.get_transform();
                builder.move_to(xform.transform_point(point));
            }
            _ => unreachable!(),
        }
    }

    pub fn line_to(&mut self, point: &Point2D<AzFloat>) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.line_to(*point),
            (None, Some(builder)) => {
                let xform = self.drawtarget.get_transform();
                builder.line_to(xform.transform_point(point));
            }
            _ => unreachable!(),
        }
    }

    fn ensure_path_builder(&mut self) {
        // If a device-space builder is present, we're done.
        if self.device_space_path_builder.is_some() {
            return;
        }

        // If a user-space builder is present, convert it to a device-space builder if
        // any transform is present.
        if self.path_builder.is_some() {
            if let Some(transform) = self.path_to_device_space.take() {
                let path = self.path_builder.take().unwrap().finish();
                self.device_space_path_builder = Some(path.transformed_copy_to_builder(&transform));
            }
            return;
        }

        // If there is a path, create a new builder, transforming the path if there is
        // a transform present. Otherwise, create a new builder from scratch.
        match (self.path.take(), self.path_to_device_space.take()) {
            (Some(path), None) => {
                self.path_builder = Some(path.copy_to_builder());
                self.path = Some(path);
            }
            (Some(path), Some(transform)) => {
                self.device_space_path_builder = Some(path.transformed_copy_to_builder(&transform));
            }
            (None, transform) => {
                assert!(transform.is_none());
                self.path_builder = Some(self.drawtarget.create_path_builder());
            }
        }
    }

    pub fn rect(&mut self, rect: &Rect<f32>) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(path_builder), None) => {
                path_builder.move_to(Point2D::new(rect.origin.x, rect.origin.y));
                path_builder.line_to(Point2D::new(rect.origin.x + rect.size.width, rect.origin.y));
                path_builder.line_to(Point2D::new(rect.origin.x + rect.size.width,
                                                  rect.origin.y + rect.size.height));
                path_builder.line_to(Point2D::new(rect.origin.x, rect.origin.y + rect.size.height));
                path_builder.close();
            }
            (None, Some(path_builder)) => {
                let xform = self.drawtarget.get_transform();
                path_builder.move_to(xform.transform_point(
                    &Point2D::new(rect.origin.x, rect.origin.y)));
                path_builder.line_to(xform.transform_point(
                    &Point2D::new(rect.origin.x + rect.size.width, rect.origin.y)));
                path_builder.line_to(xform.transform_point(
                    &Point2D::new(rect.origin.x + rect.size.width,
                                 rect.origin.y + rect.size.height)));
                path_builder.line_to(xform.transform_point(
                    &Point2D::new(rect.origin.x, rect.origin.y + rect.size.height)));
                path_builder.close();
            }
            _ => unreachable!(),
        }
    }

    pub fn quadratic_curve_to(
        &mut self,
        cp: &Point2D<AzFloat>,
        endpoint: &Point2D<AzFloat>
    ) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.quadratic_curve_to(cp, endpoint),
            (None, Some(builder)) => {
                let xform = self.drawtarget.get_transform();
                builder.quadratic_curve_to(&xform.transform_point(cp),
                                           &xform.transform_point(endpoint));
            }
            _ => unreachable!(),
        }
    }

    pub fn bezier_curve_to(
        &mut self,
        cp1: &Point2D<AzFloat>,
        cp2: &Point2D<AzFloat>,
        endpoint: &Point2D<AzFloat>,
    ) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.bezier_curve_to(cp1, cp2, endpoint),
            (None, Some(builder)) => {
                let xform = self.drawtarget.get_transform();
                builder.bezier_curve_to(&xform.transform_point(cp1),
                                        &xform.transform_point(cp2),
                                        &xform.transform_point(endpoint));
            }
            _ => unreachable!(),
        }
    }

    pub fn arc(
        &mut self,
        center: &Point2D<AzFloat>,
        radius: AzFloat,
        start_angle: AzFloat,
        end_angle: AzFloat,
        ccw: bool,
    ) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.arc(*center, radius, start_angle, end_angle, ccw),
            (None, Some(builder)) => {
                let xform = self.drawtarget.get_transform();
                builder.arc(xform.transform_point(center), radius, start_angle, end_angle, ccw)
            }
            _ => unreachable!(),
        }
    }

    pub fn arc_to(
        &mut self,
        cp1: &Point2D<AzFloat>,
        cp2: &Point2D<AzFloat>,
        radius: AzFloat
    ) {
        self.ensure_path_builder();
        let cp0 = match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.get_current_point(),
            (None, Some(builder)) => {
                let inverse = match self.drawtarget.get_transform().inverse() {
                    Some(m) => m,
                    None => return,
                };
                let current_point = builder.get_current_point();
                let transformed = inverse.transform_point(&Point2D::new(current_point.x, current_point.y));
                transformed.as_azure_point()
            }
            _ => unreachable!(),
        };
        let cp1 = *cp1;
        let cp2 = *cp2;

        if (cp0.x == cp1.x && cp0.y == cp1.y) || cp1 == cp2 || radius == 0.0 {
            self.line_to(&cp1);
            return;
        }

        // if all three control points lie on a single straight line,
        // connect the first two by a straight line
        let direction = (cp2.x - cp1.x) * (cp0.y - cp1.y) + (cp2.y - cp1.y) * (cp1.x - cp0.x);
        if direction == 0.0 {
            self.line_to(&cp1);
            return;
        }

        // otherwise, draw the Arc
        let a2 = (cp0.x - cp1.x).powi(2) + (cp0.y - cp1.y).powi(2);
        let b2 = (cp1.x - cp2.x).powi(2) + (cp1.y - cp2.y).powi(2);
        let d = {
            let c2 = (cp0.x - cp2.x).powi(2) + (cp0.y - cp2.y).powi(2);
            let cosx = (a2 + b2 - c2) / (2.0 * (a2 * b2).sqrt());
            let sinx = (1.0 - cosx.powi(2)).sqrt();
            radius / ((1.0 - cosx) / sinx)
        };

        // first tangent point
        let anx = (cp1.x - cp0.x) / a2.sqrt();
        let any = (cp1.y - cp0.y) / a2.sqrt();
        let tp1 = Point2D::new(cp1.x - anx * d, cp1.y - any * d);

        // second tangent point
        let bnx = (cp1.x - cp2.x) / b2.sqrt();
        let bny = (cp1.y - cp2.y) / b2.sqrt();
        let tp2 = Point2D::new(cp1.x - bnx * d, cp1.y - bny * d);

        // arc center and angles
        let anticlockwise = direction < 0.0;
        let cx = tp1.x + any * radius * if anticlockwise { 1.0 } else { -1.0 };
        let cy = tp1.y - anx * radius * if anticlockwise { 1.0 } else { -1.0 };
        let angle_start = (tp1.y - cy).atan2(tp1.x - cx);
        let angle_end = (tp2.y - cy).atan2(tp2.x - cx);

        self.line_to(&tp1);
        if [cx, cy, angle_start, angle_end]
            .iter()
            .all(|x| x.is_finite())
        {
            self.arc(
                &Point2D::new(cx, cy),
                radius,
                angle_start,
                angle_end,
                anticlockwise,
            );
        }
    }

    pub fn ellipse(
        &mut self,
        center: &Point2D<AzFloat>,
        radius_x: AzFloat,
        radius_y: AzFloat,
        rotation_angle: AzFloat,
        start_angle: AzFloat,
        end_angle: AzFloat,
        ccw: bool,
    ) {
        self.ensure_path_builder();
        match (self.path_builder.as_ref(), self.device_space_path_builder.as_ref()) {
            (Some(builder), None) =>
                builder.ellipse(*center, radius_x, radius_y, rotation_angle, start_angle, end_angle, ccw),
            (None, Some(builder)) => {
                let xform = self.drawtarget.get_transform();
                builder.ellipse(xform.transform_point(center), radius_x, radius_y, rotation_angle,
                                start_angle, end_angle, ccw)
            }
            _ => unreachable!(),
        }
    }

    pub fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        if let Some(pattern) = style.to_azure_pattern(&self.drawtarget) {
            self.state.fill_style = pattern
        }
    }

    pub fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        if let Some(pattern) = style.to_azure_pattern(&self.drawtarget) {
            self.state.stroke_style = pattern
        }
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.state.stroke_opts.line_width = width;
    }

    pub fn set_line_cap(&mut self, cap: LineCapStyle) {
        self.state.stroke_opts.line_cap = cap.to_azure_style();
    }

    pub fn set_line_join(&mut self, join: LineJoinStyle) {
        self.state.stroke_opts.line_join = join.to_azure_style();
    }

    pub fn set_miter_limit(&mut self, limit: f32) {
        self.state.stroke_opts.miter_limit = limit;
    }

    pub fn set_transform(&mut self, transform: &Transform2D<f32>) {
        // If there is an in-progress path, store the existing transformation required
        // to move between device and user space.
        if (self.path.is_some() || self.path_builder.is_some()) && self.path_to_device_space.is_none() {
            self.path_to_device_space = Some(self.drawtarget.get_transform());
        }
        self.state.transform = transform.clone();
        self.drawtarget.set_transform(transform)
    }

    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.state.draw_options.alpha = alpha;
    }

    pub fn set_global_composition(&mut self, op: CompositionOrBlending) {
        self.state
            .draw_options
            .set_composition_op(op.to_azure_style());
    }

    pub fn create(size: Size2D<u32>) -> DrawTarget {
        // FIXME(nox): Why is the size made of i32 values?
        DrawTarget::new(BackendType::Skia, size.to_i32(), SurfaceFormat::B8G8R8A8)
    }

    pub fn recreate(&mut self, size: Size2D<u32>) {
        self.drawtarget = CanvasData::create(size);
        self.state = CanvasPaintState::new(self.state.draw_options.antialias);
        self.saved_states.clear();
        // Webrender doesn't let images change size, so we clear the webrender image key.
        // TODO: there is an annying race condition here: the display list builder
        // might still be using the old image key. Really, we should be scheduling the image
        // for later deletion, not deleting it immediately.
        // https://github.com/servo/servo/issues/17534
        if let Some(image_key) = self.image_key.take() {
            // If this executes, then we are in a new epoch since we last recreated the canvas,
            // so `old_image_key` must be `None`.
            debug_assert!(self.old_image_key.is_none());
            self.old_image_key = Some(image_key);
        }
    }

    #[allow(unsafe_code)]
    pub fn send_pixels(&mut self, chan: IpcSender<IpcSharedMemory>) {
        let data = IpcSharedMemory::from_bytes(unsafe {
            self.drawtarget.snapshot().get_data_surface().data()
        });
        chan.send(data).unwrap();
    }

    #[allow(unsafe_code)]
    pub fn send_data(&mut self, chan: IpcSender<CanvasImageData>) {
        let size = self.drawtarget.get_size();

        let descriptor = webrender_api::ImageDescriptor {
            size: webrender_api::DeviceIntSize::new(size.width, size.height),
            stride: None,
            format: webrender_api::ImageFormat::BGRA8,
            offset: 0,
            is_opaque: false,
            allow_mipmaps: false,
        };
        let data = webrender_api::ImageData::Raw(Arc::new(unsafe {
            self.drawtarget.snapshot().get_data_surface().data().into()
        }));

        let mut txn = webrender_api::Transaction::new();

        match self.image_key {
            Some(image_key) => {
                debug!("Updating image {:?}.", image_key);
                txn.update_image(image_key, descriptor, data, &DirtyRect::All);
            },
            None => {
                self.image_key = Some(self.webrender_api.generate_image_key());
                debug!("New image {:?}.", self.image_key);
                txn.add_image(self.image_key.unwrap(), descriptor, data, None);
            },
        }

        if let Some(image_key) =
            mem::replace(&mut self.very_old_image_key, self.old_image_key.take())
        {
            txn.delete_image(image_key);
        }

        self.webrender_api.update_resources(txn.resource_updates);

        let data = CanvasImageData {
            image_key: self.image_key.unwrap(),
        };
        chan.send(data).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    pub fn put_image_data(&mut self, mut imagedata: Vec<u8>, rect: Rect<u32>) {
        assert_eq!(imagedata.len() % 4, 0);
        assert_eq!(rect.size.area() as usize, imagedata.len() / 4);
        pixels::rgba8_byte_swap_and_premultiply_inplace(&mut imagedata);
        let source_surface = self
            .drawtarget
            .create_source_surface_from_data(
                &imagedata,
                rect.size.to_i32(),
                rect.size.width as i32 * 4,
                SurfaceFormat::B8G8R8A8,
            )
            .unwrap();
        self.drawtarget.copy_surface(
            source_surface,
            Rect::from_size(rect.size.to_i32()),
            rect.origin.to_i32(),
        );
    }

    pub fn set_shadow_offset_x(&mut self, value: f64) {
        self.state.shadow_offset_x = value;
    }

    pub fn set_shadow_offset_y(&mut self, value: f64) {
        self.state.shadow_offset_y = value;
    }

    pub fn set_shadow_blur(&mut self, value: f64) {
        self.state.shadow_blur = value;
    }

    pub fn set_shadow_color(&mut self, value: Color) {
        self.state.shadow_color = value;
    }

    // https://html.spec.whatwg.org/multipage/#when-shadows-are-drawn
    fn need_to_draw_shadow(&self) -> bool {
        self.state.shadow_color.a != 0.0f32 &&
            (self.state.shadow_offset_x != 0.0f64 ||
                self.state.shadow_offset_y != 0.0f64 ||
                self.state.shadow_blur != 0.0f64)
    }

    fn create_draw_target_for_shadow(&self, source_rect: &Rect<f32>) -> DrawTarget {
        let draw_target = self.drawtarget.create_similar_draw_target(
            &Size2D::new(
                source_rect.size.width as i32,
                source_rect.size.height as i32,
            ),
            self.drawtarget.get_format(),
        );
        let matrix = Transform2D::identity()
            .pre_translate(-source_rect.origin.to_vector().cast())
            .pre_mul(&self.state.transform);
        draw_target.set_transform(&matrix);
        draw_target
    }

    fn draw_with_shadow<F>(&self, rect: &Rect<f32>, draw_shadow_source: F)
    where
        F: FnOnce(&DrawTarget),
    {
        let shadow_src_rect = self.state.transform.transform_rect(rect);
        let new_draw_target = self.create_draw_target_for_shadow(&shadow_src_rect);
        draw_shadow_source(&new_draw_target);
        self.drawtarget.draw_surface_with_shadow(
            new_draw_target.snapshot(),
            &Point2D::new(
                shadow_src_rect.origin.x as AzFloat,
                shadow_src_rect.origin.y as AzFloat,
            ),
            &self.state.shadow_color,
            &Vector2D::new(
                self.state.shadow_offset_x as AzFloat,
                self.state.shadow_offset_y as AzFloat,
            ),
            (self.state.shadow_blur / 2.0f64) as AzFloat,
            self.state.draw_options.composition,
        );
    }

    /// It reads image data from the canvas
    /// canvas_size: The size of the canvas we're reading from
    /// read_rect: The area of the canvas we want to read from
    #[allow(unsafe_code)]
    pub fn read_pixels(&self, read_rect: Rect<u32>, canvas_size: Size2D<u32>) -> Vec<u8> {
        let canvas_rect = Rect::from_size(canvas_size);
        if canvas_rect
            .intersection(&read_rect)
            .map_or(true, |rect| rect.is_empty())
        {
            return vec![];
        }
        let data_surface = self.drawtarget.snapshot().get_data_surface();
        pixels::rgba8_get_rect(
            unsafe { data_surface.data() },
            canvas_size.to_u32(),
            read_rect.to_u32(),
        )
        .into_owned()
    }
}

impl<'a> Drop for CanvasData<'a> {
    fn drop(&mut self) {
        let mut txn = webrender_api::Transaction::new();

        if let Some(image_key) = self.old_image_key.take() {
            txn.delete_image(image_key);
        }
        if let Some(image_key) = self.very_old_image_key.take() {
            txn.delete_image(image_key);
        }

        self.webrender_api.update_resources(txn.resource_updates);
    }
}

#[derive(Clone)]
struct CanvasPaintState<'a> {
    draw_options: DrawOptions,
    fill_style: Pattern,
    stroke_style: Pattern,
    stroke_opts: StrokeOptions<'a>,
    /// The current 2D transform matrix.
    transform: Transform2D<f32>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: Color,
}

impl<'a> CanvasPaintState<'a> {
    fn new(antialias: AntialiasMode) -> CanvasPaintState<'a> {
        CanvasPaintState {
            draw_options: DrawOptions::new(1.0, CompositionOp::Over, antialias),
            fill_style: Pattern::Color(ColorPattern::new(Color::black())),
            stroke_style: Pattern::Color(ColorPattern::new(Color::black())),
            stroke_opts: StrokeOptions::new(
                1.0,
                JoinStyle::MiterOrBevel,
                CapStyle::Butt,
                10.0,
                &[],
            ),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: Color::transparent(),
        }
    }
}

fn is_zero_size_gradient(pattern: &Pattern) -> bool {
    if let &Pattern::LinearGradient(ref gradient) = pattern {
        if gradient.is_zero_size() {
            return true;
        }
    }
    false
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
fn write_image(
    draw_target: &DrawTarget,
    image_data: Vec<u8>,
    image_size: Size2D<f64>,
    dest_rect: Rect<f64>,
    smoothing_enabled: bool,
    composition_op: CompositionOp,
    global_alpha: f32,
) {
    if image_data.is_empty() {
        return;
    }
    let image_rect = Rect::new(Point2D::zero(), image_size);

    // From spec https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    // When scaling up, if the imageSmoothingEnabled attribute is set to true, the user agent should attempt
    // to apply a smoothing algorithm to the image data when it is scaled.
    // Otherwise, the image must be rendered using nearest-neighbor interpolation.
    let filter = if smoothing_enabled {
        Filter::Linear
    } else {
        Filter::Point
    };
    let image_size = image_size.to_i32();

    let source_surface = draw_target
        .create_source_surface_from_data(
            &image_data,
            image_size,
            image_size.width * 4,
            SurfaceFormat::B8G8R8A8,
        )
        .unwrap();
    let draw_surface_options = DrawSurfaceOptions::new(filter, true);
    let draw_options = DrawOptions::new(global_alpha, composition_op, AntialiasMode::None);
    draw_target.draw_surface(
        source_surface,
        dest_rect.to_azure_style(),
        image_rect.to_azure_style(),
        draw_surface_options,
        draw_options,
    );
}

pub trait PointToi32 {
    fn to_i32(&self) -> Point2D<i32>;
}

impl PointToi32 for Point2D<f64> {
    fn to_i32(&self) -> Point2D<i32> {
        Point2D::new(self.x.to_i32().unwrap(), self.y.to_i32().unwrap())
    }
}

pub trait SizeToi32 {
    fn to_i32(&self) -> Size2D<i32>;
}

impl SizeToi32 for Size2D<f64> {
    fn to_i32(&self) -> Size2D<i32> {
        Size2D::new(self.width.to_i32().unwrap(), self.height.to_i32().unwrap())
    }
}

pub trait RectToi32 {
    fn to_i32(&self) -> Rect<i32>;
    fn ceil(&self) -> Rect<f64>;
}

impl RectToi32 for Rect<f64> {
    fn to_i32(&self) -> Rect<i32> {
        Rect::new(
            Point2D::new(
                self.origin.x.to_i32().unwrap(),
                self.origin.y.to_i32().unwrap(),
            ),
            Size2D::new(
                self.size.width.to_i32().unwrap(),
                self.size.height.to_i32().unwrap(),
            ),
        )
    }

    fn ceil(&self) -> Rect<f64> {
        Rect::new(
            Point2D::new(self.origin.x.ceil(), self.origin.y.ceil()),
            Size2D::new(self.size.width.ceil(), self.size.height.ceil()),
        )
    }
}

pub trait ToAzureStyle {
    type Target;
    fn to_azure_style(self) -> Self::Target;
}

impl ToAzureStyle for Rect<f64> {
    type Target = Rect<AzFloat>;

    fn to_azure_style(self) -> Rect<AzFloat> {
        Rect::new(
            Point2D::new(self.origin.x as AzFloat, self.origin.y as AzFloat),
            Size2D::new(self.size.width as AzFloat, self.size.height as AzFloat),
        )
    }
}

impl ToAzureStyle for LineCapStyle {
    type Target = CapStyle;

    fn to_azure_style(self) -> CapStyle {
        match self {
            LineCapStyle::Butt => CapStyle::Butt,
            LineCapStyle::Round => CapStyle::Round,
            LineCapStyle::Square => CapStyle::Square,
        }
    }
}

impl ToAzureStyle for LineJoinStyle {
    type Target = JoinStyle;

    fn to_azure_style(self) -> JoinStyle {
        match self {
            LineJoinStyle::Round => JoinStyle::Round,
            LineJoinStyle::Bevel => JoinStyle::Bevel,
            LineJoinStyle::Miter => JoinStyle::Miter,
        }
    }
}

impl ToAzureStyle for CompositionStyle {
    type Target = CompositionOp;

    fn to_azure_style(self) -> CompositionOp {
        match self {
            CompositionStyle::SrcIn => CompositionOp::In,
            CompositionStyle::SrcOut => CompositionOp::Out,
            CompositionStyle::SrcOver => CompositionOp::Over,
            CompositionStyle::SrcAtop => CompositionOp::Atop,
            CompositionStyle::DestIn => CompositionOp::DestIn,
            CompositionStyle::DestOut => CompositionOp::DestOut,
            CompositionStyle::DestOver => CompositionOp::DestOver,
            CompositionStyle::DestAtop => CompositionOp::DestAtop,
            CompositionStyle::Copy => CompositionOp::Source,
            CompositionStyle::Lighter => CompositionOp::Add,
            CompositionStyle::Xor => CompositionOp::Xor,
        }
    }
}

impl ToAzureStyle for BlendingStyle {
    type Target = CompositionOp;

    fn to_azure_style(self) -> CompositionOp {
        match self {
            BlendingStyle::Multiply => CompositionOp::Multiply,
            BlendingStyle::Screen => CompositionOp::Screen,
            BlendingStyle::Overlay => CompositionOp::Overlay,
            BlendingStyle::Darken => CompositionOp::Darken,
            BlendingStyle::Lighten => CompositionOp::Lighten,
            BlendingStyle::ColorDodge => CompositionOp::ColorDodge,
            BlendingStyle::ColorBurn => CompositionOp::ColorBurn,
            BlendingStyle::HardLight => CompositionOp::HardLight,
            BlendingStyle::SoftLight => CompositionOp::SoftLight,
            BlendingStyle::Difference => CompositionOp::Difference,
            BlendingStyle::Exclusion => CompositionOp::Exclusion,
            BlendingStyle::Hue => CompositionOp::Hue,
            BlendingStyle::Saturation => CompositionOp::Saturation,
            BlendingStyle::Color => CompositionOp::Color,
            BlendingStyle::Luminosity => CompositionOp::Luminosity,
        }
    }
}

impl ToAzureStyle for CompositionOrBlending {
    type Target = CompositionOp;

    fn to_azure_style(self) -> CompositionOp {
        match self {
            CompositionOrBlending::Composition(op) => op.to_azure_style(),
            CompositionOrBlending::Blending(op) => op.to_azure_style(),
        }
    }
}

pub trait ToAzurePattern {
    fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern>;
}

impl ToAzurePattern for FillOrStrokeStyle {
    fn to_azure_pattern(&self, drawtarget: &DrawTarget) -> Option<Pattern> {
        Some(match *self {
            FillOrStrokeStyle::Color(ref color) => {
                Pattern::Color(ColorPattern::new(color.to_azure_style()))
            },
            FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = linear_gradient_style
                    .stops
                    .iter()
                    .map(|s| GradientStop {
                        offset: s.offset as AzFloat,
                        color: s.color.to_azure_style(),
                    })
                    .collect();

                Pattern::LinearGradient(LinearGradientPattern::new(
                    &Point2D::new(
                        linear_gradient_style.x0 as AzFloat,
                        linear_gradient_style.y0 as AzFloat,
                    ),
                    &Point2D::new(
                        linear_gradient_style.x1 as AzFloat,
                        linear_gradient_style.y1 as AzFloat,
                    ),
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Transform2D::identity(),
                ))
            },
            FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = radial_gradient_style
                    .stops
                    .iter()
                    .map(|s| GradientStop {
                        offset: s.offset as AzFloat,
                        color: s.color.to_azure_style(),
                    })
                    .collect();

                Pattern::RadialGradient(RadialGradientPattern::new(
                    &Point2D::new(
                        radial_gradient_style.x0 as AzFloat,
                        radial_gradient_style.y0 as AzFloat,
                    ),
                    &Point2D::new(
                        radial_gradient_style.x1 as AzFloat,
                        radial_gradient_style.y1 as AzFloat,
                    ),
                    radial_gradient_style.r0 as AzFloat,
                    radial_gradient_style.r1 as AzFloat,
                    drawtarget.create_gradient_stops(&gradient_stops, ExtendMode::Clamp),
                    &Transform2D::identity(),
                ))
            },
            FillOrStrokeStyle::Surface(ref surface_style) => {
                let source_surface = drawtarget.create_source_surface_from_data(
                    &surface_style.surface_data,
                    // FIXME(nox): Why are those i32 values?
                    surface_style.surface_size.to_i32(),
                    surface_style.surface_size.width as i32 * 4,
                    SurfaceFormat::B8G8R8A8,
                )?;
                Pattern::Surface(SurfacePattern::new(
                    source_surface.azure_source_surface,
                    surface_style.repeat_x,
                    surface_style.repeat_y,
                    &Transform2D::identity(),
                ))
            },
        })
    }
}

impl ToAzureStyle for RGBA {
    type Target = Color;

    fn to_azure_style(self) -> Color {
        Color::rgba(
            self.red_f32() as AzFloat,
            self.green_f32() as AzFloat,
            self.blue_f32() as AzFloat,
            self.alpha_f32() as AzFloat,
        )
    }
}
