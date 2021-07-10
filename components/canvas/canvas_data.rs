/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::canvas_paint_thread::{AntialiasMode, ImageUpdate, WebrenderApi};
use crate::raqote_backend::Repetition;
use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use euclid::{point2, vec2};
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::metrics::Metrics;
use font_kit::properties::{Properties, Stretch, Style, Weight};
use font_kit::source::SystemSource;
use gfx::font::FontHandleMethods;
use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use ipc_channel::ipc::{IpcSender, IpcSharedMemory};
use num_traits::ToPrimitive;
use servo_arc::Arc as ServoArc;
use std::cell::RefCell;
#[allow(unused_imports)]
use std::marker::PhantomData;
use std::mem;
use std::sync::{Arc, Mutex};
use style::properties::style_structs::Font as FontStyleStruct;
use style::values::computed::font;
use style_traits::values::ToCss;
use webrender_api::units::RectExt as RectExt_;

/// The canvas data stores a state machine for the current status of
/// the path data and any relevant transformations that are
/// applied to it. The Azure drawing API expects the path to be in
/// userspace. However, when a path is being built but the canvas'
/// transform changes, we choose to transform the path and perform
/// further operations to it in device space. When it's time to
/// draw the path, we convert it back to userspace and draw it
/// with the correct transform applied.
/// TODO: De-abstract now that Azure is removed?
enum PathState {
    /// Path builder in user-space. If a transform has been applied
    /// but no further path operations have occurred, it is stored
    /// in the optional field.
    UserSpacePathBuilder(Box<dyn GenericPathBuilder>, Option<Transform2D<f32>>),
    /// Path builder in device-space.
    DeviceSpacePathBuilder(Box<dyn GenericPathBuilder>),
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

pub trait Backend {
    fn get_composition_op(&self, opts: &DrawOptions) -> CompositionOp;
    fn need_to_draw_shadow(&self, color: &Color) -> bool;
    fn set_shadow_color<'a>(&mut self, color: RGBA, state: &mut CanvasPaintState<'a>);
    fn set_fill_style<'a>(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        drawtarget: &dyn GenericDrawTarget,
    );
    fn set_stroke_style<'a>(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        drawtarget: &dyn GenericDrawTarget,
    );
    fn set_global_composition<'a>(
        &mut self,
        op: CompositionOrBlending,
        state: &mut CanvasPaintState<'a>,
    );
    fn create_drawtarget(&self, size: Size2D<u64>) -> Box<dyn GenericDrawTarget>;
    fn recreate_paint_state<'a>(&self, state: &CanvasPaintState<'a>) -> CanvasPaintState<'a>;
}

/// A generic PathBuilder that abstracts the interface for azure's and raqote's PathBuilder.
/// TODO: De-abstract now that Azure is removed?
pub trait GenericPathBuilder {
    fn arc(
        &mut self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    );
    fn bezier_curve_to(
        &mut self,
        control_point1: &Point2D<f32>,
        control_point2: &Point2D<f32>,
        control_point3: &Point2D<f32>,
    );
    fn close(&mut self);
    fn ellipse(
        &mut self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    );
    fn get_current_point(&mut self) -> Option<Point2D<f32>>;
    fn line_to(&mut self, point: Point2D<f32>);
    fn move_to(&mut self, point: Point2D<f32>);
    fn quadratic_curve_to(&mut self, control_point: &Point2D<f32>, end_point: &Point2D<f32>);
    fn finish(&mut self) -> Path;
}

/// A wrapper around a stored PathBuilder and an optional transformation that should be
/// applied to any points to ensure they are in the matching device space.
struct PathBuilderRef<'a> {
    builder: &'a mut Box<dyn GenericPathBuilder>,
    transform: Transform2D<f32>,
}

impl<'a> PathBuilderRef<'a> {
    fn line_to(&mut self, pt: &Point2D<f32>) {
        let pt = self.transform.transform_point(*pt);
        self.builder.line_to(pt);
    }

    fn move_to(&mut self, pt: &Point2D<f32>) {
        let pt = self.transform.transform_point(*pt);
        self.builder.move_to(pt);
    }

    fn rect(&mut self, rect: &Rect<f32>) {
        let (first, second, third, fourth) = (
            Point2D::new(rect.origin.x, rect.origin.y),
            Point2D::new(rect.origin.x + rect.size.width, rect.origin.y),
            Point2D::new(
                rect.origin.x + rect.size.width,
                rect.origin.y + rect.size.height,
            ),
            Point2D::new(rect.origin.x, rect.origin.y + rect.size.height),
        );
        self.move_to(&first);
        self.line_to(&second);
        self.line_to(&third);
        self.line_to(&fourth);
        self.close();
        self.move_to(&first);
    }

    fn quadratic_curve_to(&mut self, cp: &Point2D<f32>, endpoint: &Point2D<f32>) {
        self.builder.quadratic_curve_to(
            &self.transform.transform_point(*cp),
            &self.transform.transform_point(*endpoint),
        )
    }

    fn bezier_curve_to(&mut self, cp1: &Point2D<f32>, cp2: &Point2D<f32>, endpoint: &Point2D<f32>) {
        self.builder.bezier_curve_to(
            &self.transform.transform_point(*cp1),
            &self.transform.transform_point(*cp2),
            &self.transform.transform_point(*endpoint),
        )
    }

    fn arc(
        &mut self,
        center: &Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        ccw: bool,
    ) {
        let center = self.transform.transform_point(*center);
        self.builder
            .arc(center, radius, start_angle, end_angle, ccw);
    }

    pub fn ellipse(
        &mut self,
        center: &Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        ccw: bool,
    ) {
        let center = self.transform.transform_point(*center);
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

    fn current_point(&mut self) -> Option<Point2D<f32>> {
        let inverse = match self.transform.inverse() {
            Some(i) => i,
            None => return None,
        };
        match self.builder.get_current_point() {
            Some(point) => Some(inverse.transform_point(Point2D::new(point.x, point.y))),
            None => None,
        }
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

// TODO(pylbrecht)
// This defines required methods for DrawTarget of azure and raqote
// The prototypes are derived from azure's methods.
// TODO: De-abstract now that Azure is removed?
pub trait GenericDrawTarget {
    fn clear_rect(&mut self, rect: &Rect<f32>);
    fn copy_surface(
        &mut self,
        surface: SourceSurface,
        source: Rect<i32>,
        destination: Point2D<i32>,
    );
    fn create_gradient_stops(
        &self,
        gradient_stops: Vec<GradientStop>,
        extend_mode: ExtendMode,
    ) -> GradientStops;
    fn create_path_builder(&self) -> Box<dyn GenericPathBuilder>;
    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
        format: SurfaceFormat,
    ) -> Box<dyn GenericDrawTarget>;
    fn create_source_surface_from_data(
        &self,
        data: &[u8],
        size: Size2D<i32>,
        stride: i32,
    ) -> Option<SourceSurface>;
    fn draw_surface(
        &mut self,
        surface: SourceSurface,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        draw_options: &DrawOptions,
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
    fn fill(&mut self, path: &Path, pattern: Pattern, draw_options: &DrawOptions);
    fn fill_text(
        &mut self,
        font: &Font,
        point_size: f32,
        text: &str,
        start: Point2D<f32>,
        pattern: &Pattern,
        draw_options: &DrawOptions,
    );
    fn fill_rect(&mut self, rect: &Rect<f32>, pattern: Pattern, draw_options: Option<&DrawOptions>);
    fn get_format(&self) -> SurfaceFormat;
    fn get_size(&self) -> Size2D<i32>;
    fn get_transform(&self) -> Transform2D<f32>;
    fn pop_clip(&mut self);
    fn push_clip(&mut self, path: &Path);
    fn set_transform(&mut self, matrix: &Transform2D<f32>);
    fn snapshot(&self) -> SourceSurface;
    fn stroke(
        &mut self,
        path: &Path,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    );
    fn stroke_line(
        &mut self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    );
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    );
    fn snapshot_data(&self, f: &dyn Fn(&[u8]) -> Vec<u8>) -> Vec<u8>;
    fn snapshot_data_owned(&self) -> Vec<u8>;
}

#[derive(Clone)]
pub enum ExtendMode {
    Raqote(()),
}

pub enum GradientStop {
    Raqote(raqote::GradientStop),
}

pub enum GradientStops {
    Raqote(Vec<raqote::GradientStop>),
}

#[derive(Clone)]
pub enum Color {
    Raqote(raqote::SolidSource),
}

#[derive(Clone)]
pub enum CompositionOp {
    Raqote(raqote::BlendMode),
}

pub enum SurfaceFormat {
    Raqote(()),
}

#[derive(Clone)]
pub enum SourceSurface {
    Raqote(Vec<u8>), // TODO: See if we can avoid the alloc (probably?)
}

#[derive(Clone)]
pub enum Path {
    Raqote(raqote::Path),
}

#[derive(Clone)]
pub enum Pattern<'a> {
    Raqote(crate::raqote_backend::Pattern<'a>),
}

pub enum DrawSurfaceOptions {
    Raqote(()),
}

#[derive(Clone)]
pub enum DrawOptions {
    Raqote(raqote::DrawOptions),
}

#[derive(Clone)]
pub enum StrokeOptions<'a> {
    Raqote(raqote::StrokeStyle, PhantomData<&'a ()>),
}

#[derive(Clone, Copy)]
pub enum Filter {
    Linear,
    Point,
}

pub(crate) type CanvasFontContext = FontContext<FontCacheThread>;

thread_local!(static FONT_CONTEXT: RefCell<Option<CanvasFontContext>> = RefCell::new(None));

pub(crate) fn with_thread_local_font_context<F, R>(canvas_data: &CanvasData, f: F) -> R
where
    F: FnOnce(&mut CanvasFontContext) -> R,
{
    FONT_CONTEXT.with(|font_context| {
        f(font_context.borrow_mut().get_or_insert_with(|| {
            FontContext::new(canvas_data.font_cache_thread.lock().unwrap().clone())
        }))
    })
}

pub struct CanvasData<'a> {
    backend: Box<dyn Backend>,
    drawtarget: Box<dyn GenericDrawTarget>,
    path_state: Option<PathState>,
    state: CanvasPaintState<'a>,
    saved_states: Vec<CanvasPaintState<'a>>,
    webrender_api: Box<dyn WebrenderApi>,
    image_key: Option<webrender_api::ImageKey>,
    /// An old webrender image key that can be deleted when the next epoch ends.
    old_image_key: Option<webrender_api::ImageKey>,
    /// An old webrender image key that can be deleted when the current epoch ends.
    very_old_image_key: Option<webrender_api::ImageKey>,
    font_cache_thread: Mutex<FontCacheThread>,
}

fn create_backend() -> Box<dyn Backend> {
    Box::new(crate::raqote_backend::RaqoteBackend)
}

impl<'a> CanvasData<'a> {
    pub fn new(
        size: Size2D<u64>,
        webrender_api: Box<dyn WebrenderApi>,
        antialias: AntialiasMode,
        font_cache_thread: FontCacheThread,
    ) -> CanvasData<'a> {
        let backend = create_backend();
        let draw_target = backend.create_drawtarget(size);
        CanvasData {
            backend,
            drawtarget: draw_target,
            path_state: None,
            state: CanvasPaintState::new(antialias),
            saved_states: vec![],
            webrender_api,
            image_key: None,
            old_image_key: None,
            very_old_image_key: None,
            font_cache_thread: Mutex::new(font_cache_thread),
        }
    }

    pub fn draw_image(
        &mut self,
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
            pixels::rgba8_get_rect(&image_data, image_size.to_u64(), source_rect.to_u64()).into()
        } else {
            image_data.into()
        };

        let draw_options = self.state.draw_options.clone();
        let writer = |draw_target: &mut dyn GenericDrawTarget| {
            write_image(
                draw_target,
                image_data,
                source_rect.size,
                dest_rect,
                smoothing_enabled,
                &draw_options,
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
            writer(&mut *self.drawtarget);
        }
    }

    pub fn save_context_state(&mut self) {
        self.saved_states.push(self.state.clone());
    }

    pub fn restore_context_state(&mut self) {
        if let Some(state) = self.saved_states.pop() {
            let _ = mem::replace(&mut self.state, state);
            self.drawtarget.set_transform(&self.state.transform);
            self.drawtarget.pop_clip();
        }
    }

    // https://html.spec.whatwg.org/multipage/#text-preparation-algorithm
    pub fn fill_text(
        &mut self,
        text: String,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        is_rtl: bool,
    ) {
        // Step 2.
        let text = replace_ascii_whitespace(text);

        // Step 3.
        let point_size = self
            .state
            .font_style
            .as_ref()
            .map_or(10., |style| style.font_size.size().px());
        let font_style = self.state.font_style.as_ref();
        let font = font_style.map_or_else(
            || load_system_font_from_style(None),
            |style| {
                with_thread_local_font_context(&self, |font_context| {
                    let font_group = font_context.font_group(ServoArc::new(style.clone()));
                    let font = font_group
                        .borrow_mut()
                        .first(font_context)
                        .expect("couldn't find font");
                    let font = font.borrow_mut();
                    let template = font.handle.template();
                    Font::from_bytes(Arc::new(template.bytes()), 0)
                        .ok()
                        .or_else(|| load_system_font_from_style(Some(style)))
                })
            },
        );
        let font = match font {
            Some(f) => f,
            None => {
                error!("Couldn't load desired font or system fallback.");
                return;
            },
        };
        let font_width = font_width(&text, point_size, &font);

        // Step 6.
        let max_width = max_width.map(|width| width as f32);
        let (width, scale_factor) = match max_width {
            Some(max_width) if max_width > font_width => (max_width, 1.),
            Some(max_width) => (font_width, max_width / font_width),
            None => (font_width, 1.),
        };

        // Step 7.
        let start = self.text_origin(x as f32, y as f32, &font.metrics(), width, is_rtl);

        // TODO: Bidi text layout

        let old_transform = self.get_transform();
        self.set_transform(
            &old_transform
                .pre_translate(vec2(start.x, 0.))
                .pre_scale(scale_factor, 1.)
                .pre_translate(vec2(-start.x, 0.)),
        );

        // Step 8.
        self.drawtarget.fill_text(
            &font,
            point_size,
            &text,
            start,
            &self.state.fill_style,
            &self.state.draw_options,
        );

        self.set_transform(&old_transform);
    }

    fn text_origin(
        &self,
        x: f32,
        y: f32,
        metrics: &Metrics,
        width: f32,
        is_rtl: bool,
    ) -> Point2D<f32> {
        let text_align = match self.state.text_align {
            TextAlign::Start if is_rtl => TextAlign::Right,
            TextAlign::Start => TextAlign::Left,
            TextAlign::End if is_rtl => TextAlign::Left,
            TextAlign::End => TextAlign::Right,
            text_align => text_align,
        };
        let anchor_x = match text_align {
            TextAlign::Center => -width / 2.,
            TextAlign::Right => -width,
            _ => 0.,
        };

        let anchor_y = match self.state.text_baseline {
            TextBaseline::Top => metrics.ascent,
            TextBaseline::Hanging => metrics.ascent * HANGING_BASELINE_DEFAULT,
            TextBaseline::Ideographic => -metrics.descent * IDEOGRAPHIC_BASELINE_DEFAULT,
            TextBaseline::Middle => (metrics.ascent - metrics.descent) / 2.,
            TextBaseline::Alphabetic => 0.,
            TextBaseline::Bottom => -metrics.descent,
        };

        point2(x + anchor_x, y + anchor_y)
    }

    pub fn fill_rect(&mut self, rect: &Rect<f32>) {
        if self.state.fill_style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        let draw_rect = match &self.state.fill_style {
            Pattern::Raqote(pattern) => match pattern {
                crate::raqote_backend::Pattern::Surface(pattern) => {
                    let pattern_rect = Rect::new(Point2D::origin(), pattern.size());
                    let mut draw_rect = rect.intersection(&pattern_rect).unwrap_or(Rect::zero());

                    match pattern.repetition() {
                        Repetition::NoRepeat => {
                            draw_rect.size.width =
                                draw_rect.size.width.min(pattern_rect.size.width);
                            draw_rect.size.height =
                                draw_rect.size.height.min(pattern_rect.size.height);
                        },
                        Repetition::RepeatX => {
                            draw_rect.size.width = rect.size.width;
                            draw_rect.size.height =
                                draw_rect.size.height.min(pattern_rect.size.height);
                        },
                        Repetition::RepeatY => {
                            draw_rect.size.height = rect.size.height;
                            draw_rect.size.width =
                                draw_rect.size.width.min(pattern_rect.size.width);
                        },
                        Repetition::Repeat => {
                            draw_rect = *rect;
                        },
                    }

                    draw_rect
                },
                crate::raqote_backend::Pattern::Color(..) |
                crate::raqote_backend::Pattern::LinearGradient(..) |
                crate::raqote_backend::Pattern::RadialGradient(..) => *rect,
            },
        };

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&draw_rect, |new_draw_target: &mut dyn GenericDrawTarget| {
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

    pub fn clear_rect(&mut self, rect: &Rect<f32>) {
        self.drawtarget.clear_rect(rect);
    }

    pub fn stroke_rect(&mut self, rect: &Rect<f32>) {
        if self.state.stroke_style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        if self.need_to_draw_shadow() {
            self.draw_with_shadow(&rect, |new_draw_target: &mut dyn GenericDrawTarget| {
                new_draw_target.stroke_rect(
                    rect,
                    self.state.stroke_style.clone(),
                    &self.state.stroke_opts,
                    &self.state.draw_options,
                );
            });
        } else if rect.size.width == 0. || rect.size.height == 0. {
            let mut stroke_opts = self.state.stroke_opts.clone();
            stroke_opts.set_line_cap(LineCapStyle::Butt);
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
        self.path_builder().close();
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
            PathState::UserSpacePathBuilder(ref mut builder, ref mut transform) => {
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
                Some(path.transformed_copy_to_builder(transform))
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
        let new_state = match *self.path_state.as_mut().unwrap() {
            PathState::DeviceSpacePathBuilder(ref mut builder) => {
                let path = builder.finish();
                let inverse = match self.drawtarget.get_transform().inverse() {
                    Some(m) => m,
                    None => {
                        warn!("Couldn't invert canvas transformation.");
                        return;
                    },
                };
                let mut builder = path.transformed_copy_to_builder(&inverse);
                Some(builder.finish())
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
        if self.state.fill_style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        self.ensure_path();
        self.drawtarget.fill(
            &self.path().clone(),
            self.state.fill_style.clone(),
            &self.state.draw_options,
        );
    }

    pub fn stroke(&mut self) {
        if self.state.stroke_style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        self.ensure_path();
        self.drawtarget.stroke(
            &self.path().clone(),
            self.state.stroke_style.clone(),
            &self.state.stroke_opts,
            &self.state.draw_options,
        );
    }

    pub fn clip(&mut self) {
        self.ensure_path();
        let path = self.path().clone();
        self.drawtarget.push_clip(&path);
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
                path.contains_point(x, y, path_transform)
            },
            Some(_) | None => false,
        };
        chan.send(result).unwrap();
    }

    pub fn move_to(&mut self, point: &Point2D<f32>) {
        self.path_builder().move_to(point);
    }

    pub fn line_to(&mut self, point: &Point2D<f32>) {
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
            match *self.path_state.as_mut().unwrap() {
                PathState::UserSpacePathBuilder(_, None) | PathState::DeviceSpacePathBuilder(_) => {
                    None
                },
                PathState::UserSpacePathBuilder(ref mut builder, Some(ref transform)) => {
                    let path = builder.finish();
                    Some(PathState::DeviceSpacePathBuilder(
                        path.transformed_copy_to_builder(transform),
                    ))
                },
                PathState::UserSpacePath(ref path, Some(ref transform)) => Some(
                    PathState::DeviceSpacePathBuilder(path.transformed_copy_to_builder(transform)),
                ),
                PathState::UserSpacePath(ref path, None) => Some(PathState::UserSpacePathBuilder(
                    path.copy_to_builder(),
                    None,
                )),
            }
        };
        match new_state {
            // There's a new builder value that needs to be stored.
            Some(state) => self.path_state = Some(state),
            // There's an existing builder value that can be returned immediately.
            None => match *self.path_state.as_mut().unwrap() {
                PathState::UserSpacePathBuilder(ref mut builder, None) => {
                    return PathBuilderRef {
                        builder,
                        transform: Transform2D::identity(),
                    };
                },
                PathState::DeviceSpacePathBuilder(ref mut builder) => {
                    return PathBuilderRef {
                        builder,
                        transform: self.drawtarget.get_transform(),
                    };
                },
                _ => unreachable!(),
            },
        }

        match *self.path_state.as_mut().unwrap() {
            PathState::UserSpacePathBuilder(ref mut builder, None) => PathBuilderRef {
                builder,
                transform: Transform2D::identity(),
            },
            PathState::DeviceSpacePathBuilder(ref mut builder) => PathBuilderRef {
                builder,
                transform: self.drawtarget.get_transform(),
            },
            PathState::UserSpacePathBuilder(..) | PathState::UserSpacePath(..) => unreachable!(),
        }
    }

    pub fn rect(&mut self, rect: &Rect<f32>) {
        self.path_builder().rect(rect);
    }

    pub fn quadratic_curve_to(&mut self, cp: &Point2D<f32>, endpoint: &Point2D<f32>) {
        if self.path_state.is_none() {
            self.move_to(cp);
        }
        self.path_builder().quadratic_curve_to(cp, endpoint);
    }

    pub fn bezier_curve_to(
        &mut self,
        cp1: &Point2D<f32>,
        cp2: &Point2D<f32>,
        endpoint: &Point2D<f32>,
    ) {
        if self.path_state.is_none() {
            self.move_to(cp1);
        }
        self.path_builder().bezier_curve_to(cp1, cp2, endpoint);
    }

    pub fn arc(
        &mut self,
        center: &Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        ccw: bool,
    ) {
        self.path_builder()
            .arc(center, radius, start_angle, end_angle, ccw);
    }

    pub fn arc_to(&mut self, cp1: &Point2D<f32>, cp2: &Point2D<f32>, radius: f32) {
        let cp0 = match self.path_builder().current_point() {
            Some(p) => p,
            None => {
                self.path_builder().move_to(cp1);
                cp1.clone()
            },
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
        center: &Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
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
        self.backend
            .set_fill_style(style, &mut self.state, &*self.drawtarget);
    }

    pub fn set_stroke_style(&mut self, style: FillOrStrokeStyle) {
        self.backend
            .set_stroke_style(style, &mut self.state, &*self.drawtarget);
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

    pub fn get_transform(&self) -> Transform2D<f32> {
        self.drawtarget.get_transform()
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
        self.backend.set_global_composition(op, &mut self.state);
    }

    pub fn recreate(&mut self, size: Size2D<u64>) {
        self.drawtarget = self
            .backend
            .create_drawtarget(Size2D::new(size.width, size.height));
        self.state = self.backend.recreate_paint_state(&self.state);
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

    pub fn send_pixels(&mut self, chan: IpcSender<IpcSharedMemory>) {
        self.drawtarget.snapshot_data(&|bytes| {
            let data = IpcSharedMemory::from_bytes(bytes);
            chan.send(data).unwrap();
            vec![]
        });
    }

    pub fn send_data(&mut self, chan: IpcSender<CanvasImageData>) {
        let size = self.drawtarget.get_size();

        let descriptor = webrender_api::ImageDescriptor {
            size: webrender_api::units::DeviceIntSize::new(size.width, size.height),
            stride: None,
            format: webrender_api::ImageFormat::BGRA8,
            offset: 0,
            flags: webrender_api::ImageDescriptorFlags::empty(),
        };
        let data = self.drawtarget.snapshot_data_owned();
        let data = webrender_api::ImageData::Raw(Arc::new(data));

        let mut updates = vec![];

        match self.image_key {
            Some(image_key) => {
                debug!("Updating image {:?}.", image_key);
                updates.push(ImageUpdate::Update(image_key, descriptor, data));
            },
            None => {
                let key = match self.webrender_api.generate_key() {
                    Ok(key) => key,
                    Err(()) => return,
                };
                updates.push(ImageUpdate::Add(key, descriptor, data));
                self.image_key = Some(key);
                debug!("New image {:?}.", self.image_key);
            },
        }

        if let Some(image_key) =
            mem::replace(&mut self.very_old_image_key, self.old_image_key.take())
        {
            updates.push(ImageUpdate::Delete(image_key));
        }

        self.webrender_api.update_images(updates);

        let data = CanvasImageData {
            image_key: self.image_key.unwrap(),
        };
        chan.send(data).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    pub fn put_image_data(&mut self, mut imagedata: Vec<u8>, rect: Rect<u64>) {
        assert_eq!(imagedata.len() % 4, 0);
        assert_eq!(rect.size.area() as usize, imagedata.len() / 4);
        pixels::rgba8_byte_swap_and_premultiply_inplace(&mut imagedata);
        let source_surface = self
            .drawtarget
            .create_source_surface_from_data(
                &imagedata,
                rect.size.to_i32(),
                rect.size.width as i32 * 4,
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

    pub fn set_shadow_color(&mut self, value: RGBA) {
        self.backend.set_shadow_color(value, &mut self.state);
    }

    pub fn set_font(&mut self, font_style: FontStyleStruct) {
        self.state.font_style = Some(font_style)
    }

    pub fn set_text_align(&mut self, text_align: TextAlign) {
        self.state.text_align = text_align;
    }

    pub fn set_text_baseline(&mut self, text_baseline: TextBaseline) {
        self.state.text_baseline = text_baseline;
    }

    // https://html.spec.whatwg.org/multipage/#when-shadows-are-drawn
    fn need_to_draw_shadow(&self) -> bool {
        self.backend.need_to_draw_shadow(&self.state.shadow_color) &&
            (self.state.shadow_offset_x != 0.0f64 ||
                self.state.shadow_offset_y != 0.0f64 ||
                self.state.shadow_blur != 0.0f64)
    }

    fn create_draw_target_for_shadow(&self, source_rect: &Rect<f32>) -> Box<dyn GenericDrawTarget> {
        let mut draw_target = self.drawtarget.create_similar_draw_target(
            &Size2D::new(
                source_rect.size.width as i32,
                source_rect.size.height as i32,
            ),
            self.drawtarget.get_format(),
        );
        let matrix = Transform2D::identity()
            .pre_translate(-source_rect.origin.to_vector().cast::<f32>())
            .pre_transform(&self.state.transform);
        draw_target.set_transform(&matrix);
        draw_target
    }

    fn draw_with_shadow<F>(&self, rect: &Rect<f32>, draw_shadow_source: F)
    where
        F: FnOnce(&mut dyn GenericDrawTarget),
    {
        let shadow_src_rect = self.state.transform.transform_rect(rect);
        let mut new_draw_target = self.create_draw_target_for_shadow(&shadow_src_rect);
        draw_shadow_source(&mut *new_draw_target);
        self.drawtarget.draw_surface_with_shadow(
            new_draw_target.snapshot(),
            &Point2D::new(
                shadow_src_rect.origin.x as f32,
                shadow_src_rect.origin.y as f32,
            ),
            &self.state.shadow_color,
            &Vector2D::new(
                self.state.shadow_offset_x as f32,
                self.state.shadow_offset_y as f32,
            ),
            (self.state.shadow_blur / 2.0f64) as f32,
            self.backend.get_composition_op(&self.state.draw_options),
        );
    }

    /// It reads image data from the canvas
    /// canvas_size: The size of the canvas we're reading from
    /// read_rect: The area of the canvas we want to read from
    #[allow(unsafe_code)]
    pub fn read_pixels(&self, read_rect: Rect<u64>, canvas_size: Size2D<u64>) -> Vec<u8> {
        let canvas_rect = Rect::from_size(canvas_size);
        if canvas_rect
            .intersection(&read_rect)
            .map_or(true, |rect| rect.is_empty())
        {
            return vec![];
        }

        self.drawtarget.snapshot_data(&|bytes| {
            pixels::rgba8_get_rect(bytes, canvas_size, read_rect).into_owned()
        })
    }
}

impl<'a> Drop for CanvasData<'a> {
    fn drop(&mut self) {
        let mut updates = vec![];
        if let Some(image_key) = self.old_image_key.take() {
            updates.push(ImageUpdate::Delete(image_key));
        }
        if let Some(image_key) = self.very_old_image_key.take() {
            updates.push(ImageUpdate::Delete(image_key));
        }

        self.webrender_api.update_images(updates);
    }
}

const HANGING_BASELINE_DEFAULT: f32 = 0.8;
const IDEOGRAPHIC_BASELINE_DEFAULT: f32 = 0.5;

#[derive(Clone)]
pub struct CanvasPaintState<'a> {
    pub draw_options: DrawOptions,
    pub fill_style: Pattern<'a>,
    pub stroke_style: Pattern<'a>,
    pub stroke_opts: StrokeOptions<'a>,
    /// The current 2D transform matrix.
    pub transform: Transform2D<f32>,
    pub shadow_offset_x: f64,
    pub shadow_offset_y: f64,
    pub shadow_blur: f64,
    pub shadow_color: Color,
    pub font_style: Option<FontStyleStruct>,
    pub text_align: TextAlign,
    pub text_baseline: TextBaseline,
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
fn write_image(
    draw_target: &mut dyn GenericDrawTarget,
    image_data: Vec<u8>,
    image_size: Size2D<f64>,
    dest_rect: Rect<f64>,
    smoothing_enabled: bool,
    draw_options: &DrawOptions,
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
        .create_source_surface_from_data(&image_data, image_size, image_size.width * 4)
        .unwrap();

    draw_target.draw_surface(source_surface, dest_rect, image_rect, filter, draw_options);
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

pub trait RectExt {
    fn to_u64(&self) -> Rect<u64>;
}

impl RectExt for Rect<f64> {
    fn to_u64(&self) -> Rect<u64> {
        self.cast()
    }
}

impl RectExt for Rect<u32> {
    fn to_u64(&self) -> Rect<u64> {
        self.cast()
    }
}

fn to_font_kit_family(font_family: &font::SingleFontFamily) -> FamilyName {
    match font_family {
        font::SingleFontFamily::FamilyName(family_name) => {
            FamilyName::Title(family_name.to_css_string())
        },
        font::SingleFontFamily::Generic(generic) => match generic {
            font::GenericFontFamily::Serif => FamilyName::Serif,
            font::GenericFontFamily::SansSerif => FamilyName::SansSerif,
            font::GenericFontFamily::Monospace => FamilyName::Monospace,
            font::GenericFontFamily::Fantasy => FamilyName::Fantasy,
            font::GenericFontFamily::Cursive => FamilyName::Cursive,
            font::GenericFontFamily::None => unreachable!("Shouldn't appear in computed values"),
        },
    }
}

fn load_system_font_from_style(font_style: Option<&FontStyleStruct>) -> Option<Font> {
    let mut properties = Properties::new();
    let style = match font_style {
        Some(style) => style,
        None => return load_default_system_fallback_font(&properties),
    };
    let family_names = style
        .font_family
        .families
        .iter()
        .map(to_font_kit_family)
        .collect::<Vec<_>>();
    let properties = properties
        .style(match style.font_style {
            font::FontStyle::Normal => Style::Normal,
            font::FontStyle::Italic => Style::Italic,
            font::FontStyle::Oblique(..) => {
                // TODO: support oblique angle.
                Style::Oblique
            },
        })
        .weight(Weight(style.font_weight.0))
        .stretch(Stretch(style.font_stretch.value()));
    let font_handle = match SystemSource::new().select_best_match(&family_names, &properties) {
        Ok(handle) => handle,
        Err(e) => {
            error!("error getting font handle for style {:?}: {}", style, e);
            return load_default_system_fallback_font(&properties);
        },
    };
    match font_handle.load() {
        Ok(f) => Some(f),
        Err(e) => {
            error!("error loading font for style {:?}: {}", style, e);
            load_default_system_fallback_font(&properties)
        },
    }
}

fn load_default_system_fallback_font(properties: &Properties) -> Option<Font> {
    SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], properties)
        .ok()?
        .load()
        .ok()
}

fn replace_ascii_whitespace(text: String) -> String {
    text.chars()
        .map(|c| match c {
            ' ' | '\t' | '\n' | '\r' | '\x0C' => '\x20',
            _ => c,
        })
        .collect()
}

// TODO: This currently calculates the width using just advances and doesn't
// determine the fallback font in case a character glyph isn't found.
fn font_width(text: &str, point_size: f32, font: &Font) -> f32 {
    let metrics = font.metrics();
    let mut width = 0.;
    for c in text.chars() {
        if let Some(glyph_id) = font.glyph_for_char(c) {
            if let Ok(advance) = font.advance(glyph_id) {
                width += advance.x() * point_size / metrics.units_per_em as f32;
            }
        }
    }
    width
}
