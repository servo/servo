/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VREye;
use crate::dom::bindings::codegen::Bindings::VRLayerBinding::VRLayer;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::FrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{DomRoot, MutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::node::document_from_node;
use crate::dom::promise::Promise;
use crate::dom::vrdisplaycapabilities::VRDisplayCapabilities;
use crate::dom::vrdisplayevent::VRDisplayEvent;
use crate::dom::vreyeparameters::VREyeParameters;
use crate::dom::vrframedata::VRFrameData;
use crate::dom::vrpose::VRPose;
use crate::dom::vrstageparameters::VRStageParameters;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::window::ReflowReason;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrsession::XRSession;
use crate::script_runtime::CommonScriptMsg;
use crate::script_runtime::ScriptThreadEventCategory::WebVREvent;
use crate::task_source::{TaskSource, TaskSourceName};
use canvas_traits::webgl::{webgl_channel, WebGLReceiver, WebVRCommand};
use crossbeam_channel::{unbounded, Sender};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use profile_traits::ipc;
use script_layout_interface::message::ReflowGoal;
use serde_bytes::ByteBuf;
use std::cell::Cell;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::thread;
use webvr_traits::{WebVRDisplayData, WebVRDisplayEvent, WebVRFrameData, WebVRLayer, WebVRMsg};

#[dom_struct]
pub struct VRDisplay {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    display: DomRefCell<WebVRDisplayData>,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    presenting: Cell<bool>,
    left_eye_params: MutDom<VREyeParameters>,
    right_eye_params: MutDom<VREyeParameters>,
    capabilities: MutDom<VRDisplayCapabilities>,
    stage_params: MutNullableDom<VRStageParameters>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    frame_data: DomRefCell<WebVRFrameData>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    layer: DomRefCell<WebVRLayer>,
    layer_ctx: MutNullableDom<WebGLRenderingContext>,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    next_raf_id: Cell<u32>,
    /// List of request animation frame callbacks
    #[ignore_malloc_size_of = "closures are hard"]
    raf_callback_list: DomRefCell<Vec<(u32, Option<Rc<FrameRequestCallback>>)>>,
    #[ignore_malloc_size_of = "closures are hard"]
    xr_raf_callback_list: DomRefCell<Vec<(u32, Option<Rc<XRFrameRequestCallback>>)>>,
    // Compositor VRFrameData synchonization
    frame_data_status: Cell<VRFrameDataStatus>,
    #[ignore_malloc_size_of = "closures are hard"]
    frame_data_receiver: DomRefCell<Option<WebGLReceiver<Result<ByteBuf, ()>>>>,
    running_display_raf: Cell<bool>,
    paused: Cell<bool>,
    stopped_on_pause: Cell<bool>,
    /// Whether or not this is XR mode, and the session
    xr_session: MutNullableDom<XRSession>,
}

unsafe_no_jsmanaged_fields!(WebVRDisplayData);
unsafe_no_jsmanaged_fields!(WebVRFrameData);
unsafe_no_jsmanaged_fields!(WebVRLayer);

#[derive(Clone, Copy, Eq, MallocSizeOf, PartialEq)]
enum VRFrameDataStatus {
    Waiting,
    Synced,
    Exit,
}

unsafe_no_jsmanaged_fields!(VRFrameDataStatus);

impl VRDisplay {
    fn new_inherited(global: &GlobalScope, display: WebVRDisplayData) -> VRDisplay {
        let stage = match display.stage_parameters {
            Some(ref params) => Some(VRStageParameters::new(params.clone(), &global)),
            None => None,
        };

        VRDisplay {
            eventtarget: EventTarget::new_inherited(),
            display: DomRefCell::new(display.clone()),
            depth_near: Cell::new(0.01),
            depth_far: Cell::new(10000.0),
            presenting: Cell::new(false),
            left_eye_params: MutDom::new(&*VREyeParameters::new(
                display.left_eye_parameters.clone(),
                &global,
            )),
            right_eye_params: MutDom::new(&*VREyeParameters::new(
                display.right_eye_parameters.clone(),
                &global,
            )),
            capabilities: MutDom::new(&*VRDisplayCapabilities::new(
                display.capabilities.clone(),
                &global,
            )),
            stage_params: MutNullableDom::new(stage.as_ref().map(|v| v.deref())),
            frame_data: DomRefCell::new(Default::default()),
            layer: DomRefCell::new(Default::default()),
            layer_ctx: MutNullableDom::default(),
            next_raf_id: Cell::new(1),
            raf_callback_list: DomRefCell::new(vec![]),
            xr_raf_callback_list: DomRefCell::new(vec![]),
            frame_data_status: Cell::new(VRFrameDataStatus::Waiting),
            frame_data_receiver: DomRefCell::new(None),
            running_display_raf: Cell::new(false),
            // Some VR implementations (e.g. Daydream) can be paused in some life cycle situations
            // such as showing and hiding the controller pairing screen.
            paused: Cell::new(false),
            // This flag is set when the Display was presenting when it received a VR Pause event.
            // When the VR Resume event is received and the flag is set, VR presentation automatically restarts.
            stopped_on_pause: Cell::new(false),
            xr_session: MutNullableDom::default(),
        }
    }

    pub fn new(global: &GlobalScope, display: WebVRDisplayData) -> DomRoot<VRDisplay> {
        reflect_dom_object(
            Box::new(VRDisplay::new_inherited(&global, display)),
            global,
            VRDisplayBinding::Wrap,
        )
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
        self.display.borrow().connected
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-ispresenting
    fn IsPresenting(&self) -> bool {
        self.presenting.get()
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-capabilities
    fn Capabilities(&self) -> DomRoot<VRDisplayCapabilities> {
        DomRoot::from_ref(&*self.capabilities.get())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-stageparameters
    fn GetStageParameters(&self) -> Option<DomRoot<VRStageParameters>> {
        self.stage_params.get().map(|s| DomRoot::from_ref(&*s))
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-geteyeparameters
    fn GetEyeParameters(&self, eye: VREye) -> DomRoot<VREyeParameters> {
        match eye {
            VREye::Left => DomRoot::from_ref(&*self.left_eye_params.get()),
            VREye::Right => DomRoot::from_ref(&*self.right_eye_params.get()),
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-displayid
    fn DisplayId(&self) -> u32 {
        self.display.borrow().display_id
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-displayname
    fn DisplayName(&self) -> DOMString {
        DOMString::from(self.display.borrow().display_name.clone())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-getframedata-framedata-framedata
    fn GetFrameData(&self, frameData: &VRFrameData) -> bool {
        // If presenting we use a synced data with compositor for the whole frame.
        // Frame data is only synced with compositor when GetFrameData is called from
        // inside the VRDisplay.requestAnimationFrame. This is checked using the running_display_raf property.
        // This check avoids data race conditions when calling GetFrameData from outside of the
        // VRDisplay.requestAnimationFrame callbacks and fixes a possible deadlock during the interval
        // when the requestAnimationFrame is moved from window to VRDisplay.
        if self.presenting.get() && self.running_display_raf.get() {
            if self.frame_data_status.get() == VRFrameDataStatus::Waiting {
                self.sync_frame_data();
            }
            frameData.update(&self.frame_data.borrow());
            return true;
        }

        // If not presenting we fetch inmediante VRFrameData
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        self.webvr_thread()
            .send(WebVRMsg::GetFrameData(
                self.global().pipeline_id(),
                self.DisplayId(),
                self.depth_near.get(),
                self.depth_far.get(),
                sender,
            ))
            .unwrap();
        return match receiver.recv().unwrap() {
            Ok(data) => {
                frameData.update(&data);
                true
            },
            Err(e) => {
                error!("WebVR::GetFrameData: {:?}", e);
                false
            },
        };
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-getpose
    fn GetPose(&self) -> DomRoot<VRPose> {
        VRPose::new(&self.global(), &self.frame_data.borrow().pose)
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-resetpose
    fn ResetPose(&self) {
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        self.webvr_thread()
            .send(WebVRMsg::ResetPose(
                self.global().pipeline_id(),
                self.DisplayId(),
                sender,
            ))
            .unwrap();
        if let Ok(data) = receiver.recv().unwrap() {
            // Some VRDisplay data might change after calling ResetPose()
            *self.display.borrow_mut() = data;
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthnear
    fn DepthNear(&self) -> Finite<f64> {
        Finite::wrap(self.depth_near.get())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthnear
    fn SetDepthNear(&self, value: Finite<f64>) {
        self.depth_near.set(*value.deref());
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthfar
    fn DepthFar(&self) -> Finite<f64> {
        Finite::wrap(self.depth_far.get())
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-depthfar
    fn SetDepthFar(&self, value: Finite<f64>) {
        self.depth_far.set(*value.deref());
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-requestanimationframe
    fn RequestAnimationFrame(&self, callback: Rc<FrameRequestCallback>) -> u32 {
        // For devices which are expecting the browser to present, we use the regular rAF
        if self.using_dedicated_raf_thread() {
            let raf_id = self.next_raf_id.get();
            self.next_raf_id.set(raf_id + 1);
            self.raf_callback_list
                .borrow_mut()
                .push((raf_id, Some(callback)));
            raf_id
        } else {
            // WebVR spec: When a VRDisplay is not presenting it should
            // fallback to window.requestAnimationFrame.
            self.global().as_window().RequestAnimationFrame(callback)
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-cancelanimationframe
    fn CancelAnimationFrame(&self, handle: u32) {
        if self.using_dedicated_raf_thread() {
            let mut list = self.raf_callback_list.borrow_mut();
            if let Some(pair) = list.iter_mut().find(|pair| pair.0 == handle) {
                pair.1 = None;
            }
        } else {
            // WebVR spec: When a VRDisplay is not presenting it should
            // fallback to window.cancelAnimationFrame.
            self.global().as_window().CancelAnimationFrame(handle);
        }
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-requestpresent
    fn RequestPresent(&self, layers: Vec<VRLayer>) -> Rc<Promise> {
        let promise = Promise::new(&self.global());
        // TODO: WebVR spec: this method must be called in response to a user gesture

        // WebVR spec: If canPresent is false the promise MUST be rejected
        if !self.display.borrow().capabilities.can_present {
            let msg = "VRDisplay canPresent is false".to_string();
            promise.reject_native(&msg);
            return promise;
        }

        // Current WebVRSpec only allows 1 VRLayer if the VRDevice can present.
        // Future revisions of this spec may allow multiple layers to enable more complex rendering effects
        // such as compositing WebGL and DOM elements together.
        // That functionality is not allowed by this revision of the spec.
        if layers.len() != 1 {
            let msg = "The number of layers must be 1".to_string();
            promise.reject_native(&msg);
            return promise;
        }

        // Parse and validate received VRLayer
        let layer = validate_layer(&layers[0]);

        let layer_bounds;
        let layer_ctx;

        match layer {
            Ok((bounds, ctx)) => {
                layer_bounds = bounds;
                layer_ctx = ctx;
            },
            Err(msg) => {
                let msg = msg.to_string();
                promise.reject_native(&msg);
                return promise;
            },
        };

        self.request_present(layer_bounds, Some(&layer_ctx), Some(promise.clone()), |p| {
            p.resolve_native(&())
        });
        promise
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-exitpresent
    fn ExitPresent(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.global());

        // WebVR spec: If the VRDisplay is not presenting the promise MUST be rejected.
        if !self.presenting.get() {
            let msg = "VRDisplay is not presenting".to_string();
            promise.reject_native(&msg);
            return promise;
        }

        // Exit present
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        self.webvr_thread()
            .send(WebVRMsg::ExitPresent(
                self.global().pipeline_id(),
                self.display.borrow().display_id,
                Some(sender),
            ))
            .unwrap();
        match receiver.recv().unwrap() {
            Ok(()) => {
                self.stop_present();
                promise.resolve_native(&());
            },
            Err(e) => {
                promise.reject_native(&e);
            },
        }

        promise
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-submitframe
    fn SubmitFrame(&self) {
        if !self.presenting.get() {
            warn!("VRDisplay not presenting");
            return;
        }

        let display_id = self.display.borrow().display_id;
        let layer = self.layer.borrow();
        let msg = WebVRCommand::SubmitFrame(display_id, layer.left_bounds, layer.right_bounds);
        if let Some(layer_ctx) = self.layer_ctx.get() {
            layer_ctx.send_vr_command(msg);
        }
    }

    // https://w3c.github.io/webvr/spec/1.1/#dom-vrdisplay-getlayers
    fn GetLayers(&self) -> Vec<VRLayer> {
        // WebVR spec: MUST return an empty array if the VRDisplay is not currently presenting
        if !self.presenting.get() {
            return Vec::new();
        }

        let layer = self.layer.borrow();

        vec![VRLayer {
            leftBounds: Some(bounds_to_vec(&layer.left_bounds)),
            rightBounds: Some(bounds_to_vec(&layer.right_bounds)),
            source: self.layer_ctx.get().map(|ctx| ctx.Canvas()),
        }]
    }
}

impl VRDisplay {
    fn webvr_thread(&self) -> IpcSender<WebVRMsg> {
        self.global()
            .as_window()
            .webvr_thread()
            .expect("Shouldn't arrive here with WebVR disabled")
    }

    pub fn update_display(&self, display: &WebVRDisplayData) {
        *self.display.borrow_mut() = display.clone();
        if let Some(ref stage) = display.stage_parameters {
            if self.stage_params.get().is_none() {
                let params = Some(VRStageParameters::new(stage.clone(), &self.global()));
                self.stage_params.set(params.as_ref().map(|v| v.deref()));
            } else {
                self.stage_params.get().unwrap().update(&stage);
            }
        } else {
            self.stage_params.set(None);
        }
    }

    pub fn request_present<F>(
        &self,
        layer_bounds: WebVRLayer,
        ctx: Option<&WebGLRenderingContext>,
        promise: Option<Rc<Promise>>,
        resolve: F,
    ) where
        F: FnOnce(Rc<Promise>) + Send + 'static,
    {
        // WebVR spec: Repeat calls while already presenting will update the VRLayers being displayed.
        if self.presenting.get() {
            *self.layer.borrow_mut() = layer_bounds;
            self.set_layer_ctx(ctx);
            promise.map(resolve);
            return;
        }

        // Request Present
        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        self.webvr_thread()
            .send(WebVRMsg::RequestPresent(
                self.global().pipeline_id(),
                self.display.borrow().display_id,
                sender,
            ))
            .unwrap();
        let promise = promise.map(TrustedPromise::new);
        let this = Trusted::new(self);
        let ctx = ctx.map(|c| Trusted::new(c));
        let global = self.global();
        let window = global.as_window();
        let (task_source, canceller) = window
            .task_manager()
            .dom_manipulation_task_source_with_canceller();
        thread::spawn(move || {
            let recv = receiver.recv().unwrap();
            let _ = task_source.queue_with_canceller(
                task!(vr_presenting: move || {
                    let this = this.root();
                    let promise = promise.map(|p| p.root());
                    let ctx = ctx.map(|c| c.root());
                    match recv {
                        Ok(()) => {
                            *this.layer.borrow_mut() = layer_bounds;
                            this.set_layer_ctx(ctx.as_ref().map(|c| &**c));
                            this.init_present();
                            promise.map(resolve);
                        },
                        Err(e) => {
                            promise.map(|p| p.reject_native(&e));
                        },
                    }
                }),
                &canceller,
            );
        });
    }

    fn set_layer_ctx(&self, ctx: Option<&WebGLRenderingContext>) {
        // For devices that expect the browser to present, we put the canvas into full-screen mode.
        if self.layer_ctx == ctx {
            return;
        }
        if !self.display.borrow().capabilities.presented_by_browser {
            self.layer_ctx.set(ctx);
            return;
        }

        // Enter fullscreen mode.
        // TODO: fullscreen mode isn't quite right because
        // a) it requires the element to be attached to the DOM tree, and
        // b) it fullscreens the element together with it's CSS styling.
        if let Some(old) = self.layer_ctx.get() {
            debug!("VR exiting full screen mode");
            let canvas = old.Canvas();
            let element = canvas.upcast::<Element>();
            let document = document_from_node(element);
            element.set_fullscreen_state(false);
            document.set_fullscreen_element(None);
        }
        if let Some(new) = ctx {
            debug!("VR entering full screen mode");
            let canvas = new.Canvas();
            let element = canvas.upcast::<Element>();
            let document = document_from_node(element);
            element.set_fullscreen_state(true);
            document.set_fullscreen_element(Some(element));
        }

        self.layer_ctx.set(ctx);
        let global = self.global();
        let window = global.as_window();
        window.reflow(ReflowGoal::Full, ReflowReason::ElementStateChanged);
    }

    pub fn handle_webvr_event(&self, event: &WebVRDisplayEvent) {
        match *event {
            WebVRDisplayEvent::Connect(ref display) => {
                self.update_display(&display);
            },
            WebVRDisplayEvent::Disconnect(_id) => {
                self.display.borrow_mut().connected = false;
            },
            WebVRDisplayEvent::Activate(ref display, _) |
            WebVRDisplayEvent::Deactivate(ref display, _) |
            WebVRDisplayEvent::Blur(ref display) |
            WebVRDisplayEvent::Focus(ref display) => {
                self.update_display(&display);
                self.notify_event(&event);
            },
            WebVRDisplayEvent::PresentChange(ref display, presenting) => {
                self.update_display(&display);
                self.presenting.set(presenting);
                self.notify_event(&event);
            },
            WebVRDisplayEvent::Change(ref display) => {
                // Change event doesn't exist in WebVR spec.
                // So we update display data but don't notify JS.
                self.update_display(&display);
            },
            WebVRDisplayEvent::Pause(_) => {
                if self.paused.get() {
                    return;
                }
                self.paused.set(true);
                if self.presenting.get() {
                    self.stop_present();
                    self.stopped_on_pause.set(true);
                }
            },
            WebVRDisplayEvent::Resume(_) => {
                self.paused.set(false);
                if self.stopped_on_pause.get() {
                    self.stopped_on_pause.set(false);
                    self.init_present();
                }
            },
            WebVRDisplayEvent::Exit(_) => {
                self.stopped_on_pause.set(false);
                if self.presenting.get() {
                    self.stop_present();
                }
            },
        };
    }

    fn notify_event(&self, event: &WebVRDisplayEvent) {
        let root = DomRoot::from_ref(&*self);
        let event = VRDisplayEvent::new_from_webvr(&self.global(), &root, &event);
        event
            .upcast::<Event>()
            .fire(self.global().upcast::<EventTarget>());
    }

    fn init_present(&self) {
        self.presenting.set(true);
        let (sync_sender, sync_receiver) = webgl_channel().unwrap();
        *self.frame_data_receiver.borrow_mut() = Some(sync_receiver);

        let display_id = self.display.borrow().display_id;
        let api_sender = match self.layer_ctx.get() {
            Some(layer_ctx) => layer_ctx.webgl_sender(),
            None => return,
        };
        let js_sender = self.global().script_chan();
        let address = Trusted::new(&*self);
        let near_init = self.depth_near.get();
        let far_init = self.depth_far.get();
        let pipeline_id = self.global().pipeline_id();

        // For displays with no external display, we use the regular window
        // rAF, so we don't need to spin up a dedicated thread.
        if !self.using_dedicated_raf_thread() {
            // Initialize compositor
            api_sender
                .send_vr(WebVRCommand::Create(display_id))
                .unwrap();

            // No need to spin up a rAF thread
            return;
        }

        // The render loop at native headset frame rate is implemented using a dedicated thread.
        // Every loop iteration syncs pose data with the HMD, submits the pixels to the display and waits for Vsync.
        // Both the requestAnimationFrame call of a VRDisplay in the JavaScript thread and the VRSyncPoses call
        // in the Webrender thread are executed in parallel. This allows to get some JavaScript code executed ahead.
        // while the render thread is syncing the VRFrameData to be used for the current frame.
        // This thread runs until the user calls ExitPresent, the tab is closed or some unexpected error happened.
        thread::Builder::new()
            .name("WebVR_RAF".into())
            .spawn(move || {
                let (raf_sender, raf_receiver) = unbounded();
                let mut near = near_init;
                let mut far = far_init;

                // Initialize compositor
                api_sender
                    .send_vr(WebVRCommand::Create(display_id))
                    .unwrap();
                loop {
                    // Run RAF callbacks on JavaScript thread
                    let this = address.clone();
                    let sender = raf_sender.clone();
                    let task = Box::new(task!(handle_vrdisplay_raf: move || {
                        this.root().handle_raf(&sender);
                    }));
                    // NOTE: WebVR spec doesn't specify what task source we should use. Is
                    // dom-manipulation a good choice long term?
                    js_sender
                        .send(CommonScriptMsg::Task(
                            WebVREvent,
                            task,
                            Some(pipeline_id),
                            TaskSourceName::DOMManipulation,
                        ))
                        .unwrap();

                    // Run Sync Poses in parallell on Render thread
                    let msg = WebVRCommand::SyncPoses(display_id, near, far, sync_sender.clone());
                    api_sender.send_vr(msg).unwrap();

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
            })
            .expect("Thread spawning failed");
    }

    fn stop_present(&self) {
        self.presenting.set(false);
        *self.frame_data_receiver.borrow_mut() = None;

        let api_sender = match self.layer_ctx.get() {
            Some(layer_ctx) => layer_ctx.webgl_sender(),
            None => return,
        };
        let display_id = self.display.borrow().display_id;
        api_sender
            .send_vr(WebVRCommand::Release(display_id))
            .unwrap();
    }

    // Only called when the JSContext is destroyed while presenting.
    // In this case we don't want to wait for WebVR Thread response.
    fn force_stop_present(&self) {
        self.webvr_thread()
            .send(WebVRMsg::ExitPresent(
                self.global().pipeline_id(),
                self.display.borrow().display_id,
                None,
            ))
            .unwrap();
        self.stop_present();
    }

    fn sync_frame_data(&self) {
        let status = if let Some(receiver) = self.frame_data_receiver.borrow().as_ref() {
            match receiver.recv().unwrap() {
                Ok(bytes) => {
                    *self.frame_data.borrow_mut() = WebVRFrameData::from_bytes(&bytes[..]);
                    VRFrameDataStatus::Synced
                },
                Err(()) => VRFrameDataStatus::Exit,
            }
        } else {
            VRFrameDataStatus::Exit
        };

        self.frame_data_status.set(status);
    }

    fn using_dedicated_raf_thread(&self) -> bool {
        // We spin up a dedicated rAF thread when we're presenting, and
        // when the display does it's own presentation rather than asking
        // the browser to do it.
        self.presenting.get() && !self.display.borrow().capabilities.presented_by_browser
    }

    fn handle_raf(&self, end_sender: &Sender<Result<(f64, f64), ()>>) {
        self.frame_data_status.set(VRFrameDataStatus::Waiting);

        let now = self.global().as_window().Performance().Now();

        if let Some(session) = self.xr_session.get() {
            let mut callbacks = mem::replace(&mut *self.xr_raf_callback_list.borrow_mut(), vec![]);
            if callbacks.is_empty() {
                return;
            }
            self.sync_frame_data();
            let frame = XRFrame::new(&self.global(), &session, self.frame_data.borrow().clone());

            for (_, callback) in callbacks.drain(..) {
                if let Some(callback) = callback {
                    let _ = callback.Call__(Finite::wrap(*now), &frame, ExceptionHandling::Report);
                }
            }
            // frame submission is automatic in XR
            self.SubmitFrame();
        } else {
            self.running_display_raf.set(true);
            let mut callbacks = mem::replace(&mut *self.raf_callback_list.borrow_mut(), vec![]);
            // Call registered VRDisplay.requestAnimationFrame callbacks.
            for (_, callback) in callbacks.drain(..) {
                if let Some(callback) = callback {
                    let _ = callback.Call__(Finite::wrap(*now), ExceptionHandling::Report);
                }
            }

            self.running_display_raf.set(false);
            if self.frame_data_status.get() == VRFrameDataStatus::Waiting {
                // User didn't call getFrameData while presenting.
                // We automatically reads the pending VRFrameData to avoid overflowing the IPC-Channel buffers.
                // Show a warning as the WebVR Spec recommends.
                warn!("WebVR: You should call GetFrameData while presenting");
                self.sync_frame_data();
            }
        }

        match self.frame_data_status.get() {
            VRFrameDataStatus::Synced => {
                // Sync succeeded. Notify RAF thread.
                end_sender
                    .send(Ok((self.depth_near.get(), self.depth_far.get())))
                    .unwrap();
            },
            VRFrameDataStatus::Exit | VRFrameDataStatus::Waiting => {
                // ExitPresent called or some error ocurred.
                // Notify VRDisplay RAF thread to stop.
                end_sender.send(Err(())).unwrap();
            },
        }
    }
}

// XR stuff
// XXXManishearth eventually we should share as much logic as possible
impl VRDisplay {
    pub fn xr_present(
        &self,
        session: &XRSession,
        ctx: Option<&WebGLRenderingContext>,
        promise: Option<Rc<Promise>>,
    ) {
        let layer_bounds = WebVRLayer::default();
        self.xr_session.set(Some(session));
        let session = Trusted::new(session);
        self.request_present(layer_bounds, ctx, promise, move |p| {
            let session = session.root();
            p.resolve_native(&session);
        });
    }

    pub fn xr_raf(&self, callback: Rc<XRFrameRequestCallback>) -> u32 {
        let raf_id = self.next_raf_id.get();
        self.next_raf_id.set(raf_id + 1);
        self.xr_raf_callback_list
            .borrow_mut()
            .push((raf_id, Some(callback)));
        raf_id
    }

    pub fn xr_cancel_raf(&self, handle: i32) {
        let mut list = self.xr_raf_callback_list.borrow_mut();
        if let Some(pair) = list.iter_mut().find(|pair| pair.0 == handle as u32) {
            pair.1 = None;
        }
    }
}

// WebVR Spec: If the number of values in the leftBounds/rightBounds arrays
// is not 0 or 4 for any of the passed layers the promise is rejected
fn parse_bounds(src: &Option<Vec<Finite<f32>>>, dst: &mut [f32; 4]) -> Result<(), &'static str> {
    match *src {
        Some(ref values) => {
            if values.len() == 0 {
                return Ok(());
            }
            if values.len() != 4 {
                return Err(
                    "The number of values in the leftBounds/rightBounds arrays must be 0 or 4",
                );
            }
            for i in 0..4 {
                dst[i] = *values[i].deref();
            }
            Ok(())
        },
        None => Ok(()),
    }
}

fn validate_layer(
    layer: &VRLayer,
) -> Result<(WebVRLayer, DomRoot<WebGLRenderingContext>), &'static str> {
    let ctx = layer
        .source
        .as_ref()
        .map(|ref s| s.get_base_webgl_context())
        .unwrap_or(None);
    if let Some(ctx) = ctx {
        let mut data = WebVRLayer::default();
        parse_bounds(&layer.leftBounds, &mut data.left_bounds)?;
        parse_bounds(&layer.rightBounds, &mut data.right_bounds)?;
        Ok((data, ctx))
    } else {
        Err("VRLayer source must be a WebGL Context")
    }
}

fn bounds_to_vec(src: &[f32; 4]) -> Vec<Finite<f32>> {
    vec![
        Finite::wrap(src[0]),
        Finite::wrap(src[1]),
        Finite::wrap(src[2]),
        Finite::wrap(src[3]),
    ]
}
