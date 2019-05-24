/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use azure::azure::{AzFloat, AzGradientStop, AzIntSize, AzPoint};
use azure::azure_hl;
use azure::azure_hl::SurfacePattern;
use azure::azure_hl::{AntialiasMode, AsAzurePoint, CapStyle, JoinStyle};
use azure::azure_hl::{BackendType, DrawTarget};
use azure::azure_hl::{ColorPattern, Filter};
use azure::azure_hl::{LinearGradientPattern, RadialGradientPattern};
use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use num_traits::ToPrimitive;
use std::mem;
use std::sync::Arc;
use webrender::api::DirtyRect;

/// The canvas data stores a state machine for the current status of
/// the path data and any relevant transformations that are
/// applied to it. The Azure drawing API expects the path to be in
/// userspace. However, when a path is being built but the canvas'
/// transform changes, we choose to transform the path and perform
/// further operations to it in device space. When it's time to
/// draw the path, we convert it back to userspace and draw it
/// with the correct transform applied.
enum PathState {
    /// Path builder in user-space. If a transform has been applied
    /// but no further path operations have occurred, it is stored
    /// in the optional field.
    UserSpacePathBuilder(Box<GenericPathBuilder>, Option<Transform2D<f32>>),
    /// Path builder in device-space.
    DeviceSpacePathBuilder(Box<GenericPathBuilder>),
    /// Path in user-space. If a transform has been applied but
    /// but no further path operations have occurred, it is stored
    /// in the optional field.
    UserSpacePath(Path, Option<Transform2D<f32>>),
}

impl PathState {
    fn is_path(&self) -> bool {
        match *self {
            PathState::UserSpacePath(..) => true,
            PathState::UserSpacePathBuilder(..) | PathState::DeviceSpacePathBuilder(..) => false,
        }
    }

    fn path(&self) -> &Path {
        match *self {
            PathState::UserSpacePath(ref p, _) => p,
            PathState::UserSpacePathBuilder(..) | PathState::DeviceSpacePathBuilder(..) => {
                panic!("should have called ensure_path")
            },
        }
    }
}

/// A generic PathBuilder that abstracts the interface for
/// azure's and raqote's PathBuilder.
trait GenericPathBuilder {
    fn arc(
        &self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    );
    fn bezier_curve_to(
        &self,
        control_point1: &Point2D<f32>,
        control_point2: &Point2D<f32>,
        control_point3: &Point2D<f32>,
    );
    fn close(&self);
    fn ellipse(
        &self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    );
    fn get_current_point(&self) -> Point2D<f32>;
    fn line_to(&self, point: Point2D<f32>);
    fn move_to(&self, point: Point2D<f32>);
    fn quadratic_curve_to(&self, control_point: &Point2D<f32>, end_point: &Point2D<f32>);
    fn finish(&self) -> Path;
}

impl GenericPathBuilder for azure_hl::PathBuilder {
    fn arc(
        &self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        self.arc(
            origin as Point2D<AzFloat>,
            radius as AzFloat,
            start_angle as AzFloat,
            end_angle as AzFloat,
            anticlockwise,
        );
    }
    fn bezier_curve_to(
        &self,
        control_point1: &Point2D<f32>,
        control_point2: &Point2D<f32>,
        control_point3: &Point2D<f32>,
    ) {
        self.bezier_curve_to(
            control_point1 as &Point2D<AzFloat>,
            control_point2 as &Point2D<AzFloat>,
            control_point3 as &Point2D<AzFloat>,
        );
    }
    fn close(&self) {
        self.close();
    }
    fn ellipse(
        &self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        self.ellipse(
            origin as Point2D<AzFloat>,
            radius_x as AzFloat,
            radius_y as AzFloat,
            rotation_angle as AzFloat,
            start_angle as AzFloat,
            end_angle as AzFloat,
            anticlockwise,
        );
    }
    fn get_current_point(&self) -> Point2D<f32> {
        let AzPoint { x, y } = self.get_current_point();
        Point2D::new(x as f32, y as f32)
    }
    fn line_to(&self, point: Point2D<f32>) {
        self.line_to(point as Point2D<AzFloat>);
    }
    fn move_to(&self, point: Point2D<f32>) {
        self.move_to(point as Point2D<AzFloat>);
    }
    fn quadratic_curve_to(&self, control_point: &Point2D<f32>, end_point: &Point2D<f32>) {
        self.quadratic_curve_to(
            control_point as &Point2D<AzFloat>,
            end_point as &Point2D<AzFloat>,
        );
    }
    fn finish(&self) -> Path {
        Path::Azure(self.finish())
    }
}

/// A wrapper around a stored PathBuilder and an optional transformation that should be
/// applied to any points to ensure they are in the matching device space.
struct PathBuilderRef<'a> {
    builder: &'a Box<GenericPathBuilder>,
    transform: Transform2D<AzFloat>,
}

impl<'a> PathBuilderRef<'a> {
    fn line_to(&self, pt: &Point2D<AzFloat>) {
        let pt = self.transform.transform_point(pt);
        self.builder.line_to(pt);
    }

    fn move_to(&self, pt: &Point2D<AzFloat>) {
        let pt = self.transform.transform_point(pt);
        self.builder.move_to(pt);
    }

    fn rect(&self, rect: &Rect<f32>) {
        let (first, second, third, fourth) = (
            Point2D::new(rect.origin.x, rect.origin.y),
            Point2D::new(rect.origin.x + rect.size.width, rect.origin.y),
            Point2D::new(
                rect.origin.x + rect.size.width,
                rect.origin.y + rect.size.height,
            ),
            Point2D::new(rect.origin.x, rect.origin.y + rect.size.height),
        );
        self.builder.move_to(self.transform.transform_point(&first));
        self.builder
            .line_to(self.transform.transform_point(&second));
        self.builder.line_to(self.transform.transform_point(&third));
        self.builder
            .line_to(self.transform.transform_point(&fourth));
        self.builder.close();
    }

    fn quadratic_curve_to(&self, cp: &Point2D<AzFloat>, endpoint: &Point2D<AzFloat>) {
        self.builder.quadratic_curve_to(
            &self.transform.transform_point(cp),
            &self.transform.transform_point(endpoint),
        )
    }

    fn bezier_curve_to(
        &self,
        cp1: &Point2D<AzFloat>,
        cp2: &Point2D<AzFloat>,
        endpoint: &Point2D<AzFloat>,
    ) {
        self.builder.bezier_curve_to(
            &self.transform.transform_point(cp1),
            &self.transform.transform_point(cp2),
            &self.transform.transform_point(endpoint),
        )
    }

    fn arc(
        &self,
        center: &Point2D<AzFloat>,
        radius: AzFloat,
        start_angle: AzFloat,
        end_angle: AzFloat,
        ccw: bool,
    ) {
        let center = self.transform.transform_point(center);
        self.builder
            .arc(center, radius, start_angle, end_angle, ccw);
    }

    pub fn ellipse(
        &self,
        center: &Point2D<AzFloat>,
        radius_x: AzFloat,
        radius_y: AzFloat,
        rotation_angle: AzFloat,
        start_angle: AzFloat,
        end_angle: AzFloat,
        ccw: bool,
    ) {
        let center = self.transform.transform_point(center);
        self.builder.ellipse(
            center,
            radius_x,
            radius_y,
            rotation_angle,
            start_angle,
            end_angle,
            ccw,
        );
    }

    fn current_point(&self) -> Option<Point2D<AzFloat>> {
        let inverse = match self.transform.inverse() {
            Some(i) => i,
            None => return None,
        };
        let current_point = self.builder.get_current_point();
        Some(inverse.transform_point(&Point2D::new(current_point.x, current_point.y)))
    }
}

// TODO(pylbrecht)
// This defines required methods for DrawTarget of azure and raqote
// The prototypes are derived from azure's methods.
trait GenericDrawTarget {
    fn clear_rect(&self, rect: &Rect<f32>);
    fn copy_surface(&self, surface: SourceSurface, source: Rect<i32>, destination: Point2D<i32>);
    fn create_gradient_stops(
        &self,
        gradient_stops: Vec<GradientStop>,
        extend_mode: ExtendMode,
    ) -> GradientStops;
    fn create_path_builder(&self) -> Box<GenericPathBuilder>;
    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
        format: SurfaceFormat,
    ) -> Box<GenericDrawTarget>;
    fn create_source_surface_from_data(
        &self,
        data: &[u8],
        size: Size2D<i32>,
        stride: i32,
        format: SurfaceFormat,
    ) -> Option<SourceSurface>;
    fn draw_surface(
        &self,
        surface: SourceSurface,
        dest: Rect<f32>,
        source: Rect<f32>,
        surf_options: DrawSurfaceOptions,
        options: DrawOptions,
    );
    fn draw_surface_with_shadow(
        &self,
        surface: SourceSurface,
        dest: &Point2D<f32>,
        color: &Color,
        offset: &Vector2D<f32>,
        sigma: f32,
        operator: CompositionOp,
    );
    fn fill(&self, path: &Path, pattern: Pattern, draw_options: &DrawOptions);
    fn fill_rect(&self, rect: &Rect<f32>, pattern: Pattern, draw_options: Option<&DrawOptions>);
    fn get_format(&self) -> SurfaceFormat;
    fn get_size(&self) -> IntSize;
    fn get_transform(&self) -> Transform2D<f32>;
    fn pop_clip(&self);
    fn push_clip(&self, path: &Path);
    fn set_transform(&self, matrix: &Transform2D<f32>);
    fn snapshot(&self) -> SourceSurface;
    fn stroke(
        &self,
        path: &Path,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    );
    fn stroke_line(
        &self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    );
    fn stroke_rect(
        &self,
        rect: &Rect<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    );
}

impl GenericDrawTarget for azure_hl::DrawTarget {
    fn clear_rect(&self, rect: &Rect<f32>) {
        self.clear_rect(rect as &Rect<AzFloat>);
    }

    fn copy_surface(&self, surface: SourceSurface, source: Rect<i32>, destination: Point2D<i32>) {
        self.copy_surface(surface.into_azure(), source, destination);
    }

    fn create_gradient_stops(
        &self,
        gradient_stops: Vec<GradientStop>,
        extend_mode: ExtendMode,
    ) -> GradientStops {
        let gradient_stops: Vec<AzGradientStop> =
            gradient_stops.into_iter().map(|x| x.into_azure()).collect();
        GradientStops::Azure(self.create_gradient_stops(&gradient_stops, extend_mode.into_azure()))
    }

    fn create_path_builder(&self) -> Box<GenericPathBuilder> {
        Box::new(self.create_path_builder())
    }

    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
        format: SurfaceFormat,
    ) -> Box<GenericDrawTarget> {
        Box::new(self.create_similar_draw_target(size, format.into_azure()))
    }
    fn create_source_surface_from_data(
        &self,
        data: &[u8],
        size: Size2D<i32>,
        stride: i32,
        format: SurfaceFormat,
    ) -> Option<SourceSurface> {
        self.create_source_surface_from_data(data, size, stride, format.into_azure()).map(|s| SourceSurface::Azure(s))
    }
    fn draw_surface(
        &self,
        surface: SourceSurface,
        dest: Rect<f32>,
        source: Rect<f32>,
        surf_options: DrawSurfaceOptions,
        options: DrawOptions,
    ) {
        self.draw_surface(
            surface.into_azure(),
            dest as Rect<AzFloat>,
            source as Rect<AzFloat>,
            surf_options.into_azure(),
            options.into_azure(),
        );
    }
    fn draw_surface_with_shadow(
        &self,
        surface: SourceSurface,
        dest: &Point2D<f32>,
        color: &Color,
        offset: &Vector2D<f32>,
        sigma: f32,
        operator: CompositionOp,
    ) {
        self.draw_surface_with_shadow(
            surface.into_azure(),
            dest as &Point2D<AzFloat>,
            color.as_azure(),
            offset as &Vector2D<AzFloat>,
            sigma as AzFloat,
            operator.into_azure(),
        );
    }
    fn fill(&self, path: &Path, pattern: Pattern, draw_options: &DrawOptions) {
        self.fill(
            path.as_azure(),
            pattern.as_azure().to_pattern_ref(),
            draw_options.as_azure(),
        );
    }
    fn fill_rect(&self, rect: &Rect<f32>, pattern: Pattern, draw_options: Option<&DrawOptions>) {
        self.fill_rect(
            rect as &Rect<AzFloat>,
            pattern.as_azure().to_pattern_ref(),
            draw_options.map(|x| x.as_azure()),
        );
    }
    fn get_format(&self) -> SurfaceFormat {
        SurfaceFormat::Azure(self.get_format())
    }
    fn get_size(&self) -> IntSize {
        IntSize::Azure(self.get_size())
    }
    fn get_transform(&self) -> Transform2D<f32> {
        self.get_transform() as Transform2D<f32>
    }
    fn pop_clip(&self) {
        self.pop_clip();
    }
    fn push_clip(&self, path: &Path) {
        self.push_clip(path.as_azure());
    }
    fn set_transform(&self, matrix: &Transform2D<f32>) {
        self.set_transform(matrix as &Transform2D<AzFloat>);
    }
    fn snapshot(&self) -> SourceSurface {
        SourceSurface::Azure(self.snapshot())
    }
    fn stroke(
        &self,
        path: &Path,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        self.stroke(
            path.as_azure(),
            pattern.as_azure().to_pattern_ref(),
            stroke_options.as_azure(),
            draw_options.as_azure(),
        );
    }
    fn stroke_line(
        &self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        self.stroke_line(
            start as Point2D<AzFloat>,
            end as Point2D<AzFloat>,
            pattern.as_azure().to_pattern_ref(),
            stroke_options.as_azure(),
            draw_options.as_azure(),
        );
    }
    fn stroke_rect(
        &self,
        rect: &Rect<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        self.stroke_rect(
            rect as &Rect<AzFloat>,
            pattern.as_azure().to_pattern_ref(),
            stroke_options.as_azure(),
            draw_options.as_azure(),
        );
    }
}

#[derive(Clone)]
enum ExtendMode {
    Azure(azure_hl::ExtendMode),
    Raqote(()),
}

impl ExtendMode {
    fn into_azure(self) -> azure_hl::ExtendMode {
        match self {
            ExtendMode::Azure(m) => m,
            _ => unreachable!(),
        }
    }
}

enum GradientStop {
    Azure(AzGradientStop),
    Raqote(()),
}

impl GradientStop {
    fn into_azure(self) -> AzGradientStop {
        match self {
            GradientStop::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

enum GradientStops {
    Azure(azure_hl::GradientStops),
    Raqote(()),
}

impl GradientStops {
    fn into_azure(self) -> azure_hl::GradientStops {
        match self {
            GradientStops::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub enum Color {
    Azure(azure_hl::Color),
    Raqote(()),
}

impl Color {
    fn as_azure(&self) -> &azure_hl::Color {
        match self {
            Color::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum CompositionOp {
    Azure(azure_hl::CompositionOp),
    Raqote(()),
}

impl CompositionOp {
    fn into_azure(self) -> azure_hl::CompositionOp {
        match self {
            CompositionOp::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

enum SurfaceFormat {
    Azure(azure_hl::SurfaceFormat),
    Raqote(()),
}

impl SurfaceFormat {
    fn into_azure(self) -> azure_hl::SurfaceFormat {
        match self {
            SurfaceFormat::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum SourceSurface {
    Azure(azure_hl::SourceSurface),
    Raqote(()),
}

impl SourceSurface {
    fn into_azure(self) -> azure_hl::SourceSurface {
        match self {
            SourceSurface::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

enum IntSize {
    Azure(AzIntSize),
    Raqote(()),
}

impl IntSize {
    fn into_azure(self) -> AzIntSize {
        match self {
            IntSize::Azure(s) => s,
            _ => unreachable!(),
        }
    }
}

enum Path {
    Azure(azure_hl::Path),
    Raqote(()),
}

impl Path {
    fn as_azure(&self) -> &azure_hl::Path {
        match self {
            Path::Azure(p) => p,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum Pattern {
    Azure(azure_hl::Pattern),
    Raqote(()),
}

impl Pattern {
    fn as_azure(&self) -> &azure_hl::Pattern {
        match self {
            Pattern::Azure(p) => p,
            _ => unreachable!(),
        }
    }
}

enum DrawSurfaceOptions {
    Azure(azure_hl::DrawSurfaceOptions),
    Raqote(()),
}

impl DrawSurfaceOptions {
    fn into_azure(self) -> azure_hl::DrawSurfaceOptions {
        match self {
            DrawSurfaceOptions::Azure(options) => options,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum DrawOptions {
    Azure(azure_hl::DrawOptions),
    Raqote(()),
}

impl DrawOptions {
    fn as_azure(&self) -> &azure_hl::DrawOptions {
        match self {
            DrawOptions::Azure(options) => options,
            _ => unreachable!(),
        }
    }
    fn as_azure_mut(&mut self) -> &mut azure_hl::DrawOptions {
        match self {
            DrawOptions::Azure(options) => options,
            _ => unreachable!(),
        }
    }
    fn into_azure(self) -> azure_hl::DrawOptions {
        match self {
            DrawOptions::Azure(options) => options,
            _ => unreachable!(),
        }
    }
    fn set_alpha(&mut self, val: f32) {
        match self {
            DrawOptions::Azure(options) => options.alpha = val as AzFloat,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum StrokeOptions<'a> {
    Azure(azure_hl::StrokeOptions<'a>),
    Raqote(()),
}

impl<'a> StrokeOptions<'a> {
    fn as_azure(&self) -> &azure_hl::StrokeOptions<'a> {
        match self {
            StrokeOptions::Azure(options) => options,
            _ => unreachable!(),
        }
    }
    fn set_line_width(&mut self, val: f32) {
        match self {
            StrokeOptions::Azure(options) => options.line_width = val as AzFloat,
            _ => unreachable!(),
        }
    }
    fn set_miter_limit(&mut self, val: f32) {
        match self {
            StrokeOptions::Azure(options) => options.miter_limit = val as AzFloat,
            _ => unreachable!(),
        }
    }
    fn set_line_join(&mut self, val: LineJoinStyle) {
        match self {
            StrokeOptions::Azure(options) => options.line_join = val.to_azure_style(),
            _ => unreachable!(),
        }
    }
    fn set_line_cap(&mut self, val: LineCapStyle) {
        match self {
            StrokeOptions::Azure(options) => options.line_cap = val.to_azure_style(),
            _ => unreachable!(),
        }
    }
}

pub struct CanvasData<'a> {
    drawtarget: Box<GenericDrawTarget>,
    path_state: Option<PathState>,
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
        size: Size2D<u64>,
        webrender_api_sender: webrender_api::RenderApiSender,
        antialias: AntialiasMode,
        canvas_id: CanvasId,
    ) -> CanvasData<'a> {
        let draw_target = CanvasData::create(size);
        let webrender_api = webrender_api_sender.create_api();
        CanvasData {
            drawtarget: draw_target,
            path_state: None,
            state: CanvasPaintState::new(antialias, CanvasBackend::Azure),
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


        // TODO(pylbrecht) create another clousure for raqote
        let writer = |draw_target: &GenericDrawTarget| {
            write_image(
                draw_target,
                image_data,
                source_rect.size,
                dest_rect,
                smoothing_enabled,
                CompositionOp::Azure(self.state.draw_options.as_azure().composition),
                self.state.draw_options.as_azure().alpha,
            );
        };

        if self.need_to_draw_shadow() {
            let rect = Rect::new(
                Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32),
            );

            // TODO(pylbrecht) pass another closure for raqote
            self.draw_with_shadow(&rect, writer);
        } else {
            writer(&*self.drawtarget);
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
            match &self.state.fill_style {
                Pattern::Azure(pattern) => {
                    match pattern {
                        azure_hl::Pattern::Surface(ref surface) => {
                            let surface_size = surface.size();
                            match (surface.repeat_x, surface.repeat_y) {
                                (true, true) => rect.size,
                                (true, false) => Size2D::new(rect.size.width, surface_size.height as f32),
                                (false, true) => Size2D::new(surface_size.width as f32, rect.size.height),
                                (false, false) => {
                                    Size2D::new(surface_size.width as f32, surface_size.height as f32)
                                },
                            }
                        }
                        _ => rect.size,
                    }
                },
                _ => unreachable!(),
            }
        );

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&draw_rect, |new_draw_target: &GenericDrawTarget| {
                new_draw_target.fill_rect(
                    &draw_rect,
                    self.state.fill_style.clone(),
                    Some(&self.state.draw_options),
                );
            });
        } else {
            self.drawtarget.fill_rect(
                &draw_rect,
                self.state.fill_style.clone(),
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
            self.draw_with_shadow(&rect, |new_draw_target: &GenericDrawTarget| {
                new_draw_target.stroke_rect(
                    rect,
                    self.state.stroke_style.clone(),
                    &self.state.stroke_opts,
                    &self.state.draw_options,
                );
            });
        } else if rect.size.width == 0. || rect.size.height == 0. {
            let cap = match self.state.stroke_opts.as_azure().line_join {
                JoinStyle::Round => CapStyle::Round,
                _ => CapStyle::Butt,
            };

            let stroke_opts = StrokeOptions::Azure(azure_hl::StrokeOptions::new(
                self.state.stroke_opts.as_azure().line_width,
                self.state.stroke_opts.as_azure().line_join,
                cap,
                self.state.stroke_opts.as_azure().miter_limit,
                self.state.stroke_opts.as_azure().mDashPattern,
            ));
            self.drawtarget.stroke_line(
                rect.origin,
                rect.bottom_right(),
                self.state.stroke_style.clone(),
                &stroke_opts,
                &self.state.draw_options,
            );
        } else {
            self.drawtarget.stroke_rect(
                rect,
                self.state.stroke_style.clone(),
                &self.state.stroke_opts,
                &self.state.draw_options,
            );
        }
    }

    pub fn begin_path(&mut self) {
        // Erase any traces of previous paths that existed before this.
        self.path_state = None;
    }

    pub fn close_path(&mut self) {
        self.path_builder().builder.close();
    }

    fn ensure_path(&mut self) {
        // If there's no record of any path yet, create a new builder in user-space.
        if self.path_state.is_none() {
            self.path_state = Some(PathState::UserSpacePathBuilder(
                self.drawtarget.create_path_builder(),
                None,
            ));
        }

        // If a user-space builder exists, create a finished path from it.
        let new_state = match *self.path_state.as_mut().unwrap() {
            PathState::UserSpacePathBuilder(ref builder, ref mut transform) => {
                Some((builder.finish(), transform.take()))
            },
            PathState::DeviceSpacePathBuilder(..) | PathState::UserSpacePath(..) => None,
        };
        if let Some((path, transform)) = new_state {
            self.path_state = Some(PathState::UserSpacePath(path, transform));
        }

        // If a user-space path exists, create a device-space builder based on it if
        // any transform is present.
        let new_state = match *self.path_state.as_ref().unwrap() {
            PathState::UserSpacePath(ref path, Some(ref transform)) => {
                Some(Box::new(path.as_azure().transformed_copy_to_builder(transform)))
            },
            PathState::UserSpacePath(..) |
            PathState::UserSpacePathBuilder(..) |
            PathState::DeviceSpacePathBuilder(..) => None,
        };
        if let Some(builder) = new_state {
            self.path_state = Some(PathState::DeviceSpacePathBuilder(builder));
        }

        // If a device-space builder is present, create a user-space path from its
        // finished path by inverting the initial transformation.
        let new_state = match self.path_state.as_ref().unwrap() {
            PathState::DeviceSpacePathBuilder(ref builder) => {
                let path = builder.finish();
                let inverse = match self.drawtarget.get_transform().inverse() {
                    Some(m) => m,
                    None => {
                        warn!("Couldn't invert canvas transformation.");
                        return;
                    },
                };
                let builder = Box::new(path.as_azure().transformed_copy_to_builder(&inverse));
                Some(Path::Azure(builder.finish()))
            },
            PathState::UserSpacePathBuilder(..) | PathState::UserSpacePath(..) => None,
        };
        if let Some(path) = new_state {
            self.path_state = Some(PathState::UserSpacePath(path, None));
        }

        assert!(self.path_state.as_ref().unwrap().is_path())
    }

    fn path(&self) -> &Path {
        self.path_state
            .as_ref()
            .expect("Should have called ensure_path()")
            .path()
    }

    pub fn fill(&mut self) {
        if is_zero_size_gradient(&self.state.fill_style) {
            return; // Paint nothing if gradient size is zero.
        }

        self.ensure_path();
        self.drawtarget.fill(
            &self.path(),
            self.state.fill_style.clone(),
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
            self.state.stroke_style.clone(),
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
        let result = match self.path_state.as_ref() {
            Some(PathState::UserSpacePath(ref path, ref transform)) => {
                let target_transform = self.drawtarget.get_transform();
                let path_transform = transform.as_ref().unwrap_or(&target_transform);
                path.as_azure().contains_point(x, y, path_transform)
            },
            Some(_) | None => false,
        };
        chan.send(result).unwrap();
    }

    pub fn move_to(&mut self, point: &Point2D<AzFloat>) {
        self.path_builder().move_to(point);
    }

    pub fn line_to(&mut self, point: &Point2D<AzFloat>) {
        self.path_builder().line_to(point);
    }

    fn path_builder(&mut self) -> PathBuilderRef {
        if self.path_state.is_none() {
            self.path_state = Some(PathState::UserSpacePathBuilder(
                self.drawtarget.create_path_builder(),
                None,
            ));
        }

        // Rust is not pleased by returning a reference to a builder in some branches
        // and overwriting path_state in other ones. The following awkward use of duplicate
        // matches works around the resulting borrow errors.
        let new_state = {
            match self.path_state.as_ref().unwrap() {
                &PathState::UserSpacePathBuilder(_, None) |
                &PathState::DeviceSpacePathBuilder(_) => None,
                &PathState::UserSpacePathBuilder(ref builder, Some(ref transform)) => {
                    let path = builder.finish();
                    Some(PathState::DeviceSpacePathBuilder(
                        Box::new(path.as_azure().transformed_copy_to_builder(transform)),
                    ))
                },
                &PathState::UserSpacePath(ref path, Some(ref transform)) => Some(
                    PathState::DeviceSpacePathBuilder(Box::new(path.as_azure().transformed_copy_to_builder(transform))),
                ),
                &PathState::UserSpacePath(ref path, None) => Some(PathState::UserSpacePathBuilder(
                    Box::new(path.as_azure().copy_to_builder()),
                    None,
                )),
            }
        };
        match new_state {
            // There's a new builder value that needs to be stored.
            Some(state) => self.path_state = Some(state),
            // There's an existing builder value that can be returned immediately.
            None => match self.path_state.as_ref().unwrap() {
                &PathState::UserSpacePathBuilder(ref builder, None) => {
                    return PathBuilderRef {
                        builder,
                        transform: Transform2D::identity(),
                    };
                },
                &PathState::DeviceSpacePathBuilder(ref builder) => {
                    return PathBuilderRef {
                        builder,
                        transform: self.drawtarget.get_transform(),
                    };
                },
                _ => unreachable!(),
            },
        }

        match self.path_state.as_ref().unwrap() {
            &PathState::UserSpacePathBuilder(ref builder, None) => PathBuilderRef {
                builder,
                transform: Transform2D::identity(),
            },
            &PathState::DeviceSpacePathBuilder(ref builder) => PathBuilderRef {
                builder,
                transform: self.drawtarget.get_transform(),
            },
            &PathState::UserSpacePathBuilder(..) | &PathState::UserSpacePath(..) => unreachable!(),
        }
    }

    pub fn rect(&mut self, rect: &Rect<f32>) {
        self.path_builder().rect(rect);
    }

    pub fn quadratic_curve_to(&mut self, cp: &Point2D<AzFloat>, endpoint: &Point2D<AzFloat>) {
        self.path_builder().quadratic_curve_to(cp, endpoint);
    }

    pub fn bezier_curve_to(
        &mut self,
        cp1: &Point2D<AzFloat>,
        cp2: &Point2D<AzFloat>,
        endpoint: &Point2D<AzFloat>,
    ) {
        self.path_builder().bezier_curve_to(cp1, cp2, endpoint);
    }

    pub fn arc(
        &mut self,
        center: &Point2D<AzFloat>,
        radius: AzFloat,
        start_angle: AzFloat,
        end_angle: AzFloat,
        ccw: bool,
    ) {
        self.path_builder()
            .arc(center, radius, start_angle, end_angle, ccw);
    }

    pub fn arc_to(&mut self, cp1: &Point2D<AzFloat>, cp2: &Point2D<AzFloat>, radius: AzFloat) {
        let cp0 = match self.path_builder().current_point() {
            Some(p) => p.as_azure_point(),
            None => return,
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
        self.path_builder().ellipse(
            center,
            radius_x,
            radius_y,
            rotation_angle,
            start_angle,
            end_angle,
            ccw,
        );
    }

    pub fn set_fill_style(&mut self, style: FillOrStrokeStyle) {
        if let Some(pattern) = style.to_azure_pattern(&*self.drawtarget) {
            self.state.fill_style = Pattern::Azure(pattern)
        }
    }

    pub fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        if let Some(pattern) = style.to_azure_pattern(&*self.drawtarget) {
            self.state.stroke_style = Pattern::Azure(pattern)
        }
    }

    pub fn set_line_width(&mut self, width: f32) {
        self.state.stroke_opts.set_line_width(width);
    }

    pub fn set_line_cap(&mut self, cap: LineCapStyle) {
        self.state.stroke_opts.set_line_cap(cap);
    }

    pub fn set_line_join(&mut self, join: LineJoinStyle) {
        self.state.stroke_opts.set_line_join(join);
    }

    pub fn set_miter_limit(&mut self, limit: f32) {
        self.state.stroke_opts.set_miter_limit(limit);
    }

    pub fn set_transform(&mut self, transform: &Transform2D<f32>) {
        // If there is an in-progress path, store the existing transformation required
        // to move between device and user space.
        match self.path_state.as_mut() {
            None | Some(PathState::DeviceSpacePathBuilder(..)) => (),
            Some(PathState::UserSpacePathBuilder(_, ref mut transform)) |
            Some(PathState::UserSpacePath(_, ref mut transform)) => {
                if transform.is_none() {
                    *transform = Some(self.drawtarget.get_transform());
                }
            },
        }
        self.state.transform = transform.clone();
        self.drawtarget.set_transform(transform)
    }

    pub fn set_global_alpha(&mut self, alpha: f32) {
        self.state.draw_options.set_alpha(alpha);
    }

    pub fn set_global_composition(&mut self, op: CompositionOrBlending) {
        self.state
            .draw_options
            .as_azure_mut()
            .set_composition_op(op.to_azure_style());
    }

    pub fn create(size: Size2D<u64>) -> Box<GenericDrawTarget> {
        // FIXME(nox): Why is the size made of i32 values?
        Box::new(DrawTarget::new(
            BackendType::Skia,
            size.to_i32(),
            azure_hl::SurfaceFormat::B8G8R8A8,
        ))
    }

    pub fn recreate(&mut self, size: Size2D<u32>) {
        self.drawtarget = CanvasData::create(Size2D::new(size.width as u64, size.height as u64));
        self.state = CanvasPaintState::new(self.state.draw_options.as_azure().antialias, CanvasBackend::Azure);
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
            self.drawtarget.snapshot().into_azure().get_data_surface().data()
        });
        chan.send(data).unwrap();
    }

    #[allow(unsafe_code)]
    pub fn send_data(&mut self, chan: IpcSender<CanvasImageData>) {
        let size = self.drawtarget.get_size().into_azure();

        let descriptor = webrender_api::ImageDescriptor {
            size: webrender_api::DeviceIntSize::new(size.width, size.height),
            stride: None,
            format: webrender_api::ImageFormat::BGRA8,
            offset: 0,
            is_opaque: false,
            allow_mipmaps: false,
        };
        let data = webrender_api::ImageData::Raw(Arc::new(unsafe {
            self.drawtarget.snapshot().into_azure().get_data_surface().data().into()
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
                SurfaceFormat::Azure(azure_hl::SurfaceFormat::B8G8R8A8),
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
        self.state.shadow_color.as_azure().a != 0.0f32 &&
            (self.state.shadow_offset_x != 0.0f64 ||
                self.state.shadow_offset_y != 0.0f64 ||
                self.state.shadow_blur != 0.0f64)
    }

    fn create_draw_target_for_shadow(&self, source_rect: &Rect<f32>) -> Box<GenericDrawTarget> {
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
        F: FnOnce(&GenericDrawTarget),
    {
        let shadow_src_rect = self.state.transform.transform_rect(rect);
        let new_draw_target = self.create_draw_target_for_shadow(&shadow_src_rect);
        draw_shadow_source(&*new_draw_target);
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
            CompositionOp::Azure(self.state.draw_options.as_azure().composition),
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

        let data_surface = self.drawtarget.snapshot().into_azure().get_data_surface();
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

enum CanvasBackend {
    Azure,
    Raqote,
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
    fn new(antialias: AntialiasMode, backend: CanvasBackend) -> CanvasPaintState<'a> {
        match backend {
            CanvasBackend::Azure => CanvasPaintState {
                draw_options: DrawOptions::Azure(azure_hl::DrawOptions::new(1.0, azure_hl::CompositionOp::Over, antialias)),
                fill_style: Pattern::Azure(azure_hl::Pattern::Color(ColorPattern::new(azure_hl::Color::black()))),
                stroke_style: Pattern::Azure(azure_hl::Pattern::Color(ColorPattern::new(azure_hl::Color::black()))),
                stroke_opts: StrokeOptions::Azure(azure_hl::StrokeOptions::new(
                    1.0,
                    JoinStyle::MiterOrBevel,
                    CapStyle::Butt,
                    10.0,
                    &[],
                    )),
                    transform: Transform2D::identity(),
                    shadow_offset_x: 0.0,
                    shadow_offset_y: 0.0,
                    shadow_blur: 0.0,
                    shadow_color: Color::Azure(azure_hl::Color::transparent()),
            },
            CanvasBackend::Raqote => CanvasPaintState {
                draw_options: DrawOptions::Raqote(()),
                fill_style: Pattern::Raqote(()),
                stroke_style: Pattern::Raqote(()),
                stroke_opts: StrokeOptions::Raqote(()),
                transform: Transform2D::identity(),
                shadow_offset_x: 0.0,
                shadow_offset_y: 0.0,
                shadow_blur: 0.0,
                shadow_color: Color::Raqote(()),
            }
        }
    }
}

fn is_zero_size_gradient(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Azure(ref az_pattern) => {
            if let azure_hl::Pattern::LinearGradient(ref gradient) = az_pattern {
                if gradient.is_zero_size() {
                    return true;
                }
            }
            false
        },
        _ => unreachable!(),
    }
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
fn write_image(
    draw_target: &GenericDrawTarget,
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
            SurfaceFormat::Azure(azure_hl::SurfaceFormat::B8G8R8A8),
            )
        .unwrap();
    let draw_surface_options = DrawSurfaceOptions::Azure(azure_hl::DrawSurfaceOptions::new(filter, true));
    let draw_options =
        DrawOptions::Azure(azure_hl::DrawOptions::new(global_alpha, composition_op.into_azure(), AntialiasMode::None));
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
    type Target = azure_hl::CompositionOp;

    fn to_azure_style(self) -> azure_hl::CompositionOp {
        match self {
            CompositionStyle::SrcIn => azure_hl::CompositionOp::In,
            CompositionStyle::SrcOut => azure_hl::CompositionOp::Out,
            CompositionStyle::SrcOver => azure_hl::CompositionOp::Over,
            CompositionStyle::SrcAtop => azure_hl::CompositionOp::Atop,
            CompositionStyle::DestIn => azure_hl::CompositionOp::DestIn,
            CompositionStyle::DestOut => azure_hl::CompositionOp::DestOut,
            CompositionStyle::DestOver => azure_hl::CompositionOp::DestOver,
            CompositionStyle::DestAtop => azure_hl::CompositionOp::DestAtop,
            CompositionStyle::Copy => azure_hl::CompositionOp::Source,
            CompositionStyle::Lighter => azure_hl::CompositionOp::Add,
            CompositionStyle::Xor => azure_hl::CompositionOp::Xor,
        }
    }
}

impl ToAzureStyle for BlendingStyle {
    type Target = azure_hl::CompositionOp;

    fn to_azure_style(self) -> azure_hl::CompositionOp {
        match self {
            BlendingStyle::Multiply => azure_hl::CompositionOp::Multiply,
            BlendingStyle::Screen => azure_hl::CompositionOp::Screen,
            BlendingStyle::Overlay => azure_hl::CompositionOp::Overlay,
            BlendingStyle::Darken => azure_hl::CompositionOp::Darken,
            BlendingStyle::Lighten => azure_hl::CompositionOp::Lighten,
            BlendingStyle::ColorDodge => azure_hl::CompositionOp::ColorDodge,
            BlendingStyle::ColorBurn => azure_hl::CompositionOp::ColorBurn,
            BlendingStyle::HardLight => azure_hl::CompositionOp::HardLight,
            BlendingStyle::SoftLight => azure_hl::CompositionOp::SoftLight,
            BlendingStyle::Difference => azure_hl::CompositionOp::Difference,
            BlendingStyle::Exclusion => azure_hl::CompositionOp::Exclusion,
            BlendingStyle::Hue => azure_hl::CompositionOp::Hue,
            BlendingStyle::Saturation => azure_hl::CompositionOp::Saturation,
            BlendingStyle::Color => azure_hl::CompositionOp::Color,
            BlendingStyle::Luminosity => azure_hl::CompositionOp::Luminosity,
        }
    }
}

impl ToAzureStyle for CompositionOrBlending {
    type Target = azure_hl::CompositionOp;

    fn to_azure_style(self) -> azure_hl::CompositionOp {
        match self {
            CompositionOrBlending::Composition(op) => op.to_azure_style(),
            CompositionOrBlending::Blending(op) => op.to_azure_style(),
        }
    }
}

pub trait ToAzurePattern {
    fn to_azure_pattern(&self, drawtarget: &GenericDrawTarget) -> Option<azure_hl::Pattern>;
}

impl ToAzurePattern for FillOrStrokeStyle {
    fn to_azure_pattern(&self, drawtarget: &GenericDrawTarget) -> Option<azure_hl::Pattern> {
        Some(match *self {
            FillOrStrokeStyle::Color(ref color) => {
                azure_hl::Pattern::Color(ColorPattern::new(color.to_azure_style()))
            },
            FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = linear_gradient_style
                    .stops
                    .iter()
                    .map(|s| GradientStop::Azure(azure_hl::GradientStop {
                        offset: s.offset as AzFloat,
                        color: s.color.to_azure_style(),
                    }))
                    .collect();

                azure_hl::Pattern::LinearGradient(LinearGradientPattern::new(
                    &Point2D::new(
                        linear_gradient_style.x0 as AzFloat,
                        linear_gradient_style.y0 as AzFloat,
                    ),
                    &Point2D::new(
                        linear_gradient_style.x1 as AzFloat,
                        linear_gradient_style.y1 as AzFloat,
                    ),
                    drawtarget.create_gradient_stops(gradient_stops, ExtendMode::Azure(azure_hl::ExtendMode::Clamp)).into_azure(),
                    &Transform2D::identity(),
                ))
            },
            FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = radial_gradient_style
                    .stops
                    .iter()
                    .map(|s| GradientStop::Azure(azure_hl::GradientStop {
                        offset: s.offset as AzFloat,
                        color: s.color.to_azure_style(),
                    }))
                    .collect();

                azure_hl::Pattern::RadialGradient(RadialGradientPattern::new(
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
                    drawtarget.create_gradient_stops(gradient_stops, ExtendMode::Azure(azure_hl::ExtendMode::Clamp)).into_azure(),
                    &Transform2D::identity(),
                ))
            },
            FillOrStrokeStyle::Surface(ref surface_style) => {
                let source_surface = drawtarget.create_source_surface_from_data(
                    &surface_style.surface_data,
                    // FIXME(nox): Why are those i32 values?
                    surface_style.surface_size.to_i32(),
                    surface_style.surface_size.width as i32 * 4,
                    SurfaceFormat::Azure(azure_hl::SurfaceFormat::B8G8R8A8),
                )?.into_azure();
                azure_hl::Pattern::Surface(SurfacePattern::new(
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
    type Target = azure_hl::Color;

    fn to_azure_style(self) -> azure_hl::Color {
        azure_hl::Color::rgba(
            self.red_f32() as AzFloat,
            self.green_f32() as AzFloat,
            self.blue_f32() as AzFloat,
            self.alpha_f32() as AzFloat,
        )
    }
}
