/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::gl_utils::framebuffer;
use crate::{SurfmanGL, SurfmanLayerManager};
use core::slice;
use euclid::{
    Angle, Point2D, Rect, RigidTransform3D, Rotation3D, Size2D, Transform3D, UnknownUnit, Vector3D,
};
use glow::{self as gl, Context as Gl, HasContext};
use raw_window_handle::DisplayHandle;
use std::num::NonZeroU32;
use std::rc::Rc;
use surfman::chains::{PreserveBuffer, SwapChain, SwapChainAPI, SwapChains, SwapChainsAPI};
use surfman::{
    Adapter, Connection, Context as SurfmanContext, ContextAttributeFlags, ContextAttributes,
    Device as SurfmanDevice, GLApi, GLVersion, NativeWidget, SurfaceAccess, SurfaceType,
};
use webxr_api::util::ClipPlanes;
use webxr_api::{
    ContextId, DeviceAPI, DiscoveryAPI, Display, Error, Event, EventBuffer, Floor, Frame,
    InputSource, LayerGrandManager, LayerId, LayerInit, LayerManager, Native, Quitter, Sender,
    Session, SessionBuilder, SessionInit, SessionMode, SomeEye, View, Viewer, ViewerPose, Viewport,
    Viewports, Views, CUBE_BACK, CUBE_BOTTOM, CUBE_LEFT, CUBE_RIGHT, CUBE_TOP, LEFT_EYE, RIGHT_EYE,
    VIEWER,
};

// How far off the ground are the viewer's eyes?
const HEIGHT: f32 = 1.0;

// What is half the vertical field of view?
const FOV_UP: f32 = 45.0;

// Some guesstimated numbers, hopefully it doesn't matter if these are off by a bit.

// What the distance between the viewer's eyes?
const INTER_PUPILLARY_DISTANCE: f32 = 0.06;

// What is the size of a pixel?
const PIXELS_PER_METRE: f32 = 6000.0;

pub trait GlWindow {
    fn get_render_target(
        &self,
        device: &mut SurfmanDevice,
        context: &mut SurfmanContext,
    ) -> GlWindowRenderTarget;
    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit>;
    fn get_translation(&self) -> Vector3D<f32, UnknownUnit>;

    fn get_mode(&self) -> GlWindowMode {
        GlWindowMode::Blit
    }
    fn display_handle(&self) -> DisplayHandle;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GlWindowMode {
    Blit,
    StereoLeftRight,
    StereoRedCyan,
    Cubemap,
    Spherical,
}

pub enum GlWindowRenderTarget {
    NativeWidget(NativeWidget),
    SwapChain(SwapChain<SurfmanDevice>),
}

pub struct GlWindowDiscovery {
    connection: Connection,
    adapter: Adapter,
    context_attributes: ContextAttributes,
    window: Rc<dyn GlWindow>,
}

impl GlWindowDiscovery {
    pub fn new(window: Rc<dyn GlWindow>) -> GlWindowDiscovery {
        let connection = Connection::from_display_handle(window.display_handle()).unwrap();
        let adapter = connection.create_adapter().unwrap();
        let flags = ContextAttributeFlags::ALPHA
            | ContextAttributeFlags::DEPTH
            | ContextAttributeFlags::STENCIL;
        let version = match connection.gl_api() {
            GLApi::GLES => GLVersion { major: 3, minor: 0 },
            GLApi::GL => GLVersion { major: 3, minor: 2 },
        };
        let context_attributes = ContextAttributes { flags, version };
        GlWindowDiscovery {
            connection,
            adapter,
            context_attributes,
            window,
        }
    }
}

impl DiscoveryAPI<SurfmanGL> for GlWindowDiscovery {
    fn request_session(
        &mut self,
        mode: SessionMode,
        init: &SessionInit,
        xr: SessionBuilder<SurfmanGL>,
    ) -> Result<Session, Error> {
        if self.supports_session(mode) {
            let granted_features = init.validate(mode, &["local-floor".into()])?;
            let connection = self.connection.clone();
            let adapter = self.adapter.clone();
            let context_attributes = self.context_attributes.clone();
            let window = self.window.clone();
            xr.run_on_main_thread(move |grand_manager| {
                GlWindowDevice::new(
                    connection,
                    adapter,
                    context_attributes,
                    window,
                    granted_features,
                    grand_manager,
                )
            })
        } else {
            Err(Error::NoMatchingDevice)
        }
    }

    fn supports_session(&self, mode: SessionMode) -> bool {
        mode == SessionMode::ImmersiveVR || mode == SessionMode::ImmersiveAR
    }
}

pub struct GlWindowDevice {
    device: SurfmanDevice,
    context: SurfmanContext,
    gl: Rc<Gl>,
    window: Rc<dyn GlWindow>,
    grand_manager: LayerGrandManager<SurfmanGL>,
    layer_manager: Option<LayerManager>,
    target_swap_chain: Option<SwapChain<SurfmanDevice>>,
    swap_chains: SwapChains<LayerId, SurfmanDevice>,
    read_fbo: Option<gl::NativeFramebuffer>,
    events: EventBuffer,
    clip_planes: ClipPlanes,
    granted_features: Vec<String>,
    shader: Option<GlWindowShader>,
}

impl DeviceAPI for GlWindowDevice {
    fn floor_transform(&self) -> Option<RigidTransform3D<f32, Native, Floor>> {
        let translation = Vector3D::new(0.0, HEIGHT, 0.0);
        Some(RigidTransform3D::from_translation(translation))
    }

    fn viewports(&self) -> Viewports {
        let size = self.viewport_size();
        let viewports = match self.window.get_mode() {
            GlWindowMode::Cubemap | GlWindowMode::Spherical => vec![
                Rect::new(Point2D::new(size.width * 1, size.height * 1), size),
                Rect::new(Point2D::new(size.width * 0, size.height * 1), size),
                Rect::new(Point2D::new(size.width * 2, size.height * 1), size),
                Rect::new(Point2D::new(size.width * 2, size.height * 0), size),
                Rect::new(Point2D::new(size.width * 0, size.height * 0), size),
                Rect::new(Point2D::new(size.width * 1, size.height * 0), size),
            ],
            GlWindowMode::Blit | GlWindowMode::StereoLeftRight | GlWindowMode::StereoRedCyan => {
                vec![
                    Rect::new(Point2D::default(), size),
                    Rect::new(Point2D::new(size.width, 0), size),
                ]
            }
        };
        Viewports { viewports }
    }

    fn create_layer(&mut self, context_id: ContextId, init: LayerInit) -> Result<LayerId, Error> {
        self.layer_manager()?.create_layer(context_id, init)
    }

    fn destroy_layer(&mut self, context_id: ContextId, layer_id: LayerId) {
        self.layer_manager()
            .unwrap()
            .destroy_layer(context_id, layer_id)
    }

    fn begin_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) -> Option<Frame> {
        log::debug!("Begin animation frame for layers {:?}", layers);
        let translation = Vector3D::from_untyped(self.window.get_translation());
        let translation: RigidTransform3D<_, _, Native> =
            RigidTransform3D::from_translation(translation);
        let rotation = Rotation3D::from_untyped(&self.window.get_rotation());
        let rotation = RigidTransform3D::from_rotation(rotation);
        let transform = translation.then(&rotation);
        let sub_images = self.layer_manager().ok()?.begin_frame(layers).ok()?;
        Some(Frame {
            pose: Some(ViewerPose {
                transform,
                views: self.views(transform),
            }),
            inputs: vec![],
            events: vec![],
            sub_images,
            hit_test_results: vec![],
            predicted_display_time: 0.0,
        })
    }

    fn end_animation_frame(&mut self, layers: &[(ContextId, LayerId)]) {
        log::debug!("End animation frame for layers {:?}", layers);
        self.device.make_context_current(&self.context).unwrap();
        debug_assert_eq!(unsafe { self.gl.get_error() }, gl::NO_ERROR);

        let _ = self.layer_manager().unwrap().end_frame(layers);

        let window_size = self.window_size();
        let viewport_size = self.viewport_size();

        let framebuffer_object = self
            .device
            .context_surface_info(&self.context)
            .unwrap()
            .map(|info| info.framebuffer_object)
            .unwrap_or(0);
        unsafe {
            self.gl
                .bind_framebuffer(gl::FRAMEBUFFER, framebuffer(framebuffer_object));
            debug_assert_eq!(
                (
                    self.gl.get_error(),
                    self.gl.check_framebuffer_status(gl::FRAMEBUFFER)
                ),
                (gl::NO_ERROR, gl::FRAMEBUFFER_COMPLETE)
            );

            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl.clear(gl::COLOR_BUFFER_BIT);
            debug_assert_eq!(self.gl.get_error(), gl::NO_ERROR);
        }

        for &(_, layer_id) in layers {
            let swap_chain = match self.swap_chains.get(layer_id) {
                Some(swap_chain) => swap_chain,
                None => continue,
            };
            let surface = match swap_chain.take_surface() {
                Some(surface) => surface,
                None => return,
            };
            let texture_size = self.device.surface_info(&surface).size;
            let surface_texture = self
                .device
                .create_surface_texture(&mut self.context, surface)
                .unwrap();
            let raw_texture_id = self.device.surface_texture_object(&surface_texture);
            let texture_id = NonZeroU32::new(raw_texture_id).map(gl::NativeTexture);
            let texture_target = self.device.surface_gl_texture_target();
            log::debug!("Presenting texture {}", raw_texture_id);

            if let Some(ref shader) = self.shader {
                shader.draw_texture(
                    texture_id,
                    texture_target,
                    texture_size,
                    viewport_size,
                    window_size,
                );
            } else {
                self.blit_texture(texture_id, texture_target, texture_size, window_size);
            }
            debug_assert_eq!(unsafe { self.gl.get_error() }, gl::NO_ERROR);

            let surface = self
                .device
                .destroy_surface_texture(&mut self.context, surface_texture)
                .unwrap();
            swap_chain.recycle_surface(surface);
        }

        match self.target_swap_chain.as_ref() {
            Some(target_swap_chain) => {
                // Rendering to a surfman swap chain
                target_swap_chain
                    .swap_buffers(&mut self.device, &mut self.context, PreserveBuffer::No)
                    .unwrap();
            }
            None => {
                // Rendering to a native widget
                let mut surface = self
                    .device
                    .unbind_surface_from_context(&mut self.context)
                    .unwrap()
                    .unwrap();
                self.device
                    .present_surface(&self.context, &mut surface)
                    .unwrap();
                self.device
                    .bind_surface_to_context(&mut self.context, surface)
                    .unwrap();
            }
        }

        debug_assert_eq!(unsafe { self.gl.get_error() }, gl::NO_ERROR);
    }

    fn initial_inputs(&self) -> Vec<InputSource> {
        vec![]
    }

    fn set_event_dest(&mut self, dest: Sender<Event>) {
        self.events.upgrade(dest)
    }

    fn quit(&mut self) {
        self.events.callback(Event::SessionEnd);
    }

    fn set_quitter(&mut self, _: Quitter) {
        // Glwindow currently doesn't have any way to end its own session
        // XXXManishearth add something for this that listens for the window
        // being closed
    }

    fn update_clip_planes(&mut self, near: f32, far: f32) {
        self.clip_planes.update(near, far)
    }

    fn granted_features(&self) -> &[String] {
        &self.granted_features
    }
}

impl Drop for GlWindowDevice {
    fn drop(&mut self) {
        if let Some(read_fbo) = self.read_fbo {
            unsafe {
                self.gl.delete_framebuffer(read_fbo);
            }
        }
        let _ = self.device.destroy_context(&mut self.context);
    }
}

impl GlWindowDevice {
    fn new(
        connection: Connection,
        adapter: Adapter,
        context_attributes: ContextAttributes,
        window: Rc<dyn GlWindow>,
        granted_features: Vec<String>,
        grand_manager: LayerGrandManager<SurfmanGL>,
    ) -> Result<GlWindowDevice, Error> {
        let mut device = connection.create_device(&adapter).unwrap();
        let context_descriptor = device
            .create_context_descriptor(&context_attributes)
            .unwrap();
        let mut context = device.create_context(&context_descriptor, None).unwrap();
        device.make_context_current(&context).unwrap();

        let gl = Rc::new(unsafe {
            match device.gl_api() {
                GLApi::GL => Gl::from_loader_function(|symbol_name| {
                    device.get_proc_address(&context, symbol_name)
                }),
                GLApi::GLES => Gl::from_loader_function(|symbol_name| {
                    device.get_proc_address(&context, symbol_name)
                }),
            }
        });

        let target_swap_chain = match window.get_render_target(&mut device, &mut context) {
            GlWindowRenderTarget::NativeWidget(native_widget) => {
                let surface_type = SurfaceType::Widget { native_widget };
                let surface = device
                    .create_surface(&context, SurfaceAccess::GPUOnly, surface_type)
                    .unwrap();
                device
                    .bind_surface_to_context(&mut context, surface)
                    .unwrap();
                None
            }
            GlWindowRenderTarget::SwapChain(target_swap_chain) => {
                debug_assert!(target_swap_chain.is_attached());
                Some(target_swap_chain)
            }
        };

        let read_fbo = unsafe { gl.create_framebuffer().ok() };
        unsafe {
            let framebuffer_object = device
                .context_surface_info(&context)
                .unwrap()
                .map(|info| info.framebuffer_object)
                .unwrap_or(0);
            gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer(framebuffer_object));
            debug_assert_eq!(
                (gl.get_error(), gl.check_framebuffer_status(gl::FRAMEBUFFER)),
                (gl::NO_ERROR, gl::FRAMEBUFFER_COMPLETE)
            );

            gl.enable(gl::BLEND);
            gl.blend_func_separate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
        }

        let swap_chains = SwapChains::new();
        let layer_manager = None;

        let shader = GlWindowShader::new(gl.clone(), window.get_mode());
        debug_assert_eq!(unsafe { gl.get_error() }, gl::NO_ERROR);

        Ok(GlWindowDevice {
            gl,
            window,
            device,
            context,
            read_fbo,
            swap_chains,
            target_swap_chain,
            grand_manager,
            layer_manager,
            events: Default::default(),
            clip_planes: Default::default(),
            granted_features,
            shader,
        })
    }

    fn blit_texture(
        &self,
        texture_id: Option<gl::NativeTexture>,
        texture_target: u32,
        texture_size: Size2D<i32, UnknownUnit>,
        window_size: Size2D<i32, Viewport>,
    ) {
        unsafe {
            self.gl
                .bind_framebuffer(gl::READ_FRAMEBUFFER, self.read_fbo);
            self.gl.framebuffer_texture_2d(
                gl::READ_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                texture_target,
                texture_id,
                0,
            );
            self.gl.blit_framebuffer(
                0,
                0,
                texture_size.width,
                texture_size.height,
                0,
                0,
                window_size.width,
                window_size.height,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
        }
    }

    fn layer_manager(&mut self) -> Result<&mut LayerManager, Error> {
        if let Some(ref mut manager) = self.layer_manager {
            return Ok(manager);
        }
        let swap_chains = self.swap_chains.clone();
        let viewports = self.viewports();
        let layer_manager = self.grand_manager.create_layer_manager(move |_, _| {
            Ok(SurfmanLayerManager::new(viewports, swap_chains))
        })?;
        self.layer_manager = Some(layer_manager);
        Ok(self.layer_manager.as_mut().unwrap())
    }

    fn window_size(&self) -> Size2D<i32, Viewport> {
        let window_size = self
            .device
            .context_surface_info(&self.context)
            .unwrap()
            .unwrap()
            .size
            .to_i32();
        Size2D::from_untyped(window_size)
    }

    fn viewport_size(&self) -> Size2D<i32, Viewport> {
        let window_size = self.window_size();
        match self.window.get_mode() {
            GlWindowMode::StereoRedCyan => {
                // This device has a slightly odd characteristic, which is that anaglyphic stereo
                // renders both eyes to the same surface. If we want the two eyes to be parallel,
                // and to agree at distance infinity, this means gettng the XR content to render some
                // wasted pixels, which are stripped off when we render to the target surface.
                // (The wasted pixels are on the right of the left eye and vice versa.)
                let wasted_pixels = (INTER_PUPILLARY_DISTANCE / PIXELS_PER_METRE) as i32;
                Size2D::new(window_size.width + wasted_pixels, window_size.height)
            }
            GlWindowMode::Cubemap => {
                // Cubemap viewports should be square
                let size = 1.max(window_size.width / 3).max(window_size.height / 2);
                Size2D::new(size, size)
            }
            GlWindowMode::Spherical => {
                // Cubemap viewports should be square
                let size = 1.max(window_size.width / 2).max(window_size.height);
                Size2D::new(size, size)
            }
            GlWindowMode::StereoLeftRight | GlWindowMode::Blit => {
                Size2D::new(window_size.width / 2, window_size.height)
            }
        }
    }

    fn views(&self, viewer: RigidTransform3D<f32, Viewer, Native>) -> Views {
        match self.window.get_mode() {
            GlWindowMode::Cubemap | GlWindowMode::Spherical => Views::Cubemap(
                self.view(viewer, VIEWER),
                self.view(viewer, CUBE_LEFT),
                self.view(viewer, CUBE_RIGHT),
                self.view(viewer, CUBE_TOP),
                self.view(viewer, CUBE_BOTTOM),
                self.view(viewer, CUBE_BACK),
            ),
            GlWindowMode::Blit | GlWindowMode::StereoLeftRight | GlWindowMode::StereoRedCyan => {
                Views::Stereo(self.view(viewer, LEFT_EYE), self.view(viewer, RIGHT_EYE))
            }
        }
    }

    fn view<Eye>(
        &self,
        viewer: RigidTransform3D<f32, Viewer, Native>,
        eye: SomeEye<Eye>,
    ) -> View<Eye> {
        let projection = self.perspective();
        let translation = if eye == RIGHT_EYE {
            Vector3D::new(-INTER_PUPILLARY_DISTANCE / 2.0, 0.0, 0.0)
        } else if eye == LEFT_EYE {
            Vector3D::new(INTER_PUPILLARY_DISTANCE / 2.0, 0.0, 0.0)
        } else {
            Vector3D::zero()
        };
        let rotation = if eye == CUBE_TOP {
            Rotation3D::euler(
                Angle::degrees(270.0),
                Angle::degrees(0.0),
                Angle::degrees(90.0),
            )
        } else if eye == CUBE_BOTTOM {
            Rotation3D::euler(
                Angle::degrees(90.0),
                Angle::degrees(0.0),
                Angle::degrees(90.0),
            )
        } else if eye == CUBE_LEFT {
            Rotation3D::around_y(Angle::degrees(-90.0))
        } else if eye == CUBE_RIGHT {
            Rotation3D::around_y(Angle::degrees(90.0))
        } else if eye == CUBE_BACK {
            Rotation3D::euler(
                Angle::degrees(180.0),
                Angle::degrees(0.0),
                Angle::degrees(90.0),
            )
        } else {
            Rotation3D::identity()
        };
        let transform: RigidTransform3D<f32, Viewer, Eye> =
            RigidTransform3D::new(rotation, translation);
        View {
            transform: transform.inverse().then(&viewer),
            projection,
        }
    }

    fn perspective<Eye>(&self) -> Transform3D<f32, Eye, Display> {
        let near = self.clip_planes.near;
        let far = self.clip_planes.far;
        // https://github.com/toji/gl-matrix/blob/bd3307196563fbb331b40fc6ebecbbfcc2a4722c/src/mat4.js#L1271
        let fov_up = match self.window.get_mode() {
            GlWindowMode::Spherical | GlWindowMode::Cubemap => Angle::degrees(45.0),
            GlWindowMode::Blit | GlWindowMode::StereoLeftRight | GlWindowMode::StereoRedCyan => {
                Angle::degrees(FOV_UP)
            }
        };
        let f = 1.0 / fov_up.radians.tan();
        let nf = 1.0 / (near - far);
        let viewport_size = self.viewport_size();
        let aspect = viewport_size.width as f32 / viewport_size.height as f32;

        // Dear rustfmt, This is a 4x4 matrix, please leave it alone. Best, ajeffrey.
        {
            #[rustfmt::skip]
            // Sigh, row-major vs column-major
            return Transform3D::new(
                f / aspect, 0.0, 0.0,                   0.0,
                0.0,        f,   0.0,                   0.0,
                0.0,        0.0, (far + near) * nf,     -1.0,
                0.0,        0.0, 2.0 * far * near * nf, 0.0,
            );
        }
    }
}

struct GlWindowShader {
    gl: Rc<Gl>,
    buffer: Option<gl::NativeBuffer>,
    vao: Option<gl::NativeVertexArray>,
    program: gl::NativeProgram,
    mode: GlWindowMode,
}

const VERTEX_ATTRIBUTE: u32 = 0;
const VERTICES: &[[f32; 2]; 4] = &[[-1.0, -1.0], [-1.0, 1.0], [1.0, -1.0], [1.0, 1.0]];

const PASSTHROUGH_VERTEX_SHADER: &str = "
  #version 330 core
  layout(location=0) in vec2 coord;
  out vec2 vTexCoord;
  void main(void) {
    gl_Position = vec4(coord, 0.0, 1.0);
    vTexCoord = coord * 0.5 + 0.5;
  }
";

const PASSTHROUGH_FRAGMENT_SHADER: &str = "
  #version 330 core
  layout(location=0) out vec4 color;
  uniform sampler2D image;
  in vec2 vTexCoord;
  void main() {
    color = texture(image, vTexCoord);
  }
";

const ANAGLYPH_VERTEX_SHADER: &str = "
  #version 330 core
  layout(location=0) in vec2 coord;
  uniform float wasted; // What fraction of the image is wasted?
  out vec2 left_coord;
  out vec2 right_coord;
  void main(void) {
    gl_Position = vec4(coord, 0.0, 1.0);
    vec2 coordn = coord * 0.5 + 0.5;
    left_coord = vec2(mix(wasted/2, 0.5, coordn.x), coordn.y);
    right_coord = vec2(mix(0.5, 1-wasted/2, coordn.x), coordn.y);
  }
";

const ANAGLYPH_RED_CYAN_FRAGMENT_SHADER: &str = "
  #version 330 core
  layout(location=0) out vec4 color;
  uniform sampler2D image;
  in vec2 left_coord;
  in vec2 right_coord;
  void main() {
    vec4 left_color = texture(image, left_coord);
    vec4 right_color = texture(image, right_coord);
    float red = left_color.x;
    float green = right_color.y;
    float blue = right_color.z;
    color = vec4(red, green, blue, 1.0);
  }
";

const SPHERICAL_VERTEX_SHADER: &str = "
  #version 330 core
  layout(location=0) in vec2 coord;
  out vec2 lon_lat;
  const float PI = 3.141592654;
  void main(void) {
    lon_lat = coord * vec2(PI, 0.5*PI);
    gl_Position = vec4(coord, 0.0, 1.0);
  }
";

const SPHERICAL_FRAGMENT_SHADER: &str = "
  #version 330 core
  layout(location=0) out vec4 color;
  uniform sampler2D image;
  in vec2 lon_lat;
  void main() {
    vec3 direction = vec3(
      sin(lon_lat.x)*cos(lon_lat.y),
      sin(lon_lat.y),
      cos(lon_lat.x)*cos(lon_lat.y)
    );
    vec2 vTexCoord;
    if ((direction.y > abs(direction.x)) && (direction.y > abs(direction.z))) {
      // Looking up
      vTexCoord.x = direction.z / (direction.y*6.0) + 5.0/6.0;
      vTexCoord.y = direction.x / (direction.y*4.0) + 1.0/4.0;
    } else if ((direction.y < -abs(direction.x)) && (direction.y < -abs(direction.z))) {
      // Looking down
      vTexCoord.x = direction.z / (direction.y*6.0) + 1.0/6.0;
      vTexCoord.y = -direction.x / (direction.y*4.0) + 1.0/4.0;
    } else if (direction.z < -abs(direction.x)) {
      // Looking back
      vTexCoord.x = -direction.y / (direction.z*6.0) + 3.0/6.0;
      vTexCoord.y = -direction.x / (direction.z*4.0) + 1.0/4.0;
    } else if (direction.x < -abs(direction.z)) {
      // Looking left
      vTexCoord.x = -direction.z / (direction.x*6.0) + 1.0/6.0;
      vTexCoord.y = -direction.y / (direction.x*4.0) + 3.0/4.0;
    } else if (direction.x > abs(direction.z)) {
      // Looking right
      vTexCoord.x = -direction.z / (direction.x*6.0) + 5.0/6.0;
      vTexCoord.y = direction.y / (direction.x*4.0) + 3.0/4.0;
    } else {
      // Looking ahead
      vTexCoord.x = direction.x / (direction.z*6.0) + 3.0/6.0;
      vTexCoord.y = direction.y / (direction.z*4.0) + 3.0/4.0;
    }
    color = texture(image, vTexCoord);
  }
";

impl GlWindowShader {
    fn new(gl: Rc<Gl>, mode: GlWindowMode) -> Option<GlWindowShader> {
        // The shader source
        let (vertex_source, fragment_source) = match mode {
            GlWindowMode::Blit => {
                return None;
            }
            GlWindowMode::StereoLeftRight | GlWindowMode::Cubemap => {
                (PASSTHROUGH_VERTEX_SHADER, PASSTHROUGH_FRAGMENT_SHADER)
            }
            GlWindowMode::StereoRedCyan => {
                (ANAGLYPH_VERTEX_SHADER, ANAGLYPH_RED_CYAN_FRAGMENT_SHADER)
            }
            GlWindowMode::Spherical => (SPHERICAL_VERTEX_SHADER, SPHERICAL_FRAGMENT_SHADER),
        };

        // TODO: work out why shaders don't work on macos
        if cfg!(target_os = "macos") {
            log::warn!("XR shaders may not render on MacOS.");
        }

        unsafe {
            // The four corners of the window in a VAO, set to attribute 0
            let buffer = gl.create_buffer().ok();
            let vao = gl.create_vertex_array().ok();
            gl.bind_buffer(gl::ARRAY_BUFFER, buffer);

            let data =
                slice::from_raw_parts(VERTICES as *const _ as _, std::mem::size_of_val(VERTICES));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, data, gl::STATIC_DRAW);

            gl.bind_vertex_array(vao);
            gl.vertex_attrib_pointer_f32(
                VERTEX_ATTRIBUTE,
                VERTICES[0].len() as i32,
                gl::FLOAT,
                false,
                0,
                0,
            );
            gl.enable_vertex_attrib_array(VERTEX_ATTRIBUTE);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            // The shader program
            let program = gl.create_program().unwrap();
            let vertex_shader = gl.create_shader(gl::VERTEX_SHADER).unwrap();
            let fragment_shader = gl.create_shader(gl::FRAGMENT_SHADER).unwrap();
            gl.shader_source(vertex_shader, vertex_source);
            gl.compile_shader(vertex_shader);
            gl.attach_shader(program, vertex_shader);
            gl.shader_source(fragment_shader, fragment_source);
            gl.compile_shader(fragment_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            // Check for errors
            // TODO: something other than panic?
            let status = gl.get_shader_compile_status(vertex_shader);
            assert!(
                status,
                "Failed to compile vertex shader: {}",
                gl.get_shader_info_log(vertex_shader)
            );
            let status = gl.get_shader_compile_status(fragment_shader);
            assert!(
                status,
                "Failed to compile fragment shader: {}",
                gl.get_shader_info_log(fragment_shader)
            );
            let status = gl.get_program_link_status(program);
            assert!(
                status,
                "Failed to link: {}",
                gl.get_program_info_log(program)
            );

            // Clean up
            gl.delete_shader(vertex_shader);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
            gl.delete_shader(fragment_shader);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            // And we're done
            Some(GlWindowShader {
                gl,
                buffer,
                vao,
                program,
                mode,
            })
        }
    }

    fn draw_texture(
        &self,
        texture_id: Option<gl::NativeTexture>,
        texture_target: u32,
        texture_size: Size2D<i32, UnknownUnit>,
        viewport_size: Size2D<i32, Viewport>,
        window_size: Size2D<i32, Viewport>,
    ) {
        unsafe {
            self.gl.use_program(Some(self.program));

            self.gl.enable_vertex_attrib_array(VERTEX_ATTRIBUTE);
            self.gl.vertex_attrib_pointer_f32(
                VERTEX_ATTRIBUTE,
                VERTICES[0].len() as i32,
                gl::FLOAT,
                false,
                0,
                0,
            );

            debug_assert_eq!(self.gl.get_error(), gl::NO_ERROR);

            self.gl.active_texture(gl::TEXTURE0);
            self.gl.bind_texture(texture_target, texture_id);

            match self.mode {
                GlWindowMode::StereoRedCyan => {
                    let wasted = 1.0
                        - (texture_size.width as f32 / viewport_size.width as f32)
                            .max(0.0)
                            .min(1.0);
                    let wasted_location = self.gl.get_uniform_location(self.program, "wasted");
                    self.gl.uniform_1_f32(wasted_location.as_ref(), wasted);
                }
                GlWindowMode::Blit
                | GlWindowMode::Cubemap
                | GlWindowMode::Spherical
                | GlWindowMode::StereoLeftRight => {}
            }

            self.gl
                .viewport(0, 0, window_size.width, window_size.height);
            self.gl
                .draw_arrays(gl::TRIANGLE_STRIP, 0, VERTICES.len() as i32);
            self.gl.disable_vertex_attrib_array(VERTEX_ATTRIBUTE);
            debug_assert_eq!(self.gl.get_error(), gl::NO_ERROR);
        }
    }
}

impl Drop for GlWindowShader {
    fn drop(&mut self) {
        unsafe {
            if let Some(buffer) = self.buffer {
                self.gl.delete_buffer(buffer);
            }
            if let Some(vao) = self.vao {
                self.gl.delete_vertex_array(vao);
            }
            self.gl.delete_program(self.program);
        }
    }
}
