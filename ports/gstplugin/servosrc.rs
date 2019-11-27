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
use glib::subclass::object::ObjectClassSubclassExt;
use glib::subclass::object::ObjectImpl;
use glib::subclass::object::ObjectImplExt;
use glib::subclass::object::Property;
use glib::subclass::simple::ClassStruct;
use glib::subclass::types::ObjectSubclass;
use glib::value::Value;
use glib::ParamSpec;
use gstreamer::gst_element_error;
use gstreamer::gst_error_msg;
use gstreamer::gst_loggable_error;
use gstreamer::subclass::element::ElementClassSubclassExt;
use gstreamer::subclass::element::ElementImpl;
use gstreamer::subclass::ElementInstanceStruct;
use gstreamer::BufferRef;
use gstreamer::Caps;
use gstreamer::CoreError;
use gstreamer::ErrorMessage;
use gstreamer::FlowError;
use gstreamer::FlowSuccess;
use gstreamer::Format;
use gstreamer::Fraction;
use gstreamer::FractionRange;
use gstreamer::IntRange;
use gstreamer::LoggableError;
use gstreamer::PadDirection;
use gstreamer::PadPresence;
use gstreamer::PadTemplate;
use gstreamer::ResourceError;
use gstreamer_base::subclass::base_src::BaseSrcImpl;
use gstreamer_base::BaseSrc;
use gstreamer_base::BaseSrcExt;
use gstreamer_video::VideoFormat;
use gstreamer_video::VideoFrameRef;
use gstreamer_video::VideoInfo;

use log::debug;
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

use surfman::platform::generic::universal::context::Context;
use surfman::platform::generic::universal::device::Device;
use surfman::SurfaceAccess;
use surfman::SurfaceType;

use surfman_chains::SwapChain;
use surfman_chains_api::SwapChainAPI;

use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

pub struct ServoSrc {
    sender: Sender<ServoSrcMsg>,
    swap_chain: SwapChain,
    url: Mutex<Option<String>>,
    info: Mutex<Option<VideoInfo>>,
}

struct ServoSrcGfx {
    device: Device,
    context: Context,
    gl: Rc<Gl>,
    fbo: GLuint,
}

impl ServoSrcGfx {
    fn new() -> ServoSrcGfx {
        let version = surfman::GLVersion { major: 4, minor: 3 };
        let flags = surfman::ContextAttributeFlags::empty();
        let attributes = surfman::ContextAttributes { version, flags };

        let connection = surfman::Connection::new().expect("Failed to create connection");
        let adapter = surfman::Adapter::default().expect("Failed to create adapter");
        let mut device =
            surfman::Device::new(&connection, &adapter).expect("Failed to create device");
        let descriptor = device
            .create_context_descriptor(&attributes)
            .expect("Failed to create descriptor");
        let context = device
            .create_context(&descriptor)
            .expect("Failed to create context");
        let gl = Gl::gl_fns(gl::ffi_gl::Gl::load_with(|s| {
            device.get_proc_address(&context, s)
        }));

        // This is a workaround for surfman having a different bootstrap API with Angle
        #[cfg(target_os = "windows")]
        let mut device = device;
        #[cfg(not(target_os = "windows"))]
        let mut device = Device::Hardware(device);
        #[cfg(target_os = "windows")]
        let mut context = context;
        #[cfg(not(target_os = "windows"))]
        let mut context = Context::Hardware(context);

        device.make_context_current(&context).unwrap();

        let size = Size2D::new(512, 512);
        let surface_type = SurfaceType::Generic { size };
        let surface = device
            .create_surface(&mut context, SurfaceAccess::GPUCPU, &surface_type)
            .expect("Failed to create surface");

        gl.viewport(0, 0, size.width, size.height);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        device
            .bind_surface_to_context(&mut context, surface)
            .expect("Failed to bind surface");
        let fbo = device
            .context_surface_info(&context)
            .expect("Failed to get context info")
            .expect("Failed to get context info")
            .framebuffer_object;
        gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
        debug_assert_eq!(
            (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
            (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
        );

        let fbo = gl.gen_framebuffers(1)[0];
        debug_assert_eq!(
            (gl.check_framebuffer_status(gl::FRAMEBUFFER), gl.get_error()),
            (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
        );

        device.make_no_context_current().unwrap();

        Self {
            device,
            context,
            gl,
            fbo,
        }
    }
}

impl Drop for ServoSrcGfx {
    fn drop(&mut self) {
        let _ = self.device.destroy_context(&mut self.context);
    }
}

thread_local! {
    static GFX: RefCell<ServoSrcGfx> = RefCell::new(ServoSrcGfx::new());
}

#[derive(Debug)]
enum ServoSrcMsg {
    Start(ServoUrl),
    GetSwapChain(Sender<SwapChain>),
    Resize(Size2D<i32, DevicePixel>),
    Heartbeat,
    Stop,
}

const DEFAULT_URL: &'static str =
    "https://rawcdn.githack.com/mrdoob/three.js/r105/examples/webgl_animation_cloth.html";

struct ServoThread {
    receiver: Receiver<ServoSrcMsg>,
    swap_chain: SwapChain,
    servo: Servo<ServoSrcWindow>,
}

impl ServoThread {
    fn new(receiver: Receiver<ServoSrcMsg>) -> Self {
        let embedder = Box::new(ServoSrcEmbedder);
        let window = Rc::new(ServoSrcWindow::new());
        let swap_chain = window.swap_chain.clone();
        let servo = Servo::new(embedder, window);
        Self {
            receiver,
            swap_chain,
            servo,
        }
    }

    fn run(&mut self) {
        while let Ok(msg) = self.receiver.recv() {
            debug!("Servo thread handling message {:?}", msg);
            match msg {
                ServoSrcMsg::Start(url) => self.new_browser(url),
                ServoSrcMsg::GetSwapChain(sender) => sender
                    .send(self.swap_chain.clone())
                    .expect("Failed to send swap chain"),
                ServoSrcMsg::Resize(size) => self.resize(size),
                ServoSrcMsg::Heartbeat => self.servo.handle_events(vec![]),
                ServoSrcMsg::Stop => break,
            }
        }
        self.servo.handle_events(vec![WindowEvent::Quit]);
    }

    fn new_browser(&mut self, url: ServoUrl) {
        let id = TopLevelBrowsingContextId::new();
        self.servo
            .handle_events(vec![WindowEvent::NewBrowser(url, id)]);
    }

    fn resize(&mut self, size: Size2D<i32, DevicePixel>) {
        GFX.with(|gfx| {
            let mut gfx = gfx.borrow_mut();
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
            gfx.gl.bind_framebuffer(gl::FRAMEBUFFER, fbo);
            debug_assert_eq!(
                (
                    gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                    gfx.gl.get_error()
                ),
                (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
            );
        });
        self.servo.handle_events(vec![WindowEvent::Resize]);
    }
}

impl Drop for ServoThread {
    fn drop(&mut self) {
        GFX.with(|gfx| {
            let mut gfx = gfx.borrow_mut();
            let gfx = &mut *gfx;
            self.swap_chain
                .destroy(&mut gfx.device, &mut gfx.context)
                .expect("Failed to destroy swap chain")
        })
    }
}

struct ServoSrcEmbedder;

impl EmbedderMethods for ServoSrcEmbedder {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoSrcEmbedder)
    }
}

impl EventLoopWaker for ServoSrcEmbedder {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoSrcEmbedder)
    }

    fn wake(&self) {}
}

struct ServoSrcWindow {
    swap_chain: SwapChain,
    gl: Rc<dyn gleam::gl::Gl>,
}

impl ServoSrcWindow {
    fn new() -> Self {
        GFX.with(|gfx| {
            let mut gfx = gfx.borrow_mut();
            let gfx = &mut *gfx;
            let access = SurfaceAccess::GPUCPU;
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
            let swap_chain = SwapChain::create_attached(&mut gfx.device, &mut gfx.context, access)
                .expect("Failed to create swap chain");
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
            let gl = unsafe {
                gleam::gl::GlFns::load_with(|s| gfx.device.get_proc_address(&gfx.context, s))
            };
            Self { swap_chain, gl }
        })
    }
}

impl WindowMethods for ServoSrcWindow {
    fn present(&self) {
        GFX.with(|gfx| {
            debug!("EMBEDDER present");
            let mut gfx = gfx.borrow_mut();
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
            let _ = self
                .swap_chain
                .swap_buffers(&mut gfx.device, &mut gfx.context);
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
        })
    }

    fn make_gl_context_current(&self) {
        GFX.with(|gfx| {
            debug!("EMBEDDER make_context_current");
            let mut gfx = gfx.borrow_mut();
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
        })
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

impl ObjectSubclass for ServoSrc {
    const NAME: &'static str = "ServoSrc";
    // gstreamer-gl doesn't have support for GLBaseSrc yet
    // https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/issues/219
    type ParentType = BaseSrc;
    type Instance = ElementInstanceStruct<Self>;
    type Class = ClassStruct<Self>;

    fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(1);
        thread::spawn(move || ServoThread::new(receiver).run());
        let (acks, ackr) = crossbeam_channel::bounded(1);
        let _ = sender.send(ServoSrcMsg::GetSwapChain(acks));
        let swap_chain = ackr.recv().expect("Failed to get swap chain");
        let info = Mutex::new(None);
        let url = Mutex::new(None);
        Self {
            sender,
            swap_chain,
            info,
            url,
        }
    }

    fn class_init(klass: &mut ClassStruct<Self>) {
        klass.set_metadata(
            "Servo as a gstreamer src",
            "Filter/Effect/Converter/Video",
            "The Servo web browser",
            env!("CARGO_PKG_AUTHORS"),
        );

        let src_caps = Caps::new_simple(
            "video/x-raw",
            &[
                ("format", &VideoFormat::Bgrx.to_string()),
                ("width", &IntRange::<i32>::new(1, std::i32::MAX)),
                ("height", &IntRange::<i32>::new(1, std::i32::MAX)),
                (
                    "framerate",
                    &FractionRange::new(
                        Fraction::new(1, std::i32::MAX),
                        Fraction::new(std::i32::MAX, 1),
                    ),
                ),
            ],
        );
        let src_pad_template =
            PadTemplate::new("src", PadDirection::Src, PadPresence::Always, &src_caps).unwrap();
        klass.add_pad_template(src_pad_template);
        klass.install_properties(&PROPERTIES);
    }

    glib_object_subclass!();
}

impl ObjectImpl for ServoSrc {
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

impl ElementImpl for ServoSrc {}

impl BaseSrcImpl for ServoSrc {
    fn set_caps(&self, _src: &BaseSrc, outcaps: &Caps) -> Result<(), LoggableError> {
        let info = VideoInfo::from_caps(outcaps)
            .ok_or_else(|| gst_loggable_error!(CATEGORY, "Failed to get video info"))?;
        *self.info.lock().unwrap() = Some(info);
        Ok(())
    }

    fn get_size(&self, _src: &BaseSrc) -> Option<u64> {
        u64::try_from(self.info.lock().ok()?.as_ref()?.size()).ok()
    }

    fn start(&self, _src: &BaseSrc) -> Result<(), ErrorMessage> {
        info!("Starting");
        let guard = self
            .url
            .lock()
            .map_err(|_| gst_error_msg!(ResourceError::Settings, ["Failed to lock mutex"]))?;
        let url = guard.as_ref().map(|s| &**s).unwrap_or(DEFAULT_URL);
        let url = ServoUrl::parse(url)
            .map_err(|_| gst_error_msg!(ResourceError::Settings, ["Failed to parse url"]))?;
        let _ = self.sender.send(ServoSrcMsg::Start(url));
        Ok(())
    }

    fn stop(&self, _src: &BaseSrc) -> Result<(), ErrorMessage> {
        info!("Stopping");
        let _ = self.sender.send(ServoSrcMsg::Stop);
        Ok(())
    }

    fn fill(
        &self,
        src: &BaseSrc,
        _offset: u64,
        _length: u32,
        buffer: &mut BufferRef,
    ) -> Result<FlowSuccess, FlowError> {
        let guard = self.info.lock().map_err(|_| {
            gst_element_error!(src, CoreError::Negotiation, ["Lock poisoned"]);
            FlowError::NotNegotiated
        })?;
        let info = guard.as_ref().ok_or_else(|| {
            gst_element_error!(src, CoreError::Negotiation, ["Caps not set yet"]);
            FlowError::NotNegotiated
        })?;
        let mut frame = VideoFrameRef::from_buffer_ref_writable(buffer, info).ok_or_else(|| {
            gst_element_error!(
                src,
                CoreError::Failed,
                ["Failed to map output buffer writable"]
            );
            FlowError::Error
        })?;
        let height = frame.height() as i32;
        let width = frame.width() as i32;
        let size = Size2D::new(width, height);
        let format = frame.format();
        debug!(
            "Filling servosrc buffer {}x{} {:?} {:?}",
            width, height, format, frame,
        );
        let data = frame.plane_data_mut(0).unwrap();

        GFX.with(|gfx| {
            let mut gfx = gfx.borrow_mut();
            let gfx = &mut *gfx;
            if let Some(surface) = self.swap_chain.take_surface() {
                let surface_size = Size2D::from_untyped(gfx.device.surface_info(&surface).size);
                if size != surface_size {
                    // If we're being asked to fill frames that are a different size than servo is providing,
                    // ask it to change size.
                    let _ = self.sender.send(ServoSrcMsg::Resize(size));
                }

                gfx.device.make_context_current(&gfx.context).unwrap();
                debug_assert_eq!(
                    (
                        gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                        gfx.gl.get_error()
                    ),
                    (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
                );

                gfx.gl.viewport(0, 0, width, height);
                debug_assert_eq!(
                    (
                        gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                        gfx.gl.get_error()
                    ),
                    (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
                );

                let surface_texture = gfx
                    .device
                    .create_surface_texture(&mut gfx.context, surface)
                    .unwrap();
                let texture_id = surface_texture.gl_texture();

                gfx.gl.bind_framebuffer(gl::FRAMEBUFFER, gfx.fbo);
                gfx.gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gfx.device.surface_gl_texture_target(),
                    texture_id,
                    0,
                );
                debug_assert_eq!(
                    (
                        gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                        gfx.gl.get_error()
                    ),
                    (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
                );

                // TODO: use GL memory to avoid readback
                gfx.gl.read_pixels_into_buffer(
                    0,
                    0,
                    width,
                    height,
                    gl::BGRA,
                    gl::UNSIGNED_BYTE,
                    data,
                );
                debug_assert_eq!(
                    (
                        gfx.gl.check_framebuffer_status(gl::FRAMEBUFFER),
                        gfx.gl.get_error()
                    ),
                    (gl::FRAMEBUFFER_COMPLETE, gl::NO_ERROR)
                );

                gfx.device.make_no_context_current().unwrap();

                let surface = gfx
                    .device
                    .destroy_surface_texture(&mut gfx.context, surface_texture)
                    .unwrap();
                self.swap_chain.recycle_surface(surface);
            }
        });
        let _ = self.sender.send(ServoSrcMsg::Heartbeat);
        Ok(FlowSuccess::Ok)
    }
}
