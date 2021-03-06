/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::logging::CATEGORY;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;

use euclid::default::Rotation3D;
use euclid::default::Vector3D;
use euclid::Point2D;
use euclid::Rect;
use euclid::Scale;
use euclid::Size2D;

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
use glib::value::Value;
use glib::ParamSpec;
use gstreamer::gst_element_error;
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
use log::warn;

use servo::compositing::windowing::AnimationState;
use servo::compositing::windowing::EmbedderCoordinates;
use servo::compositing::windowing::EmbedderMethods;
use servo::compositing::windowing::WindowEvent;
use servo::compositing::windowing::WindowMethods;
use servo::embedder_traits::EmbedderProxy;
use servo::embedder_traits::EventLoopWaker;
use servo::msg::constellation_msg::TopLevelBrowsingContextId;
use servo::servo_config::prefs::add_user_prefs;
use servo::servo_config::prefs::read_prefs_map;
use servo::servo_config::prefs::PrefValue;
use servo::servo_config::set_pref;
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::DevicePixel;
use servo::webrender_surfman::WebrenderSurfman;
use servo::Servo;

use sparkle::gl;
use sparkle::gl::types::GLuint;
use sparkle::gl::Gl;

use surfman::Connection;
use surfman::Context;
use surfman::Device;
use surfman::SurfaceAccess;
use surfman::SurfaceType;
use surfman_chains::SwapChain;
use surfman_chains_api::SwapChainAPI;

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::c_void;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use webxr::glwindow::GlWindow as WebXRWindow;
use webxr::glwindow::GlWindowDiscovery as WebXRDiscovery;
use webxr::glwindow::GlWindowMode as WebXRMode;
use webxr::glwindow::GlWindowRenderTarget as WebXRRenderTarget;

pub struct ServoWebSrc {
    sender: Sender<ServoWebSrcMsg>,
    url: Mutex<Option<String>>,
    webxr_mode: Mutex<Option<WebXRMode>>,
    prefs: Mutex<Option<String>>,
    outcaps: Mutex<Option<Caps>>,
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
    swap_chain: Option<SwapChain<Device>>,
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

struct SwapChainWhichImplementsDebug(SwapChain<Device>);

impl std::fmt::Debug for SwapChainWhichImplementsDebug {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        "SwapChain".fmt(fmt)
    }
}

#[derive(Debug)]
enum ServoWebSrcMsg {
    Start(
        ConnectionWhichImplementsDebug,
        ServoUrl,
        Option<WebXRMode>,
        HashMap<String, PrefValue>,
        Size2D<i32, DevicePixel>,
    ),
    GetSwapChain(Sender<SwapChain<Device>>),
    SetSwapChain(SwapChainWhichImplementsDebug),
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
    servo: Servo<ServoWebSrcWindow>,
    swap_chain: Option<SwapChain<Device>>,
}

impl ServoThread {
    fn new(sender: Sender<ServoWebSrcMsg>, receiver: Receiver<ServoWebSrcMsg>) -> Self {
        let (connection, url, webxr_mode, prefs, size) = match receiver.recv() {
            Ok(ServoWebSrcMsg::Start(connection, url, webxr_mode, prefs, size)) => {
                (connection.0, url, webxr_mode, prefs, size)
            },
            e => panic!("Failed to start ({:?})", e),
        };
        info!(
            "Created new servo thread for {} ({:?}, {:?})",
            url, webxr_mode, prefs
        );
        let window = Rc::new(ServoWebSrcWindow::new(connection, webxr_mode, sender, size));
        let embedder = Box::new(ServoWebSrcEmbedder::new(&window));
        let webrender_swap_chain = window
            .webrender_surfman
            .swap_chain()
            .expect("Failed to get webrender swap chain")
            .clone();
        let mut servo = Servo::new(embedder, window, None);

        let id = TopLevelBrowsingContextId::new();
        servo.handle_events(vec![WindowEvent::NewBrowser(url, id)]);

        let swap_chain = match webxr_mode {
            None => Some(webrender_swap_chain),
            Some(..) => {
                set_pref!(dom.webxr.sessionavailable, true);
                set_pref!(dom.webxr.unsafe_assume_user_intent, true);
                servo.handle_events(vec![WindowEvent::ChangeBrowserVisibility(id, false)]);
                None
            },
        };

        add_user_prefs(prefs);

        Self {
            receiver,
            servo,
            swap_chain,
        }
    }

    fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            debug!("Servo thread handling message {:?}", msg);
            match msg {
                ServoWebSrcMsg::Start(..) => error!("Already started"),
                ServoWebSrcMsg::GetSwapChain(sender) => self.send_swap_chain(sender),
                ServoWebSrcMsg::SetSwapChain(swap_chain) => self.swap_chain = Some(swap_chain.0),
                ServoWebSrcMsg::Resize(size) => self.resize(size),
                ServoWebSrcMsg::Heartbeat => {
                    self.servo.handle_events(vec![]);
                },
                ServoWebSrcMsg::Stop => break,
            }
        }
        self.servo.handle_events(vec![WindowEvent::Quit]);
    }

    fn send_swap_chain(&mut self, sender: Sender<SwapChain<Device>>) {
        if let Some(ref swap_chain) = self.swap_chain {
            debug!("Sending swap chain");
            let _ = sender.send(swap_chain.clone());
        }
    }

    fn resize(&mut self, size: Size2D<i32, DevicePixel>) {
        debug!("Servo resized to {:?}", size);
        let _ = self
            .servo
            .window()
            .webrender_surfman
            .resize(size.to_untyped());
        self.servo.handle_events(vec![WindowEvent::Resize]);
    }
}

struct ServoWebSrcEmbedder {
    webrender_surfman: WebrenderSurfman,
    webxr_mode: Option<WebXRMode>,
    sender: Sender<ServoWebSrcMsg>,
}

impl ServoWebSrcEmbedder {
    fn new(window: &ServoWebSrcWindow) -> ServoWebSrcEmbedder {
        let webxr_mode = window.webxr_mode;
        let sender = window.sender.clone();
        let webrender_surfman = window.webrender_surfman.clone();
        ServoWebSrcEmbedder {
            webxr_mode,
            webrender_surfman,
            sender,
        }
    }
}

impl EmbedderMethods for ServoWebSrcEmbedder {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoWebSrcEventLoopWaker)
    }

    fn register_webxr(&mut self, registry: &mut webxr::MainThreadRegistry, _: EmbedderProxy) {
        let connection = self.webrender_surfman.connection();
        let adapter = self.webrender_surfman.adapter();
        let context_attributes = self.webrender_surfman.context_attributes();
        let sender = self.sender.clone();
        let webrender_surfman = self.webrender_surfman.clone();
        let webxr_mode = self.webxr_mode;
        let factory = Box::new(move || {
            ServoWebSrcWebXR::new(webrender_surfman.clone(), webxr_mode, sender.clone())
        });
        let discovery = WebXRDiscovery::new(connection, adapter, context_attributes, factory);
        registry.register(discovery);
    }
}

struct ServoWebSrcEventLoopWaker;

impl EventLoopWaker for ServoWebSrcEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoWebSrcEventLoopWaker)
    }

    fn wake(&self) {}
}

struct ServoWebSrcWindow {
    webrender_surfman: WebrenderSurfman,
    webxr_mode: Option<WebXRMode>,
    sender: Sender<ServoWebSrcMsg>,
}

impl ServoWebSrcWindow {
    fn new(
        connection: Connection,
        webxr_mode: Option<WebXRMode>,
        sender: Sender<ServoWebSrcMsg>,
        size: Size2D<i32, DevicePixel>,
    ) -> Self {
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");
        let size = size.to_untyped();
        let surface_type = SurfaceType::Generic { size };
        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create surfman");

        Self {
            webrender_surfman,
            webxr_mode,
            sender,
        }
    }
}

impl WindowMethods for ServoWebSrcWindow {
    fn webrender_surfman(&self) -> WebrenderSurfman {
        self.webrender_surfman.clone()
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        let size = self
            .webrender_surfman
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| Size2D::from_untyped(info.size))
            .unwrap_or(Size2D::new(0, 0));
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

struct ServoWebSrcWebXR {
    webrender_surfman: WebrenderSurfman,
    webxr_mode: Option<WebXRMode>,
    sender: Sender<ServoWebSrcMsg>,
}

impl ServoWebSrcWebXR {
    fn new(
        webrender_surfman: WebrenderSurfman,
        webxr_mode: Option<WebXRMode>,
        sender: Sender<ServoWebSrcMsg>,
    ) -> Result<Box<dyn WebXRWindow>, ()> {
        Ok(Box::new(ServoWebSrcWebXR {
            webrender_surfman,
            webxr_mode,
            sender,
        }))
    }
}

impl WebXRWindow for ServoWebSrcWebXR {
    fn get_mode(&self) -> WebXRMode {
        self.webxr_mode.unwrap()
    }

    fn get_render_target(&self, device: &mut Device, context: &mut Context) -> WebXRRenderTarget {
        log::debug!("Creating webxr render target");
        let size = self
            .webrender_surfman
            .context_surface_info()
            .expect("Failed to get webrender size")
            .expect("Failed to get webrender size")
            .size;
        let surface_access = SurfaceAccess::GPUOnly;
        let surface_type = SurfaceType::Generic { size };
        let surface = device
            .create_surface(context, surface_access, surface_type)
            .expect("Failed to create target surface");
        device
            .bind_surface_to_context(context, surface)
            .expect("Failed to bind target surface");
        let webxr_swap_chain = SwapChain::create_attached(device, context, surface_access)
            .expect("Failed to create target swap chain");

        log::debug!("Created webxr render target {:?}", size);
        let _ = self
            .sender
            .send(ServoWebSrcMsg::SetSwapChain(SwapChainWhichImplementsDebug(
                webxr_swap_chain.clone(),
            )));
        WebXRRenderTarget::SwapChain(webxr_swap_chain)
    }

    fn get_rotation(&self) -> Rotation3D<f32> {
        Rotation3D::identity()
    }

    fn get_translation(&self) -> Vector3D<f32> {
        Vector3D::zero()
    }
}

static PROPERTIES: [Property; 3] = [
    Property("prefs", |name| {
        ParamSpec::string(
            name,
            "prefs",
            "Servo preferences",
            None,
            glib::ParamFlags::READWRITE,
        )
    }),
    Property("url", |name| {
        ParamSpec::string(
            name,
            "URL",
            "Initial URL",
            Some(DEFAULT_URL),
            glib::ParamFlags::READWRITE,
        )
    }),
    Property("webxr", |name| {
        ParamSpec::string(
            name,
            "WebXR",
            "Stream immersive WebXR content",
            None,
            glib::ParamFlags::READWRITE,
        )
    }),
];

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
        let (sender, receiver) = crossbeam_channel::unbounded();
        let sender_clone = sender.clone();
        thread::spawn(move || ServoThread::new(sender_clone, receiver).run());
        let info = Mutex::new(None);
        let outcaps = Mutex::new(None);
        let url = Mutex::new(None);
        let prefs = Mutex::new(None);
        let webxr_mode = Mutex::new(None);
        let buffer_pool = Mutex::new(None);
        let gl_context = Mutex::new(None);
        let connection = Mutex::new(None);
        let start = Instant::now();
        let frame_duration_micros = AtomicU64::new(DEFAULT_FRAME_DURATION.as_micros() as u64);
        let next_frame_micros = AtomicU64::new(0);
        Self {
            sender,
            info,
            outcaps,
            url,
            prefs,
            webxr_mode,
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

        let src_caps = Caps::from_str(CAPS).unwrap();
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
            Property("prefs", ..) => {
                let mut guard = self.prefs.lock().expect("Failed to lock mutex");
                let prefs = value.get().expect("Failed to get prefs value");
                *guard = prefs;
            },
            Property("url", ..) => {
                let mut guard = self.url.lock().expect("Failed to lock mutex");
                let url = value.get().expect("Failed to get url value");
                *guard = url;
            },
            Property("webxr", ..) => {
                let mut guard = self.webxr_mode.lock().expect("Failed to lock mutex");
                let webxr_mode = match value.get().expect("Failed to get url value") {
                    None | Some("none") => None,
                    Some("blit") => Some(WebXRMode::Blit),
                    Some("left-right") => Some(WebXRMode::StereoLeftRight),
                    Some("red-cyan") => Some(WebXRMode::StereoRedCyan),
                    Some("cubemap") => Some(WebXRMode::Cubemap),
                    Some("spherical") => Some(WebXRMode::Spherical),
                    Some(mode) => panic!("Unknown WebXR mode {}", mode),
                };
                *guard = webxr_mode;
            },
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &Object, id: usize) -> Result<Value, ()> {
        let prop = &PROPERTIES[id];
        match *prop {
            Property("prefs", ..) => {
                let guard = self.url.lock().expect("Failed to lock mutex");
                Ok(Value::from(guard.as_ref()))
            },
            Property("url", ..) => {
                let guard = self.url.lock().expect("Failed to lock mutex");
                Ok(Value::from(guard.as_ref()))
            },
            Property("webxr", ..) => {
                let guard = self.webxr_mode.lock().expect("Failed to lock mutex");
                let webxr_mode = match guard.as_ref() {
                    Some(WebXRMode::Blit) => Some("blit"),
                    Some(WebXRMode::StereoLeftRight) => Some("left-right"),
                    Some(WebXRMode::StereoRedCyan) => Some("red-cyan"),
                    Some(WebXRMode::Cubemap) => Some("cubemap"),
                    Some(WebXRMode::Spherical) => Some("spherical"),
                    None => None,
                };
                Ok(Value::from(webxr_mode))
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
            .map_err(|_| gst_loggable_error!(CATEGORY, "Failed to get video info"))?;
        *self.info.lock().unwrap() = Some(info);

        // Save the framerate if it is set
        let framerate = outcaps
            .get_structure(0)
            .and_then(|cap| cap.get::<Fraction>("framerate").ok());
        if let Some(Some(framerate)) = framerate {
            let frame_duration_micros =
                1_000_000 * *framerate.denom() as u64 / *framerate.numer() as u64;
            debug!("Setting frame duration to {}micros", frame_duration_micros);
            self.frame_duration_micros
                .store(frame_duration_micros, Ordering::SeqCst);
        }

        // Save the caps for later use
        *self.outcaps.lock().expect("Poisoned mutex") = Some(outcaps.copy());

        Ok(())
    }

    fn get_size(&self, _src: &BaseSrc) -> Option<u64> {
        u64::try_from(self.info.lock().ok()?.as_ref()?.size()).ok()
    }

    fn is_seekable(&self, _: &BaseSrc) -> bool {
        false
    }

    fn start(&self, _: &BaseSrc) -> Result<(), ErrorMessage> {
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
        self.ensure_gl(src)?;
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

        // Put down a GL sync point if needed
        if let Some(meta) = buffer.get_meta::<GLSyncMeta>() {
            let gl_context = unsafe { GLContext::from_glib_borrow(gl_memory.mem.context) };
            meta.set_sync_point(&gl_context);
        }

        // Wake up Servo
        debug!("Sending heartbeat");
        let _ = self.sender.send(ServoWebSrcMsg::Heartbeat);

        task.result?;
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
    // Create the GL state if necessary
    fn ensure_gl(&self, src: &BaseSrc) -> Result<(), FlowError> {
        if self.gl_context.lock().expect("Poisoned lock").is_some() {
            return Ok(());
        }

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
            error!("Failed to get GL context");
            return Err(FlowError::Error);
        }
        let gl_context = unsafe { GLContext::from_glib_borrow(gst_gl_context) };
        *self.gl_context.lock().expect("Poisoned lock") = Some(gl_context);

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
        *self.connection.lock().expect("Poisoned lock") = Some(connection.clone());

        // Inform servo we're starting
        let url_guard = self.url.lock().expect("Poisoned mutex");
        let url_string = url_guard.as_ref().map(|s| &**s).unwrap_or(DEFAULT_URL);
        let url = ServoUrl::parse(url_string).map_err(|e| {
            error!("Failed to parse url {} ({:?})", url_string, e);
            FlowError::Error
        })?;
        let prefs_guard = self.prefs.lock().expect("Poisoned mutex");
        let prefs_string = prefs_guard.as_ref().map(|s| &**s).unwrap_or("{}");
        let prefs = read_prefs_map(prefs_string).map_err(|e| {
            error!("Failed to parse prefs {} ({:?})", prefs_string, e);
            FlowError::Error
        })?;
        let size = self
            .info
            .lock()
            .expect("Poisoned mutex")
            .as_ref()
            .map(|info| Size2D::new(info.width() as i32, info.height() as i32))
            .unwrap_or(Size2D::new(512, 512));
        let webxr_mode = *self.webxr_mode.lock().expect("Poisoned mutex");
        let _ = self.sender.send(ServoWebSrcMsg::Start(
            ConnectionWhichImplementsDebug(connection),
            url,
            webxr_mode,
            prefs,
            size,
        ));

        // Create a new buffer pool for GL memory
        let gst_gl_buffer_pool =
            unsafe { gstreamer_gl_sys::gst_gl_buffer_pool_new(gst_gl_context) };
        if gst_gl_buffer_pool.is_null() {
            error!("Failed to create buffer pool");
            return Err(FlowError::Error);
        }
        let pool = unsafe { BufferPool::from_glib_borrow(gst_gl_buffer_pool) };

        // Configure the buffer pool with the negotiated caps
        let mut config = pool.get_config();
        let (_, size, min_buffers, max_buffers) = config.get_params().unwrap_or((None, 0, 0, 1024));
        let outcaps = self.outcaps.lock().expect("Poisoned mutex");
        config.set_params(outcaps.as_ref(), size, min_buffers, max_buffers);
        pool.set_config(config).map_err(|e| {
            error!("Failed to parse url {:?}", e);
            FlowError::Error
        })?;
        *self.buffer_pool.lock().expect("Poisoned lock") = Some(pool);

        Ok(())
    }
}

impl ServoWebSrc {
    // Runs on the GL thread
    fn bootstrap_surfman(&self, gl_context: GLContext) -> Option<Connection> {
        gl_context
            .activate(true)
            .expect("Failed to activate GL context");
        // TODO: support other connections on linux?
        #[cfg(target_os = "linux")]
        {
            use surfman::platform::generic::multi;
            use surfman::platform::unix::wayland;
            let native_connection = wayland::connection::NativeConnection::current()
                .expect("Failed to bootstrap native connection");
            let wayland_connection = unsafe {
                wayland::connection::Connection::from_native_connection(native_connection)
                    .expect("Failed to bootstrap wayland connection")
            };
            let connection = multi::connection::Connection::Default(
                multi::connection::Connection::Default(wayland_connection),
            );
            Some(connection)
        }
        #[cfg(not(target_os = "linux"))]
        {
            use surfman::connection::Connection as ConnectionAPI;
            type NativeConnection = <Connection as ConnectionAPI>::NativeConnection;
            let native_connection =
                NativeConnection::current().expect("Failed to bootstrap native connection");
            let connection = unsafe { Connection::from_native_connection(native_connection) }
                .expect("Failed to bootstrap surfman connection");
            Some(connection)
        }
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
                #[cfg(target_os = "linux")]
                let native_context = {
                    use surfman::platform::generic::multi;
                    use surfman::platform::unix::wayland;
                    multi::context::NativeContext::Default(multi::context::NativeContext::Default(
                        wayland::context::NativeContext::current()
                            .expect("Failed to bootstrap native context"),
                    ))
                };
                #[cfg(not(target_os = "linux"))]
                let native_context = {
                    use surfman::device::Device as DeviceAPI;
                    type NativeContext = <Device as DeviceAPI>::NativeContext;
                    NativeContext::current().expect("Failed to bootstrap native context")
                };
                let context = unsafe {
                    device
                        .create_context_from_native_context(native_context)
                        .expect("Failed to bootstrap surfman context")
                };

                let swap_chain = None;

                debug!("Creating GL bindings");
                let gl = Gl::gl_fns(gl::ffi_gl::Gl::load_with(|s| {
                    gl_context.get_proc_address(s) as *const _
                }));
                let draw_fbo = gl.gen_framebuffers(1)[0];
                let read_fbo = gl.gen_framebuffers(1)[0];

                ServoWebSrcGfx {
                    device,
                    context,
                    swap_chain,
                    gl,
                    read_fbo,
                    draw_fbo,
                }
            });

            if gfx.swap_chain.is_none() {
                debug!("Getting the swap chain");
                let (acks, ackr) = crossbeam_channel::unbounded();
                let _ = self.sender.send(ServoWebSrcMsg::GetSwapChain(acks));
                gfx.swap_chain = ackr.recv_timeout(Duration::from_millis(16)).ok();
            }

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

            if let Some((swap_chain, surface)) = gfx.swap_chain.as_ref().and_then(|swap_chain| {
                swap_chain
                    .take_surface()
                    .map(|surface| (swap_chain, surface))
            }) {
                debug!("Rendering surface");
                let surface_size = Size2D::from_untyped(gfx.device.surface_info(&surface).size);
                if size != surface_size {
                    // If we're being asked to fill frames that are a different size than servo is providing,
                    // ask it to change size.
                    let _ = self.sender.send(ServoWebSrcMsg::Resize(size));
                }

                if size.width <= 0 || size.height <= 0 {
                    info!("Surface is zero-sized");
                    swap_chain.recycle_surface(surface);
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
                swap_chain.recycle_surface(surface);
            } else {
                warn!("Failed to get current surface");
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
