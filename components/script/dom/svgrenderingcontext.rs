use canvas_traits::webgl::{webgl_channel, WebGLMsgSender, WebGLContextShareMode, GLContextAttributes, WebGLMsg, WebGLVersion};
use std::cell::Cell;
use crate::dom::window::Window;
use euclid::Size2D;
use crate::dom::bindings::root::{DomRoot, Dom};
use crate::dom::event::{EventBubbles, EventCancelable, Event};
use crate::dom::webglcontextevent::WebGLContextEvent;
use crate::dom::svgsvgelement::SVGSVGElement;
use dom_struct::dom_struct;
use crate::dom::bindings::str::DOMString;

#[dom_struct]
pub struct SVGRenderingContext{
    #[ignore_malloc_size_of = "Channels are hard"]
    webgl_sender: WebGLMsgSender,
    share_mode: WebGLContextShareMode,
    #[ignore_malloc_size_of = "Channels are hard"]
    webrender_image: Cell<Option<webrender_api::ImageKey>>,
    svg: Dom<SVGSVGElement>,
}

impl SVGRenderingContext{
    pub fn new_inherited(
        window: &Window,
        size: Size2D<u32>,
        svg: &SVGSVGElement,
    ) -> Result<SVGRenderingContext, String>{
        let webgl_chan = match window.webgl_chan() {
            Some(chan) => chan,
            None => panic!("Crash the system!"),
        };

        let (sender, receiver) = webgl_channel()
            .unwrap();
        let attrs = GLContextAttributes{
            depth: false,
            stencil: false,
            alpha: true,
            antialias: true,
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
        };
        webgl_chan
            .send(WebGLMsg::CreateContext(WebGLVersion::WebGL1, size, attrs, sender))
            .unwrap();
        let result = receiver.recv().unwrap();

        result.map(|ctx_data| {
            let webgl_sender = ctx_data.sender;
            let share_mode = ctx_data.share_mode;
            SVGRenderingContext{
                webgl_sender,
                share_mode,
                webrender_image: Cell::new(None),
                Dom::from_ref(svg),
            }
        })
    }
    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        size: Size2D<u32>,
        svg: &SVGSVGElement,
    ) -> DomRoot<Option<SVGRenderingContext>>{
        match SVGRenderingContext::new_inherited(window, size, svg) {
            Ok(ctx) => DomRoot::from_ref(Some(ctx)),
            Err(msg) => {
                error!("Couldn't create SVGRenderingContext:{}", msg);
                let event = WebGLContextEvent::new(
                    window,
                    atom!("webglcontextcreationerror"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::Cancelable,
                    DOMString::from(msg),
                );
                event.upcast::<Event>().fire(svg.upcast());
                None
            },
        }
    }
    pub fn extract_image_key(&self) -> webrender_api::ImageKey{
        match self.share_mode {
            WebGLContextShareMode::SharedTexture => {
                // WR using ExternalTexture requires a single update message.
                self.webrender_image.get().unwrap_or_else(|| {
                    let (sender, receiver) = webgl_channel().unwrap();
                    self.webgl_sender.send_update_wr_image(sender).unwrap();
                    let image_key = receiver.recv().unwrap();
                    self.webrender_image.set(Some(image_key));

                    image_key
                })
            },
            WebGLContextShareMode::Readback => {
                // WR using Readback requires to update WR image every frame
                // in order to send the new raw pixels.
                let (sender, receiver) = webgl_channel().unwrap();
                self.webgl_sender.send_update_wr_image(sender).unwrap();
                receiver.recv().unwrap()
            },
        }
    }
}