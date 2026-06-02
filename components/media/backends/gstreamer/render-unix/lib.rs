/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `RenderUnix` is a `Render` implementation for Unix-based
//! platforms. It implements an OpenGL mechanism shared by Linux and
//! many of the BSD flavors.
//!
//! Internally it uses GStreamer's *glsinkbin* element as *videosink*
//! wrapping the *appsink* from the Player. And the shared frames are
//! mapped as texture IDs.

use std::sync::{Arc, Mutex};

use gstreamer_gl::prelude::*;
use sm_gst_render::Render;
use sm_player::PlayerError;
use sm_player::context::{GlApi, GlContext, NativeDisplay, PlayerGLContext};
use sm_player::video::{Buffer, VideoFrame, VideoFrameData};

struct GStreamerBuffer {
    is_external_oes: bool,
    frame: gstreamer_gl::GLVideoFrame<gstreamer_gl::gl_video_frame::Readable>,
}

impl Buffer for GStreamerBuffer {
    fn to_vec(&self) -> Option<VideoFrameData> {
        // packed formats are guaranteed to be in a single plane
        if self.frame.format() == gstreamer_video::VideoFormat::Rgba {
            let tex_id = self.frame.texture_id(0).ok()?;
            Some(if self.is_external_oes {
                VideoFrameData::OESTexture(tex_id)
            } else {
                VideoFrameData::Texture(tex_id)
            })
        } else {
            None
        }
    }
}

pub struct RenderUnix {
    display: gstreamer_gl::GLDisplay,
    app_context: gstreamer_gl::GLContext,
    gst_context: Arc<Mutex<Option<gstreamer_gl::GLContext>>>,
    gl_upload: Arc<Mutex<Option<gstreamer::Element>>>,
}

impl RenderUnix {
    /// Tries to create a new intance of the `RenderUnix`
    ///
    /// # Arguments
    ///
    /// * `context` - is the PlayerContext trait object from application.
    pub fn new(app_gl_context: Box<dyn PlayerGLContext>) -> Option<RenderUnix> {
        // Check that we actually have the elements that we
        // need to make this work.
        gstreamer::ElementFactory::find("glsinkbin")?;

        let display_native = app_gl_context.get_native_display();
        let gl_context = app_gl_context.get_gl_context();
        let gl_api = match app_gl_context.get_gl_api() {
            GlApi::OpenGL => gstreamer_gl::GLAPI::OPENGL,
            GlApi::OpenGL3 => gstreamer_gl::GLAPI::OPENGL3,
            GlApi::Gles1 => gstreamer_gl::GLAPI::GLES1,
            GlApi::Gles2 => gstreamer_gl::GLAPI::GLES2,
            GlApi::None => return None,
        };

        let (wrapped_context, display) = match gl_context {
            GlContext::Egl(context) => {
                let display = match display_native {
                    #[cfg(feature = "gl-egl")]
                    NativeDisplay::Egl(display_native) => {
                        unsafe { gstreamer_gl_egl::GLDisplayEGL::with_egl_display(display_native) }
                            .map(|display| display.upcast())
                            .ok()
                    },
                    #[cfg(feature = "gl-wayland")]
                    NativeDisplay::Wayland(display_native) => unsafe {
                        gstreamer_gl_wayland::GLDisplayWayland::with_display(display_native)
                    }
                    .map(|display| display.upcast())
                    .ok(),
                    _ => None,
                };

                RenderUnix::create_wrapped_context(
                    display,
                    context,
                    gstreamer_gl::GLPlatform::EGL,
                    gl_api,
                )
            },
            GlContext::Glx(context) => {
                let display = match display_native {
                    #[cfg(feature = "gl-x11")]
                    NativeDisplay::X11(display_native) => {
                        unsafe { gstreamer_gl_x11::GLDisplayX11::with_display(display_native) }
                            .map(|display| display.upcast())
                            .ok()
                    },
                    _ => None,
                };

                RenderUnix::create_wrapped_context(
                    display,
                    context,
                    gstreamer_gl::GLPlatform::GLX,
                    gl_api,
                )
            },
            GlContext::Unknown => (None, None),
        };

        match wrapped_context {
            Some(app_context) => {
                let cat = gstreamer::DebugCategory::get("servoplayer").unwrap();
                let _: Result<(), ()> = app_context
                    .activate(true)
                    .and_then(|_| {
                        app_context.fill_info().or_else(|err| {
                            gstreamer::warning!(
                                cat,
                                "Couldn't fill the wrapped app GL context: {}",
                                err.to_string()
                            );
                            Ok(())
                        })
                    })
                    .or_else(|_| {
                        gstreamer::warning!(cat, "Couldn't activate the wrapped app GL context");
                        Ok(())
                    });
                Some(RenderUnix {
                    display: display.unwrap(),
                    app_context,
                    gst_context: Arc::new(Mutex::new(None)),
                    gl_upload: Arc::new(Mutex::new(None)),
                })
            },
            _ => None,
        }
    }

    fn create_wrapped_context(
        display: Option<gstreamer_gl::GLDisplay>,
        handle: usize,
        platform: gstreamer_gl::GLPlatform,
        api: gstreamer_gl::GLAPI,
    ) -> (
        Option<gstreamer_gl::GLContext>,
        Option<gstreamer_gl::GLDisplay>,
    ) {
        match display {
            Some(display) => {
                let wrapped_context = unsafe {
                    gstreamer_gl::GLContext::new_wrapped(&display, handle, platform, api)
                };
                (wrapped_context, Some(display))
            },
            _ => (None, None),
        }
    }
}

impl Render for RenderUnix {
    fn is_gl(&self) -> bool {
        true
    }

    fn build_frame(&self, sample: gstreamer::Sample) -> Option<VideoFrame> {
        if self.gst_context.lock().unwrap().is_none() && self.gl_upload.lock().unwrap().is_some() {
            *self.gst_context.lock().unwrap() = self
                .gl_upload
                .lock()
                .unwrap()
                .as_ref()
                .map(|glupload| glupload.property::<gstreamer_gl::GLContext>("context"));
        }

        let buffer = sample.buffer_owned()?;
        let caps = sample.caps()?;

        let is_external_oes = caps
            .structure(0)
            .and_then(|s| {
                s.get::<&str>("texture-target").ok().and_then(|target| {
                    if target == "external-oes" {
                        Some(s)
                    } else {
                        None
                    }
                })
            })
            .is_some();

        let info = gstreamer_video::VideoInfo::from_caps(caps).ok()?;
        let frame = gstreamer_gl::GLVideoFrame::from_buffer_readable(buffer, &info).ok()?;
        VideoFrame::new(
            info.width() as i32,
            info.height() as i32,
            Arc::new(GStreamerBuffer {
                is_external_oes,
                frame,
            }),
        )
    }

    fn build_video_sink(
        &self,
        appsink: &gstreamer::Element,
        pipeline: &gstreamer::Element,
    ) -> Result<(), PlayerError> {
        if self.gl_upload.lock().unwrap().is_some() {
            return Err(PlayerError::Backend(
                "render unix already setup the video sink".to_owned(),
            ));
        }

        let vsinkbin = gstreamer::ElementFactory::make("glsinkbin")
            .name("servo-media-vsink")
            .build()
            .map_err(|error| {
                PlayerError::Backend(format!("glupload creation failed: {error:?}"))
            })?;

        let caps = gstreamer::Caps::builder("video/x-raw")
            .features([gstreamer_gl::CAPS_FEATURE_MEMORY_GL_MEMORY])
            .field("format", gstreamer_video::VideoFormat::Rgba.to_str())
            .field(
                "texture-target",
                gstreamer::List::new(["2D", "external-oes"]),
            )
            .build();
        appsink.set_property("caps", caps);

        vsinkbin.set_property("sink", appsink);

        pipeline.set_property("video-sink", &vsinkbin);

        let bus = pipeline.bus().expect("pipeline with no bus");
        let display_ = self.display.clone();
        let context_ = self.app_context.clone();
        bus.set_sync_handler(move |_, msg| {
            if let gstreamer::MessageView::NeedContext(ctxt) = msg.view() {
                if let Some(el) = msg
                    .src()
                    .map(|s| s.clone().downcast::<gstreamer::Element>().unwrap())
                {
                    let context_type = ctxt.context_type();
                    if context_type == *gstreamer_gl::GL_DISPLAY_CONTEXT_TYPE {
                        let ctxt = gstreamer::Context::new(context_type, true);
                        ctxt.set_gl_display(&display_);
                        el.set_context(&ctxt);
                    } else if context_type == "gst.gl.app_context" {
                        let mut ctxt = gstreamer::Context::new(context_type, true);
                        {
                            let s = ctxt.get_mut().unwrap().structure_mut();
                            s.set_value("context", context_.to_send_value());
                        }
                        el.set_context(&ctxt);
                    }
                }
            }

            gstreamer::BusSyncReply::Pass
        });

        let mut iter = vsinkbin
            .dynamic_cast::<gstreamer::Bin>()
            .unwrap()
            .iterate_elements();
        *self.gl_upload.lock().unwrap() = loop {
            match iter.next() {
                Ok(Some(element)) => {
                    if "glupload" == element.factory().unwrap().name() {
                        break Some(element);
                    }
                },
                Err(gstreamer::IteratorError::Resync) => iter.resync(),
                _ => break None,
            }
        };

        Ok(())
    }
}
