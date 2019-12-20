/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::logging::CATEGORY;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use euclid::Point2D;
use euclid::Rect;
use euclid::Scale;
use euclid::Size2D;

use glib::glib_bool_error;
use glib::glib_object_impl;
use glib::glib_object_subclass;
use glib::object::Cast;
use glib::object::Object;
use glib::object::ObjectType;
use glib::subclass::object::ObjectClassSubclassExt;
use glib::subclass::object::ObjectImpl;
use glib::subclass::object::ObjectImplExt;
use glib::subclass::object::Property;
use glib::subclass::simple::ClassStruct;
use glib::subclass::types::ObjectSubclass;
use glib::translate::FromGlibPtrBorrow;
use glib::translate::ToGlibPtr;
use glib::value::Value;
use glib::ParamSpec;
use gstreamer::gst_element_error;
use gstreamer::gst_error_msg;
use gstreamer::gst_loggable_error;
use gstreamer::subclass::element::ElementClassSubclassExt;
use gstreamer::subclass::element::ElementImpl;
use gstreamer::subclass::ElementInstanceStruct;
use gstreamer::Buffer;
use gstreamer::BufferPool;
use gstreamer::BufferPoolExt;
use gstreamer::BufferPoolExtManual;
use gstreamer::Caps;
use gstreamer::CoreError;
use gstreamer::Element;
use gstreamer::ErrorMessage;
use gstreamer::FlowError;
use gstreamer::Format;
use gstreamer::Fraction;
use gstreamer::LoggableError;
use gstreamer::PadDirection;
use gstreamer::PadPresence;
use gstreamer::PadTemplate;
use gstreamer::ResourceError;
use gstreamer_base::subclass::base_src::BaseSrcImpl;
use gstreamer_base::BaseSrc;
use gstreamer_base::BaseSrcExt;
use gstreamer_gl::GLContext;
use gstreamer_gl::GLContextExt;
use gstreamer_gl::GLContextExtManual;
use gstreamer_gl::GLSyncMeta;
use gstreamer_gl_sys::gst_gl_context_thread_add;
use gstreamer_gl_sys::gst_gl_texture_target_to_gl;
use gstreamer_gl_sys::gst_is_gl_memory;
use gstreamer_gl_sys::GstGLContext;
use gstreamer_gl_sys::GstGLMemory;
use gstreamer_video::VideoInfo;

use log::debug;
use log::error;
use log::info;

use servo::compositing::windowing::AnimationState;
use servo::compositing::windowing::EmbedderCoordinates;
use servo::compositing::windowing::EmbedderMethods;
use servo::compositing::windowing::WindowEvent;
use servo::compositing::windowing::WindowMethods;
use servo::embedder_traits::EventLoopWaker;
use servo::msg::constellation_msg::TopLevelBrowsingContextId;
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::DevicePixel;
use servo::Servo;

use sparkle::gl;
use sparkle::gl::types::GLuint;
use sparkle::gl::Gl;

use surfman::connection::Connection as ConnectionAPI;
use surfman::device::Device as DeviceAPI;
use surfman::ContextAttributeFlags;
use surfman::ContextAttributes;
use surfman::GLApi;
use surfman::GLVersion;
use surfman::SurfaceAccess;
use surfman::SurfaceType;
use surfman_chains::SwapChain;
use surfman_chains_api::SwapChainAPI;

// For the moment, we only support wayland and cgl.
#[cfg(target_os = "macos")]
use surfman::platform::macos::cgl::device::Device;
#[cfg(all(unix, not(target_os = "macos")))]
use surfman::platform::unix::wayland::device::Device;

type Context = <Device as DeviceAPI>::Context;
type Connection = <Device as DeviceAPI>::Connection;
type NativeContext = <Device as DeviceAPI>::NativeContext;
type NativeConnection = <Connection as ConnectionAPI>::NativeConnection;

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

pub struct ServoWebSrc {
    sender: Sender<ServoWebSrcMsg>,
    url: Mutex<Option<String>>,
    info: Mutex<Option<VideoInfo>>,
    buffer_pool: Mutex<Option<BufferPool>>,
    gl_context: Mutex<Option<GLContext>>,
    connection: Mutex<Option<Connection>>,
    // When did the plugin get created?
    start: Instant,
    // How long should each frame last?
    // TODO: make these AtomicU128s once that's stable
    frame_duration_micros: AtomicU64,
    // When should the next frame be displayed?
    // (in microseconds, elapsed time since the start)
    next_frame_micros: AtomicU64,
}

struct ServoWebSrcGfx {
    device: Device,
    context: Context,
    swap_chain: SwapChain<Device>,
    gl: Rc<Gl>,
    read_fbo: GLuint,
    draw_fbo: GLuint,
}

impl Drop for ServoWebSrcGfx {
    fn drop(&mut self) {
        self.gl.delete_framebuffers(&[self.read_fbo, self.draw_fbo]);
        let _ = self.device.destroy_context(&mut self.context);
    }
}

thread_local! {
    static GFX_CACHE: RefCell<HashMap<GLContext, ServoWebSrcGfx>> = RefCell::new(HashMap::new());
}

struct ConnectionWhichImplementsDebug(Connection);

impl std::fmt::Debug for ConnectionWhichImplementsDebug {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        "Connection".fmt(fmt)
    }
}

#[derive(Debug)]
enum ServoWebSrcMsg {
    Start(ConnectionWhichImplementsDebug, GLVersion, ServoUrl),
    GetSwapChain(Sender<SwapChain<Device>>),
    Resize(Size2D<i32, DevicePixel>),
    Heartbeat,
    Stop,
}

const DEFAULT_URL: &'static str =
    "https://rawcdn.githack.com/mrdoob/three.js/r105/examples/webgl_animation_cloth.html";

// Default framerate is 60fps
const DEFAULT_FRAME_DURATION: Duration = Duration::from_micros(16_667);

struct ServoThread {
    receiver: Receiver<ServoWebSrcMsg>,
    swap_chain: SwapChain<Device>,
    gfx: Rc<RefCell<ServoThreadGfx>>,
    servo: Servo<ServoWebSrcWindow>,
}

struct ServoThreadGfx {
    device: Device,
    context: Context,
    gl: Rc<Gl>,
}

impl ServoThread {
    fn new(receiver: Receiver<ServoWebSrcMsg>) -> Self {
        let (connection, version, url) = match receiver.recv() {
            Ok(ServoWebSrcMsg::Start(connection, version, url)) => (connection.0, version, url),
            e => panic!("Failed to start ({:?})", e),
        };
        info!(
            "Created new servo thread (GL v{}.{} for {})",
            version.major, version.minor, url
        );
        let embedder = Box::new(ServoWebSrcEmbedder);
        let window = Rc::new(ServoWebSrcWindow::new(connection, version));
        let swap_chain = window.swap_chain.clone();
        let gfx = window.gfx.clone();
        let mut servo = Servo::new(embedder, window);
        let id = TopLevelBrowsingContextId::new();
        servo.handle_events(vec![WindowEvent::NewBrowser(url, id)]);

        Self {
            receiver,
            swap_chain,
            gfx,
            servo,
        }
    }

    fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            debug!("Servo thread handling message {:?}", msg);
            match msg {
                ServoWebSrcMsg::Start(..) => error!("Already started"),
                ServoWebSrcMsg::GetSwapChain(sender) => sender
                    .send(self.swap_chain.clone())
                    .expect("Failed to send swap chain"),
                ServoWebSrcMsg::Resize(size) => self.resize(size),
                ServoWebSrcMsg::Heartbeat => self.servo.handle_events(vec![]),
                ServoWebSrcMsg::Stop => break,
            }
        }
        self.servo.handle_events(vec![WindowEvent::Quit]);
    }

    fn resize(&mut self, size: Size2D<i32, DevicePixel>) {
        {
            let mut gfx = self.gfx.borrow_mut();
            let gfx = &mut *gfx;
            self.swap_chain
                .resize(&mut gfx.device, &mut gfx.context, size.to_untyped())
                .expect("Failed to resize");
            gfx.gl.viewport(0, 0, size.width, size.height);
            let fbo = gfx
                .device
                .context_surface_info(&gfx.context)
                .expect("Failed to get context info")
                .expect("Failed to get context info")
                .framebuffer_object;
            gfx.device
                .make_context_current(&gfx.context)
                .expect("Failed to make current");
            gfx.gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
            debug_assert_eq!(
                (
                    gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gfx.gl.get_error()
                ),
                (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
            );
        }
        self.servo.handle_events(vec![WindowEvent::Resize]);
    }
}

impl Drop for ServoThread {
    fn drop(&mut self) {
        let mut gfx = self.gfx.borrow_mut();
        let gfx = &mut *gfx;
        self.swap_chain
            .destroy(&mut gfx.device, &mut gfx.context)
            .expect("Failed to destroy swap chain")
    }
}

struct ServoWebSrcEmbedder;

impl EmbedderMethods for ServoWebSrcEmbedder {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoWebSrcEmbedder)
    }
}

impl EventLoopWaker for ServoWebSrcEmbedder {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoWebSrcEmbedder)
    }

    fn wake(&self) {}
}

struct ServoWebSrcWindow {
    swap_chain: SwapChain<Device>,
    gfx: Rc<RefCell<ServoThreadGfx>>,
    gl: Rc<dyn gleam::gl::Gl>,
}

impl ServoWebSrcWindow {
    fn new(connection: Connection, version: GLVersion) -> Self {
        let flags = ContextAttributeFlags::DEPTH |
            ContextAttributeFlags::STENCIL |
            ContextAttributeFlags::ALPHA;
        let attributes = ContextAttributes { version, flags };

        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");
        let mut device = connection
            .create_device(&adapter)
            .expect("Failed to create device");
        let descriptor = device
            .create_context_descriptor(&attributes)
            .expect("Failed to create descriptor");
        let mut context = device
            .create_context(&descriptor)
            .expect("Failed to create context");

        let (gleam, gl) = unsafe {
            match device.gl_api() {
                GLApi::GL => (
                    gleam::gl::GlFns::load_with(|s| device.get_proc_address(&context, s)),
                    Gl::gl_fns(gl::ffi_gl::Gl::load_with(|s| {
                        device.get_proc_address(&context, s)
                    })),
                ),
                GLApi::GLES => (
                    gleam::gl::GlesFns::load_with(|s| device.get_proc_address(&context, s)),
                    Gl::gles_fns(gl::ffi_gles::Gles2::load_with(|s| {
                        device.get_proc_address(&context, s)
                    })),
                ),
            }
        };

        device
            .make_context_current(&mut context)
            .expect("Failed to make context current");
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);
        let access = SurfaceAccess::GPUOnly;
        let size = Size2D::new(512, 512);
        let surface_type = SurfaceType::Generic { size };
        let surface = device
            .create_surface(&mut context, access, surface_type)
            .expect("Failed to create surface");

        device
            .bind_surface_to_context(&mut context, surface)
            .expect("Failed to bind surface");
        let fbo = device
            .context_surface_info(&context)
            .expect("Failed to get context info")
            .expect("Failed to get context info")
            .framebuffer_object;
        gl.viewport(0, 0, size.width, size.height);
        gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        debug_assert_eq!(
            (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
            (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
        );

        let swap_chain = SwapChain::create_attached(&mut device, &mut context, access)
            .expect("Failed to create swap chain");

        device.make_no_context_current().unwrap();

        let gfx = Rc::new(RefCell::new(ServoThreadGfx {
            device,
            context,
            gl,
        }));

        Self {
            swap_chain,
            gfx,
            gl: gleam,
        }
    }
}

impl WindowMethods for ServoWebSrcWindow {
    fn present(&self) {
        debug!("EMBEDDER present");
        let mut gfx = self.gfx.borrow_mut();
        let gfx = &mut *gfx;
        gfx.device
            .make_context_current(&mut gfx.context)
            .expect("Failed to make context current");
        debug_assert_eq!(
            (
                gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gfx.gl.get_error()
            ),
            (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
        );
        self.swap_chain
            .swap_buffers(&mut gfx.device, &mut gfx.context)
            .expect("Failed to swap buffers");
        let fbo = gfx
            .device
            .context_surface_info(&gfx.context)
            .expect("Failed to get context info")
            .expect("Failed to get context info")
            .framebuffer_object;
        gfx.gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
        debug_assert_eq!(
            (
                gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gfx.gl.get_error()
            ),
            (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
        );
        let _ = gfx.device.make_no_context_current();
    }

    fn make_gl_context_current(&self) {
        debug!("EMBEDDER make_context_current");
        let mut gfx = self.gfx.borrow_mut();
        let gfx = &mut *gfx;
        gfx.device
            .make_context_current(&mut gfx.context)
            .expect("Failed to make context current");
        debug!("EMBEDDER done make_context_current");
        debug_assert_eq!(
            (
                gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                gfx.gl.get_error()
            ),
            (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
        );
    }

    fn gl(&self) -> Rc<dyn gleam::gl::Gl> {
        self.gl.clone()
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        let size = Size2D::from_untyped(self.swap_chain.size());
        info!("EMBEDDER coordinates {}", size);
        let origin = Point2D::origin();
        EmbedderCoordinates {
            hidpi_factor: Scale::new(1.0),
            screen: size,
            screen_avail: size,
            window: (size, origin),
            framebuffer: size,
            viewport: Rect::new(origin, size),
        }
    }

    fn set_animation_state(&self, _: AnimationState) {}

    fn get_gl_context(&self) -> servo_media::player::context::GlContext {
        servo_media::player::context::GlContext::Unknown
    }

    fn get_native_display(&self) -> servo_media::player::context::NativeDisplay {
        servo_media::player::context::NativeDisplay::Unknown
    }

    fn get_gl_api(&self) -> servo_media::player::context::GlApi {
        servo_media::player::context::GlApi::OpenGL3
    }
}

static PROPERTIES: [Property; 1] = [Property("url", |name| {
    ParamSpec::string(
        name,
        "URL",
        "Initial URL",
        Some(DEFAULT_URL),
        glib::ParamFlags::READWRITE,
    )
})];

const CAPS: &str = "video/x-raw(memory:GLMemory),
  format={RGBA,RGBx},
  width=[1,2147483647],
  height=[1,2147483647],
  framerate=[0/1,2147483647/1]";

impl ObjectSubclass for ServoWebSrc {
    const NAME: &'static str = "ServoWebSrc";
    // gstreamer-gl doesn't have support for GLBaseSrc yet
    // https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/issues/219
    type ParentType = BaseSrc;
    type Instance = ElementInstanceStruct<Self>;
    type Class = ClassStruct<Self>;

    fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        thread::spawn(move || ServoThread::new(receiver).run());
        let info = Mutex::new(None);
        let url = Mutex::new(None);
        let buffer_pool = Mutex::new(None);
        let gl_context = Mutex::new(None);
        let connection = Mutex::new(None);
        let start = Instant::now();
        let frame_duration_micros = AtomicU64::new(DEFAULT_FRAME_DURATION.as_micros() as u64);
        let next_frame_micros = AtomicU64::new(0);
        Self {
            sender,
            info,
            url,
            buffer_pool,
            gl_context,
            connection,
            start,
            frame_duration_micros,
            next_frame_micros,
        }
    }

    fn class_init(klass: &mut ClassStruct<Self>) {
        klass.set_metadata(
            "Servo as a gstreamer src",
            "Filter/Effect/Converter/Video",
            "The Servo web browser",
            env!("CARGO_PKG_AUTHORS"),
        );

        let src_caps = Caps::from_string(CAPS).unwrap();
        let src_pad_template =
            PadTemplate::new("src", PadDirection::Src, PadPresence::Always, &src_caps).unwrap();
        klass.add_pad_template(src_pad_template);
        klass.install_properties(&PROPERTIES);
    }

    glib_object_subclass!();
}

impl ObjectImpl for ServoWebSrc {
    glib_object_impl!();

    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);
        let basesrc = obj.downcast_ref::<BaseSrc>().unwrap();
        basesrc.set_live(true);
        basesrc.set_format(Format::Time);
        basesrc.set_do_timestamp(true);
    }

    fn set_property(&self, _obj: &Object, id: usize, value: &Value) {
        let prop = &PROPERTIES[id];
        match *prop {
            Property("url", ..) => {
                let mut guard = self.url.lock().expect("Failed to lock mutex");
                let url = value.get().expect("Failed to get url value");
                *guard = Some(url);
            },
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &Object, id: usize) -> Result<Value, ()> {
        let prop = &PROPERTIES[id];
        match *prop {
            Property("url", ..) => {
                let guard = self.url.lock().expect("Failed to lock mutex");
                Ok(Value::from(guard.as_ref()))
            },
            _ => unimplemented!(),
        }
    }
}

impl ElementImpl for ServoWebSrc {}

impl BaseSrcImpl for ServoWebSrc {
    fn set_caps(&self, _src: &BaseSrc, outcaps: &Caps) -> Result<(), LoggableError> {
        info!("Setting caps {:?}", outcaps);

        // Save the video info for later use
        let info = VideoInfo::from_caps(outcaps)
            .ok_or_else(|| gst_loggable_error!(CATEGORY, "Failed to get video info"))?;
        *self.info.lock().unwrap() = Some(info);

        // Save the framerate if it is set
        let framerate = outcaps
            .get_structure(0)
            .and_then(|cap| cap.get::<Fraction>("framerate"));
        if let Some(framerate) = framerate {
            let frame_duration_micros =
                1_000_000 * *framerate.denom() as u64 / *framerate.numer() as u64;
            debug!("Setting frame duration to {}micros", frame_duration_micros);
            self.frame_duration_micros
                .store(frame_duration_micros, Ordering::SeqCst);
        }

        // Create a new buffer pool for GL memory
        let gst_gl_context = self
            .gl_context
            .lock()
            .unwrap()
            .as_ref()
            .expect("Set caps before starting")
            .to_glib_none()
            .0;
        let gst_gl_buffer_pool =
            unsafe { gstreamer_gl_sys::gst_gl_buffer_pool_new(gst_gl_context) };
        if gst_gl_buffer_pool.is_null() {
            return Err(gst_loggable_error!(
                CATEGORY,
                "Failed to create buffer pool"
            ));
        }
        let pool = unsafe { BufferPool::from_glib_borrow(gst_gl_buffer_pool) };

        // Configure the buffer pool with the negotiated caps
        let mut config = pool.get_config();
        let (_, size, min_buffers, max_buffers) = config.get_params().unwrap_or((None, 0, 0, 1024));
        config.set_params(Some(outcaps), size, min_buffers, max_buffers);
        pool.set_config(config)
            .map_err(|_| gst_loggable_error!(CATEGORY, "Failed to update config"))?;

        // Save the buffer pool for later use
        *self.buffer_pool.lock().expect("Poisoned lock") = Some(pool);

        Ok(())
    }

    fn get_size(&self, _src: &BaseSrc) -> Option<u64> {
        u64::try_from(self.info.lock().ok()?.as_ref()?.size()).ok()
    }

    fn is_seekable(&self, _: &BaseSrc) -> bool {
        false
    }

    fn start(&self, src: &BaseSrc) -> Result<(), ErrorMessage> {
        info!("Starting");

        // Get the URL
        let url_guard = self
            .url
            .lock()
            .map_err(|_| gst_error_msg!(ResourceError::Settings, ["Failed to lock mutex"]))?;
        let url_string = url_guard.as_ref().map(|s| &**s).unwrap_or(DEFAULT_URL);
        let url = ServoUrl::parse(url_string)
            .map_err(|_| gst_error_msg!(ResourceError::Settings, ["Failed to parse url"]))?;

        // Get the downstream GL context
        let mut gst_gl_context = std::ptr::null_mut();
        let el = src.upcast_ref::<Element>();
        unsafe {
            gstreamer_gl_sys::gst_gl_query_local_gl_context(
                el.as_ptr(),
                gstreamer_sys::GST_PAD_SRC,
                &mut gst_gl_context,
            );
        }
        if gst_gl_context.is_null() {
            return Err(gst_error_msg!(
                ResourceError::Settings,
                ["Failed to get GL context"]
            ));
        }
        let gl_context = unsafe { GLContext::from_glib_borrow(gst_gl_context) };
        let gl_version = gl_context.get_gl_version();
        let version = GLVersion {
            major: gl_version.0 as u8,
            minor: gl_version.1 as u8,
        };

        // Get the surfman connection on the GL thread
        let mut task = BootstrapSurfmanOnGLThread {
            servo_web_src: self,
            result: None,
        };

        let data = &mut task as *mut BootstrapSurfmanOnGLThread as *mut c_void;
        unsafe {
            gst_gl_context_thread_add(gst_gl_context, Some(bootstrap_surfman_on_gl_thread), data)
        };
        let connection = task.result.expect("Failed to get connection");

        // Save the GL context and connection for later use
        *self.gl_context.lock().expect("Poisoned lock") = Some(gl_context);
        *self.connection.lock().expect("Poisoned lock") = Some(connection.clone());

        // Inform servo we're starting
        let _ = self.sender.send(ServoWebSrcMsg::Start(
            ConnectionWhichImplementsDebug(connection),
            version,
            url,
        ));
        Ok(())
    }

    fn stop(&self, _src: &BaseSrc) -> Result<(), ErrorMessage> {
        info!("Stopping");
        let _ = self.sender.send(ServoWebSrcMsg::Stop);
        Ok(())
    }

    fn create(&self, src: &BaseSrc, _offset: u64, _length: u32) -> Result<Buffer, FlowError> {
        // We block waiting for the next frame to be needed.
        // TODO: Once get_times is in BaseSrcImpl, we can use that instead.
        // It's been merged but not yet published.
        // https://github.com/servo/servo/issues/25234
        let elapsed_micros = self.start.elapsed().as_micros() as u64;
        let frame_duration_micros = self.frame_duration_micros.load(Ordering::SeqCst);
        let next_frame_micros = self
            .next_frame_micros
            .fetch_add(frame_duration_micros, Ordering::SeqCst);
        if elapsed_micros < next_frame_micros {
            // Delay by at most a second
            let delay = 1_000_000.min(next_frame_micros - elapsed_micros);
            debug!("Waiting for {}micros", delay);
            thread::sleep(Duration::from_micros(delay));
            debug!("Done waiting");
        }

        // Get the buffer pool
        let pool_guard = self.buffer_pool.lock().unwrap();
        let pool = pool_guard.as_ref().ok_or(FlowError::NotNegotiated)?;

        // Activate the pool if necessary
        if !pool.is_active() {
            debug!("Activating the buffer pool");
            pool.set_active(true).map_err(|_| FlowError::Error)?;
        }

        // Get a buffer to fill
        debug!("Acquiring a buffer");
        let buffer = pool.acquire_buffer(None)?;

        // Get the GL memory from the buffer
        let memory = buffer.get_all_memory().ok_or_else(|| {
            gst_element_error!(src, CoreError::Failed, ["Failed to get memory"]);
            FlowError::Error
        })?;
        let memory = unsafe { memory.into_ptr() };
        if unsafe { gst_is_gl_memory(memory) } == 0 {
            gst_element_error!(src, CoreError::Failed, ["Memory isn't GL memory"]);
            return Err(FlowError::Error);
        }
        let gl_memory = unsafe { (memory as *mut GstGLMemory).as_ref() }.ok_or_else(|| {
            gst_element_error!(src, CoreError::Failed, ["Memory is null"]);
            FlowError::Error
        })?;

        // Fill the buffer on the GL thread
        let result = Err(FlowError::Error);
        let mut task = FillOnGLThread {
            servo_web_src: self,
            src,
            gl_memory,
            result,
        };

        let data = &mut task as *mut FillOnGLThread as *mut c_void;
        unsafe { gst_gl_context_thread_add(gl_memory.mem.context, Some(fill_on_gl_thread), data) };
        task.result?;

        // Put down a GL sync point if needed
        if let Some(meta) = buffer.get_meta::<GLSyncMeta>() {
            let gl_context = unsafe { GLContext::from_glib_borrow(gl_memory.mem.context) };
            meta.set_sync_point(&gl_context);
        }

        // Wake up Servo
        let _ = self.sender.send(ServoWebSrcMsg::Heartbeat);
        Ok(buffer)
    }
}

struct BootstrapSurfmanOnGLThread<'a> {
    servo_web_src: &'a ServoWebSrc,
    result: Option<Connection>,
}

unsafe extern "C" fn bootstrap_surfman_on_gl_thread(context: *mut GstGLContext, data: *mut c_void) {
    let task = &mut *(data as *mut BootstrapSurfmanOnGLThread);
    let gl_context = GLContext::from_glib_borrow(context);
    task.result = task.servo_web_src.bootstrap_surfman(gl_context);
}

impl ServoWebSrc {
    // Runs on the GL thread
    fn bootstrap_surfman(&self, gl_context: GLContext) -> Option<Connection> {
        gl_context
            .activate(true)
            .expect("Failed to activate GL context");
        let native_connection =
            NativeConnection::current().expect("Failed to bootstrap native connection");
        let connection = unsafe { Connection::from_native_connection(native_connection) }
            .expect("Failed to bootstrap surfman connection");
        Some(connection)
    }
}

struct FillOnGLThread<'a> {
    servo_web_src: &'a ServoWebSrc,
    src: &'a BaseSrc,
    gl_memory: &'a GstGLMemory,
    result: Result<(), FlowError>,
}

unsafe extern "C" fn fill_on_gl_thread(context: *mut GstGLContext, data: *mut c_void) {
    let task = &mut *(data as *mut FillOnGLThread);
    let gl_context = GLContext::from_glib_borrow(context);
    task.result = task
        .servo_web_src
        .fill_gl_memory(task.src, gl_context, task.gl_memory);
}

impl ServoWebSrc {
    // Runs on the GL thread
    fn fill_gl_memory(
        &self,
        src: &BaseSrc,
        gl_context: GLContext,
        gl_memory: &GstGLMemory,
    ) -> Result<(), FlowError> {
        // Get the data out of the memory
        let draw_texture_id = gl_memory.tex_id;
        let draw_texture_target = unsafe { gst_gl_texture_target_to_gl(gl_memory.tex_target) };
        let height = gl_memory.info.height;
        let width = gl_memory.info.width;
        let size = Size2D::new(width, height);
        debug!("Filling texture {} {}x{}", draw_texture_id, width, height);

        gl_context.activate(true).map_err(|_| {
            gst_element_error!(src, CoreError::Failed, ["Failed to activate GL context"]);
            FlowError::Error
        })?;

        GFX_CACHE.with(|gfx_cache| {
            let mut gfx_cache = gfx_cache.borrow_mut();
            let gfx = gfx_cache.entry(gl_context.clone()).or_insert_with(|| {
                debug!("Bootstrapping surfman");
                let connection_guard = self.connection.lock().unwrap();
                let connection = connection_guard.as_ref().expect("Failed to get surfman");
                let adapter = connection
                    .create_adapter()
                    .expect("Failed to bootstrap surfman adapter");
                let device = connection
                    .create_device(&adapter)
                    .expect("Failed to bootstrap surfman device");
                let native_context =
                    NativeContext::current().expect("Failed to bootstrap native context");
                let context = unsafe {
                    device
                        .create_context_from_native_context(native_context)
                        .expect("Failed to bootstrap surfman context")
                };

                debug!("Creating GL bindings");
                let gl = Gl::gl_fns(gl::ffi_gl::Gl::load_with(|s| {
                    gl_context.get_proc_address(s) as *const _
                }));
                let draw_fbo = gl.gen_framebuffers(1)[0];
                let read_fbo = gl.gen_framebuffers(1)[0];

                debug!("Getting the swap chain");
                let (acks, ackr) = crossbeam_channel::bounded(1);
                let _ = self.sender.send(ServoWebSrcMsg::GetSwapChain(acks));
                let swap_chain = ackr.recv().expect("Failed to get swap chain");

                ServoWebSrcGfx {
                    device,
                    context,
                    swap_chain,
                    gl,
                    read_fbo,
                    draw_fbo,
                }
            });

            gfx.device
                .make_context_current(&gfx.context)
                .expect("Failed to make surfman context current");
            debug_assert_eq!(gfx.gl.get_error(), gl::NO_ERROR);

            // Save the current GL state
            debug!("Saving the GL context");
            let mut bound_fbos = [0, 0];
            unsafe {
                gfx.gl
                    .get_integer_v(gl::DRAW_FRAMEBUFFER_BINDING, &mut bound_fbos[0..]);
                gfx.gl
                    .get_integer_v(gl::READ_FRAMEBUFFER_BINDING, &mut bound_fbos[1..]);
            }

            gfx.gl.bind_framebuffer(gl::FRAMEBUFFER, gfx.draw_fbo);
            gfx.gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                draw_texture_target,
                draw_texture_id,
                0,
            );
            debug_assert_eq!(gfx.gl.get_error(), gl::NO_ERROR);

            gfx.gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gfx.gl.clear(gl::COLOR_BUFFER_BIT);
            debug_assert_eq!(gfx.gl.get_error(), gl::NO_ERROR);

            if let Some(surface) = gfx.swap_chain.take_surface() {
                debug!("Rendering surface");
                let surface_size = Size2D::from_untyped(gfx.device.surface_info(&surface).size);
                if size != surface_size {
                    // If we're being asked to fill frames that are a different size than servo is providing,
                    // ask it to change size.
                    let _ = self.sender.send(ServoWebSrcMsg::Resize(size));
                }

                if size.width <= 0 || size.height <= 0 {
                    info!("Surface is zero-sized");
                    gfx.swap_chain.recycle_surface(surface);
                    return;
                }

                let surface_texture = gfx
                    .device
                    .create_surface_texture(&mut gfx.context, surface)
                    .unwrap();
                let read_texture_id = gfx.device.surface_texture_object(&surface_texture);
                let read_texture_target = gfx.device.surface_gl_texture_target();

                debug!(
                    "Filling with {}/{} {}",
                    read_texture_id, read_texture_target, surface_size
                );
                gfx.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, gfx.read_fbo);
                gfx.gl.framebuffer_texture_2d(
                    gl::READ_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    read_texture_target,
                    read_texture_id,
                    0,
                );
                gfx.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, gfx.draw_fbo);
                gfx.gl.framebuffer_texture_2d(
                    gl::DRAW_FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    draw_texture_target,
                    draw_texture_id,
                    0,
                );
                debug_assert_eq!(
                    (
                        gfx.gl.check_framebuffer_status(gl::READ_FRAMEBUFFER),
                        gfx.gl.check_framebuffer_status(gl::DRAW_FRAMEBUFFER),
                        gfx.gl.get_error()
                    ),
                    (
                        gl::FRAMEBUFFER_COMPLETE,
                        gl::FRAMEBUFFER_COMPLETE,
                        gl::NO_ERROR
                    )
                );

                debug!("Blitting");
                gfx.gl.blit_framebuffer(
                    0,
                    0,
                    surface_size.width,
                    surface_size.height,
                    0,
                    0,
                    width,
                    height,
                    gl::COLOR_BUFFER_BIT,
                    gl::NEAREST,
                );
                debug_assert_eq!(
                    (
                        gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                        gfx.gl.get_error()
                    ),
                    (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
                );

                let surface = gfx
                    .device
                    .destroy_surface_texture(&mut gfx.context, surface_texture)
                    .unwrap();
                gfx.swap_chain.recycle_surface(surface);
            } else {
                debug!("Failed to get current surface");
            }

            // Restore the GL state
            gfx.gl
                .bind_framebuffer(gl::DRAW_FRAMEBUFFER, bound_fbos[0] as GLuint);
            gfx.gl
                .bind_framebuffer(gl::READ_FRAMEBUFFER, bound_fbos[1] as GLuint);
            debug_assert_eq!(gfx.gl.get_error(), gl::NO_ERROR);
        });

        Ok(())
    }
}

// TODO: Implement that trait for more platforms
