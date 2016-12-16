/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::CanvasMsg;
use core::ops::Deref;
use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceBinding::PerformanceMethods;
use dom::bindings::codegen::Bindings::VRDisplayBinding;
use dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use dom::bindings::codegen::Bindings::VRDisplayBinding::VREye;
use dom::bindings::codegen::Bindings::VRLayerBinding::VRLayer;
use dom::bindings::codegen::Bindings::WindowBinding::FrameRequestCallback;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, MutJS, Root};
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::vrdisplaycapabilities::VRDisplayCapabilities;
use dom::vrdisplayevent::VRDisplayEvent;
use dom::vreyeparameters::VREyeParameters;
use dom::vrframedata::VRFrameData;
use dom::vrpose::VRPose;
use dom::vrstageparameters::VRStageParameters;
use dom::webglrenderingcontext::WebGLRenderingContext;
use ipc_channel::ipc;
use ipc_channel::ipc::{IpcSender, IpcReceiver};
use js::jsapi::JSContext;
use script_runtime::CommonScriptMsg;
use script_runtime::ScriptThreadEventCategory::WebVREvent;
use script_thread::Runnable;
use std::cell::Cell;
use std::mem;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use vr_traits::WebVRMsg;
use vr_traits::webvr;
use webrender_traits::VRCompositorCommand;

#[dom_struct]
pub struct VRDisplay {
    eventtarget: EventTarget,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    display: DOMRefCell<WebVRDisplayData>,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    presenting: Cell<bool>,
    left_eye_params: MutJS<VREyeParameters>,
    right_eye_params: MutJS<VREyeParameters>,
    capabilities: MutJS<VRDisplayCapabilities>,
    stage_params: MutNullableJS<VRStageParameters>,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    frame_data: DOMRefCell<WebVRFrameData>,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    layer: DOMRefCell<WebVRLayer>,
    layer_ctx: MutNullableJS<WebGLRenderingContext>,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    next_raf_id: Cell<u32>,
    /// List of request animation frame callbacks
    #[ignore_heap_size_of = "closures are hard"]
    raf_callback_list: DOMRefCell<Vec<(u32, Option<Rc<FrameRequestCallback>>)>>,
    // Compositor VRFrameData synchonization
    frame_data_status: Cell<VRFrameDataStatus>,
    #[ignore_heap_size_of = "channels are hard"]
    frame_data_receiver: DOMRefCell<Option<IpcReceiver<Result<Vec<u8>, ()>>>>
}

// Wrappers to include WebVR structs in a DOM struct
#[derive(Clone)]
pub struct WebVRDisplayData(webvr::VRDisplayData);
unsafe_no_jsmanaged_fields!(WebVRDisplayData);

#[derive(Clone, Default)]
pub struct WebVRFrameData(webvr::VRFrameData);
unsafe_no_jsmanaged_fields!(WebVRFrameData);

#[derive(Clone, Default)]
pub struct WebVRLayer(webvr::VRLayer);
unsafe_no_jsmanaged_fields!(WebVRLayer);

#[derive(Clone, Copy, PartialEq, Eq, HeapSizeOf)]
enum VRFrameDataStatus {
    Waiting,
    Synced,
    Exit
}

unsafe_no_jsmanaged_fields!(VRFrameDataStatus);

impl VRDisplay {
    fn new_inherited(global: &GlobalScope, display:&webvr::VRDisplayData) -> VRDisplay {
        let stage = match display.stage_parameters {
            Some(ref params) => Some(VRStageParameters::new(&params, &global)),
            None => None
        };

        VRDisplay {
            eventtarget: EventTarget::new_inherited(),
            display: DOMRefCell::new(WebVRDisplayData(display.clone())),
            depth_near: Cell::new(0.01),
            depth_far: Cell::new(10000.0),
            presenting: Cell::new(false),
            left_eye_params: MutJS::new(&*VREyeParameters::new(&display.left_eye_parameters, &global)),
            right_eye_params: MutJS::new(&*VREyeParameters::new(&display.right_eye_parameters, &global)),
            capabilities: MutJS::new(&*VRDisplayCapabilities::new(&display.capabilities, &global)),
            stage_params: MutNullableJS::new(stage.as_ref().map(|v| v.deref())),
            frame_data: DOMRefCell::new(Default::default()),
            layer: DOMRefCell::new(Default::default()),
            layer_ctx: MutNullableJS::default(),
            next_raf_id: Cell::new(1),
            raf_callback_list: DOMRefCell::new(vec![]),
            frame_data_status: Cell::new(VRFrameDataStatus::Waiting),
            frame_data_receiver: DOMRefCell::new(None)
        }
    }

    pub fn new(global: &GlobalScope, display:&webvr::VRDisplayData) -> Root<VRDisplay> {
        reflect_dom_object(box VRDisplay::new_inherited(&global, &display),
                           global,
                           VRDisplayBinding::Wrap)
    }
}

impl Drop for VRDisplay {
    fn drop(&mut self) {
        if self.presenting.get() {
            self.force_stop_present();
        }
    }
}

impl VRDisplayMethods for VRDisplay {
    // https://w3c.github.io/webvr/#dom-vrdisplay-isconnected
    fn IsConnected(&self) -> bool {
        self.display.borrow().0.connected
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-ispresenting
    fn IsPresenting(&self) -> bool {
        self.presenting.get()
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-capabilities
    fn Capabilities(&self) -> Root<VRDisplayCapabilities> {
        Root::from_ref(&*self.capabilities.get())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-stageparameters
    fn GetStageParameters(&self) -> Option<Root<VRStageParameters>> {
        self.stage_params.get().map(|s| Root::from_ref(&*s))
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-geteyeparameters
    fn GetEyeParameters(&self, eye: VREye) -> Root<VREyeParameters> {
        match eye {
            VREye::Left => Root::from_ref(&*self.left_eye_params.get()),
            VREye::Right => Root::from_ref(&*self.right_eye_params.get())
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-displayid
    fn DisplayId(&self) -> u32 {
        self.display.borrow().0.display_id as u32
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-displayname
    fn DisplayName(&self) -> DOMString {
        DOMString::from(self.display.borrow().0.display_name.clone())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-getframedata-framedata-framedata
    fn GetFrameData(&self, frameData: &VRFrameData) -> bool {
        // If presenting we use a synced data with compositor for the whole frame
        if self.presenting.get() {
            if self.frame_data_status.get() == VRFrameDataStatus::Waiting {
                self.sync_frame_data();
            }
            frameData.update(& self.frame_data.borrow().0);
            return true;
        }

        // If not presenting we fetch inmediante VRFrameData
        if let Some(wevbr_sender) = self.webvr_thread() {
            let (sender, receiver) = ipc::channel().unwrap();
            wevbr_sender.send(WebVRMsg::GetFrameData(self.global().pipeline_id(),
                                                     self.get_display_id(),
                                                     self.depth_near.get(),
                                                     self.depth_far.get(),
                                                     sender)).unwrap();
            return match receiver.recv().unwrap() {
                Ok(data) => {
                    frameData.update(&data);
                    true
                },
                Err(e) => {
                    error!("WebVR::GetFrameData: {:?}", e);
                    false
                }
            };
        }

        false
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-getpose
    fn GetPose(&self) -> Root<VRPose> {
        VRPose::new(&self.global(), &self.frame_data.borrow().0.pose)
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-resetpose
    fn ResetPose(&self) -> () {
        if let Some(wevbr_sender) = self.webvr_thread() {
            let (sender, receiver) = ipc::channel().unwrap();
            wevbr_sender.send(WebVRMsg::ResetPose(self.global().pipeline_id(),
                                                  self.get_display_id(),
                                                  sender)).unwrap();
            if let Ok(data) = receiver.recv().unwrap() {
                // Some VRDisplay data might change after calling ResetPose()
                self.display.borrow_mut().0 = data;
            }
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthnear
    fn DepthNear(&self) -> Finite<f64> {
        Finite::wrap(self.depth_near.get())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthnear
    fn SetDepthNear(&self, value: Finite<f64>) -> () {
        self.depth_near.set(*value.deref());
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthfar
    fn DepthFar(&self) -> Finite<f64> {
        Finite::wrap(self.depth_far.get())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthfar
    fn SetDepthFar(&self, value: Finite<f64>) -> () {
        self.depth_far.set(*value.deref());
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<FrameRequestCallback>) -> u32 {
        if self.presenting.get() {
            let raf_id = self.next_raf_id.get();
            self.next_raf_id.set(raf_id + 1);
            self.raf_callback_list.borrow_mut().push((raf_id, Some(callback)));
            raf_id
        } else {
            // WebVR spec: When a VRDisplay is not presenting it should
            // fallback to window.requestAnimationFrame.
            self.global().as_window().RequestAnimationFrame(callback)
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-cancelanimationframe
    fn CancelAnimationFrame(&self, handle: u32) -> () {
        if self.presenting.get() {
            let mut list = self.raf_callback_list.borrow_mut();
            if let Some(mut pair) = list.iter_mut().find(|pair| pair.0 == handle) {
                pair.1 = None;
            }
        } else {
            // WebVR spec: When a VRDisplay is not presenting it should
            // fallback to window.cancelAnimationFrame.
            self.global().as_window().CancelAnimationFrame(handle);
        }
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/webvr/#dom-vrdisplay-requestpresent
    fn RequestPresent(&self, layers: Vec<VRLayer>) -> Rc<Promise> {
        let promise = Promise::new(&self.global());
        // TODO: WebVR spec: this method must be called in response to a user gesture

        // WebVR spec: If canPresent is false the promise MUST be rejected
        if !self.display.borrow().0.capabilities.can_present {
            let msg = "VRDisplay canPresent is false".to_string();
            promise.reject_native(promise.global().get_cx(), &msg);
            return promise;
        }

        // Current WebVRSpec only allows 1 VRLayer if the VRDevice can present.
        // Future revisions of this spec may allow multiple layers to enable more complex rendering effects
        // such as compositing WebGL and DOM elements together.
        // That functionality is not allowed by this revision of the spec.
        if layers.len() != 1 {
            let msg = "The number of layers must be 1".to_string();
            promise.reject_native(promise.global().get_cx(), &msg);
            return promise;
        }

        // Parse and validate received VRLayer
        let layer = validate_layer(self.global().get_cx(), &layers[0]);

        if let Err(msg) = layer {
            let msg = msg.to_string();
            promise.reject_native(promise.global().get_cx(), &msg);
            return promise;
        }

        let (layer_bounds, layer_ctx) = layer.unwrap();

        // WebVR spec: Repeat calls while already presenting will update the VRLayers being displayed.
        if self.presenting.get() {
            *self.layer.borrow_mut() = layer_bounds;
            self.layer_ctx.set(Some(&layer_ctx));
            promise.resolve_native(promise.global().get_cx(), &());
            return promise;
        }

        // Request Present
        if let Some(wevbr_sender) = self.webvr_thread() {
            let (sender, receiver) = ipc::channel().unwrap();
            wevbr_sender.send(WebVRMsg::RequestPresent(self.global().pipeline_id(),
                                                       self.display.borrow().0.display_id,
                                                       sender))
                                                       .unwrap();
            match receiver.recv().unwrap() {
                Ok(()) => {
                    *self.layer.borrow_mut() = layer_bounds;
                    self.layer_ctx.set(Some(&layer_ctx));
                    self.init_present();
                    promise.resolve_native(promise.global().get_cx(), &());
                },
                Err(e) => {
                    promise.reject_native(promise.global().get_cx(), &e);
                }
            }
        } else {
            let msg = "Not available".to_string();
            promise.reject_native(promise.global().get_cx(), &msg);
        }

        promise
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/webvr/#dom-vrdisplay-exitpresent
    fn ExitPresent(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.global());

        // WebVR spec: If the VRDisplay is not presenting the promise MUST be rejected.
        if !self.presenting.get() {
            let msg = "VRDisplay is not presenting".to_string();
            promise.reject_native(promise.global().get_cx(), &msg);
            return promise;
        }

        // Exit present
        if let Some(wevbr_sender) = self.webvr_thread() {
            let (sender, receiver) = ipc::channel().unwrap();
            wevbr_sender.send(WebVRMsg::ExitPresent(self.global().pipeline_id(),
                                                    self.display.borrow().0.display_id,
                                                    Some(sender)))
                                                    .unwrap();
            match receiver.recv().unwrap() {
                Ok(()) => {
                    self.stop_present();
                    promise.resolve_native(promise.global().get_cx(), &());
                },
                Err(e) => {
                    promise.reject_native(promise.global().get_cx(), &e);
                }
            }
        } else {
            let msg = "Not available".to_string();
            promise.reject_native(promise.global().get_cx(), &msg);
        }

        promise
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-submitframe
    fn SubmitFrame(&self) -> () {
        if !self.presenting.get() {
            warn!("VRDisplay not presenting");
            return;
        }

        let api_sender = self.layer_ctx.get().unwrap().ipc_renderer();
        let display_id = self.display.borrow().0.display_id;
        let layer = self.layer.borrow();
        let msg = VRCompositorCommand::SubmitFrame(display_id, layer.0.left_bounds, layer.0.right_bounds);
        api_sender.send(CanvasMsg::WebVR(msg)).unwrap();
    }
}

impl VRDisplay {
    fn webvr_thread(&self) -> Option<IpcSender<WebVRMsg>> {
        self.global().as_window().webvr_thread()
    }

    pub fn get_display_id(&self) -> u64 {
        self.display.borrow().0.display_id
    }

    pub fn update_display(&self, display: &webvr::VRDisplayData) {
        self.display.borrow_mut().0 = display.clone();
        if let Some(ref stage) = display.stage_parameters {
            if self.stage_params.get().is_none() {
                let params = Some(VRStageParameters::new(&stage, &self.global()));
                self.stage_params.set(params.as_ref().map(|v| v.deref()));
            } else {
                self.stage_params.get().unwrap().update(&stage);
            }
        } else {
            self.stage_params.set(None);
        }

    }

    pub fn handle_webvr_event(&self, event: &webvr::VRDisplayEvent) {
        match *event {
            webvr::VRDisplayEvent::Connect(ref display) => {
                self.update_display(&display);
            },
            webvr::VRDisplayEvent::Disconnect(_id) => {
                self.display.borrow_mut().0.connected = false;
            },
            webvr::VRDisplayEvent::Activate(ref display, _) |
            webvr::VRDisplayEvent::Deactivate(ref display, _) |
            webvr::VRDisplayEvent::Blur(ref display) |
            webvr::VRDisplayEvent::Focus(ref display) => {
                self.update_display(&display);
                self.notify_event(&event);
            },
            webvr::VRDisplayEvent::PresentChange(ref display, presenting) => {
                self.update_display(&display);
                self.presenting.set(presenting);
                self.notify_event(&event);
            },
            webvr::VRDisplayEvent::Change(ref display) => {
                // Change event doesn't exist in WebVR spec.
                // So we update display data but don't notify JS.
                self.update_display(&display);
            }
        };
    }

    fn notify_event(&self, event: &webvr::VRDisplayEvent) {
        let root = Root::from_ref(&*self);
        let event = VRDisplayEvent::new_from_webvr(&self.global(), &root, &event);
        event.upcast::<Event>().fire(self.upcast());
    }

    fn init_present(&self) {
        self.presenting.set(true);
        let (sync_sender, sync_receiver) = ipc::channel().unwrap();
        *self.frame_data_receiver.borrow_mut() = Some(sync_receiver);

        let display_id = self.display.borrow().0.display_id;
        let api_sender = self.layer_ctx.get().unwrap().ipc_renderer();
        let js_sender = self.global().script_chan();
        let address = Trusted::new(&*self);
        let near_init = self.depth_near.get();
        let far_init = self.depth_far.get();

        thread::Builder::new().name("WebVR_RAF".into()).spawn(move || {
            let (raf_sender, raf_receiver) = mpsc::channel();
            let mut near = near_init;
            let mut far = far_init;

            // Initialize compositor
            api_sender.send(CanvasMsg::WebVR(VRCompositorCommand::Create(display_id))).unwrap();
            loop {
                // Run RAF callbacks on JavaScript thread
                let msg = box NotifyDisplayRAF {
                    address: address.clone(),
                    sender: raf_sender.clone()
                };
                js_sender.send(CommonScriptMsg::RunnableMsg(WebVREvent, msg)).unwrap();

                // Run Sync Poses in parallell on Render thread
                let msg = VRCompositorCommand::SyncPoses(display_id, near, far, sync_sender.clone());
                api_sender.send(CanvasMsg::WebVR(msg)).unwrap();

                // Wait until both SyncPoses & RAF ends
                if let Ok(depth) = raf_receiver.recv().unwrap() {
                    near = depth.0;
                    far = depth.1;
                } else {
                    // Stop thread
                    // ExitPresent called or some error happened
                    return;
                }
            }
        }).expect("Thread spawning failed");
    }

    fn stop_present(&self) {
        self.presenting.set(false);
        *self.frame_data_receiver.borrow_mut() = None;

        let api_sender = self.layer_ctx.get().unwrap().ipc_renderer();
        let display_id = self.display.borrow().0.display_id;
        let msg = VRCompositorCommand::Release(display_id);
        api_sender.send(CanvasMsg::WebVR(msg)).unwrap();
    }

    // Only called when the JSContext is destroyed while presenting
    // In this case we don't want to wait for WebVR Thread response
    fn force_stop_present(&self) {
        if let Some(wevbr_sender) = self.webvr_thread() {
            wevbr_sender.send(WebVRMsg::ExitPresent(self.global().pipeline_id(),
                                                    self.display.borrow().0.display_id,
                                                    None))
                                                    .unwrap();
        }
        self.stop_present();
    }

    fn sync_frame_data(&self) {
        let status = if let Some(receiver) = self.frame_data_receiver.borrow().as_ref() {
            match receiver.recv().unwrap() {
                Ok(bytes) => {
                    self.frame_data.borrow_mut().0 = webvr::VRFrameData::from_bytes(&bytes[..]);
                    VRFrameDataStatus::Synced
                },
                Err(()) => {
                    VRFrameDataStatus::Exit
                }
            }
        } else {
            VRFrameDataStatus::Exit
        };

        self.frame_data_status.set(status);
    }

    fn handle_raf(&self, end_sender: &mpsc::Sender<Result<(f64, f64), ()>>) {
        self.frame_data_status.set(VRFrameDataStatus::Waiting);

        let mut callbacks = mem::replace(&mut *self.raf_callback_list.borrow_mut(), vec![]);
        let now = self.global().as_window().Performance().Now();

        for (_, callback) in callbacks.drain(..) {
            if let Some(callback) = callback {
                let _ = callback.Call__(Finite::wrap(*now), ExceptionHandling::Report);
            }
        }

        if self.frame_data_status.get() == VRFrameDataStatus::Waiting {
            // User didn't call getFrameData while presenting
            // Show a warning as the WebVR Spec recommends
            warn!("WebVR: You should call GetFrameData while presenting");
            self.sync_frame_data();
        }

        match self.frame_data_status.get() {
            VRFrameDataStatus::Synced => {
                end_sender.send(Ok((self.depth_near.get(), self.depth_far.get()))).unwrap();
            },
            _ => {
                end_sender.send(Err(())).unwrap();
            }
        }
    }
}

struct NotifyDisplayRAF {
    address: Trusted<VRDisplay>,
    sender: mpsc::Sender<Result<(f64, f64), ()>>
}

impl Runnable for NotifyDisplayRAF {
    fn name(&self) -> &'static str { "NotifyDisplayRAF" }

    fn handler(self: Box<Self>) {
        let display = self.address.root();
        display.handle_raf(&self.sender);
    }
}


// WebVR Spect: If the number of values in the leftBounds/rightBounds arrays
// is not 0 or 4 for any of the passed layers the promise is rejected
fn parse_bounds(src: &Option<Vec<Finite<f32>>>, dst: &mut [f32; 4]) -> Result<(), &'static str> {
    match *src {
        Some(ref values) => {
            if values.len() == 0 {
                return Ok(())
            }
            if values.len() > 4 {
                return Err("The number of values in the leftBounds/rightBounds arrays must be 0 or 4")
            }
            for i in 0..4 {
                dst[i] = *values[i].deref();
            }
            Ok(())
        },
        None => Ok(())
    }
}

fn validate_layer(cx: *mut JSContext,
                  layer: &VRLayer)
                  -> Result<(WebVRLayer, Root<WebGLRenderingContext>), &'static str> {
    let ctx = layer.source.as_ref().map(|ref s| s.get_or_init_webgl_context(cx, None)).unwrap_or(None);
    if let Some(ctx) = ctx {
        let mut data = webvr::VRLayer::default();
        try!(parse_bounds(&layer.leftBounds, &mut data.left_bounds));
        try!(parse_bounds(&layer.rightBounds, &mut data.right_bounds));
        Ok((WebVRLayer(data), ctx))
    } else {
        Err("VRLayer source must be a WebGL Context")
    }
}
