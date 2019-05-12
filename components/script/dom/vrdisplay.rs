/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::compartments::{AlreadyInCompartment, InCompartment};
use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigatorBinding::NavigatorMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VREye;
use crate::dom::bindings::codegen::Bindings::VRLayerBinding::VRLayer;
use crate::dom::bindings::codegen::Bindings::WebGLRenderingContextBinding::WebGLRenderingContextMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::FrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::Bindings::XRRenderStateBinding::XRRenderStateInit;
use crate::dom::bindings::codegen::Bindings::XRSessionBinding::XRFrameRequestCallback;
use crate::dom::bindings::codegen::Bindings::XRWebGLLayerBinding::XRWebGLLayerMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::vrdisplaycapabilities::VRDisplayCapabilities;
use crate::dom::vrdisplayevent::VRDisplayEvent;
use crate::dom::vreyeparameters::VREyeParameters;
use crate::dom::vrframedata::VRFrameData;
use crate::dom::vrpose::VRPose;
use crate::dom::vrstageparameters::VRStageParameters;
use crate::dom::webglrenderingcontext::WebGLRenderingContext;
use crate::dom::xrframe::XRFrame;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrsession::XRSession;
use crate::dom::xrwebgllayer::XRWebGLLayer;
use crate::script_runtime::CommonScriptMsg;
use crate::script_runtime::ScriptThreadEventCategory::WebVREvent;
use crate::task_source::{TaskSource, TaskSourceName};
use canvas_traits::webgl::{webgl_channel, WebGLMsgSender, WebGLReceiver, WebVRCommand};
use crossbeam_channel::{unbounded, Sender};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use profile_traits::ipc;
use std::cell::Cell;
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::thread;
use webvr_traits::{WebVRDisplayData, WebVRDisplayEvent, WebVRFrameData, WebVRPoseInformation};
use webvr_traits::{WebVRLayer, WebVRMsg};

#[dom_struct]
pub struct VRDisplay {
    eventtarget: EventTarget,
    #[ignore_malloc_size_of = "Defined in rust-webvr"]
    display: DomRefCell<WebVRDisplayData>,
    depth_near: Cell<f64>,
    depth_far: Cell<f64>,
    presenting: Cell<bool>,
    has_raf_thread: Cell<bool>,
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
    /// When there isn't any layer_ctx the RAF thread needs to be "woken up"
    raf_wakeup_sender: DomRefCell<Option<Sender<()>>>,
    #[ignore_malloc_size_of = "Rc is hard"]
    pending_renderstate_updates: DomRefCell<Vec<(XRRenderStateInit, Rc<Promise>)>>,
    // Compositor VRFrameData synchonization
    frame_data_status: Cell<VRFrameDataStatus>,
    #[ignore_malloc_size_of = "closures are hard"]
    frame_data_receiver: DomRefCell<Option<WebGLReceiver<Result<WebVRPoseInformation, ()>>>>,
    running_display_raf: Cell<bool>,
    paused: Cell<bool>,
    stopped_on_pause: Cell<bool>,
    /// Whether or not this is XR mode, and the session
    xr_session: MutNullableDom<XRSession>,
    /// Have inputs been initialized? (i.e, has getInputSources() been called?)
    /// XR only
    initialized_inputs: Cell<bool>,
    input_sources: DomRefCell<HashMap<u32, Dom<XRInputSource>>>,
}

unsafe_no_jsmanaged_fields!(WebVRDisplayData);
unsafe_no_jsmanaged_fields!(WebVRFrameData);
unsafe_no_jsmanaged_fields!(WebVRLayer);
unsafe_no_jsmanaged_fields!(VRFrameDataStatus);

#[derive(Clone, Copy, Eq, MallocSizeOf, PartialEq)]
enum VRFrameDataStatus {
    Waiting,
    Synced,
    Exit,
}

#[derive(Clone, MallocSizeOf)]
struct VRRAFUpdate {
    depth_near: f64,
    depth_far: f64,
    /// WebGL API sender
    api_sender: Option<WebGLMsgSender>,
    /// Number uniquely identifying the WebGL context
    /// so that we may setup/tear down VR compositors as things change
    context_id: usize,
    /// Do we need input data?
    needs_inputs: bool,
}

type VRRAFUpdateSender = Sender<Result<VRRAFUpdate, ()>>;

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
            has_raf_thread: Cell::new(false),
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
            raf_wakeup_sender: DomRefCell::new(None),
            pending_renderstate_updates: DomRefCell::new(vec![]),
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
            initialized_inputs: Cell::new(false),
            input_sources: DomRefCell::new(HashMap::new()),
        }
    }

    pub fn new(global: &GlobalScope, display: WebVRDisplayData) -> DomRoot<VRDisplay> {
        reflect_dom_object(
            Box::new(VRDisplay::new_inherited(&global, display)),
            global,
            VRDisplayBinding::Wrap,
        )
    }

    pub fn left_eye_params_offset(&self) -> [f32; 3] {
        self.left_eye_params.get().offset_array()
    }

    pub fn right_eye_params_offset(&self) -> [f32; 3] {
        self.right_eye_params.get().offset_array()
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
        if self.presenting.get() {
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
        if self.presenting.get() {
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
        let in_compartment_proof = AlreadyInCompartment::assert(&self.global());
        let promise = Promise::new_in_current_compartment(
            &self.global(),
            InCompartment::Already(&in_compartment_proof),
        );
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

        // WebVR spec: Repeat calls while already presenting will update the VRLayers being displayed.
        if self.presenting.get() {
            *self.layer.borrow_mut() = layer_bounds;
            self.layer_ctx.set(Some(&layer_ctx));
            promise.resolve_native(&());
            return promise;
        }

        let xr = self.global().as_window().Navigator().Xr();

        if xr.pending_or_active_session() {
            // WebVR spec doesn't mandate anything here, however
            // the WebXR spec expects there to be only one immersive XR session at a time,
            // and WebVR is deprecated
            promise.reject_error(Error::InvalidState);
            return promise;
        }

        self.request_present(layer_bounds, Some(&layer_ctx), Some(promise.clone()), |p| {
            p.resolve_native(&())
        });
        promise
    }

    // https://w3c.github.io/webvr/#dom-vrdisplay-exitpresent
    fn ExitPresent(&self) -> Rc<Promise> {
        let in_compartment_proof = AlreadyInCompartment::assert(&self.global());
        let promise = Promise::new_in_current_compartment(
            &self.global(),
            InCompartment::Already(&in_compartment_proof),
        );

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
        self.layer_ctx
            .get()
            .expect("SubmitFrame can only be called when there is a webgl layer")
            .send_vr_command(msg);
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
        // FIXME: use a dedicated VR task-source.
        let (task_source, canceller) = window
            .task_manager()
            .media_element_task_source_with_canceller();
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
                            this.layer_ctx.set(ctx.as_ref().map(|c| &**c));
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

    fn api_sender(&self) -> Option<WebGLMsgSender> {
        self.layer_ctx.get().map(|c| c.webgl_sender())
    }

    fn context_id(&self) -> usize {
        self.layer_ctx
            .get()
            .map(|c| &*c as *const WebGLRenderingContext as usize)
            .unwrap_or(0)
    }

    fn vr_raf_update(&self) -> VRRAFUpdate {
        VRRAFUpdate {
            depth_near: self.depth_near.get(),
            depth_far: self.depth_far.get(),
            api_sender: self.api_sender(),
            context_id: self.context_id(),
            needs_inputs: self.initialized_inputs.get(),
        }
    }

    pub fn queue_renderstate(&self, state: &XRRenderStateInit, promise: Rc<Promise>) {
        // can't clone dictionaries
        let new_state = XRRenderStateInit {
            depthNear: state.depthNear,
            depthFar: state.depthFar,
            baseLayer: state.baseLayer.clone(),
        };
        self.pending_renderstate_updates
            .borrow_mut()
            .push((new_state, promise));

        if let Some(ref wakeup) = *self.raf_wakeup_sender.borrow() {
            let _ = wakeup.send(());
        }
    }

    fn process_renderstate_queue(&self) {
        let mut updates = self.pending_renderstate_updates.borrow_mut();

        debug_assert!(updates.is_empty() || self.xr_session.get().is_some());
        for update in updates.drain(..) {
            if let Some(near) = update.0.depthNear {
                self.depth_near.set(*near);
            }
            if let Some(far) = update.0.depthFar {
                self.depth_far.set(*far);
            }
            if let Some(ref layer) = update.0.baseLayer {
                self.xr_session.get().unwrap().set_layer(&layer);
                let layer = layer.downcast::<XRWebGLLayer>().unwrap();
                self.layer_ctx.set(Some(&layer.Context()));
            }
            update.1.resolve_native(&());
        }
    }

    fn init_present(&self) {
        self.presenting.set(true);
        let xr = self.global().as_window().Navigator().Xr();
        xr.set_active_immersive_session(&self);
        self.process_renderstate_queue();
        if self.has_raf_thread.get() {
            return;
        }
        self.has_raf_thread.set(true);
        let (sync_sender, sync_receiver) = webgl_channel().unwrap();
        *self.frame_data_receiver.borrow_mut() = Some(sync_receiver);

        let display_id = self.display.borrow().display_id;
        let mut api_sender = self.api_sender();
        let mut context_id = self.context_id();
        let js_sender = self.global().script_chan();
        let address = Trusted::new(&*self);
        let mut near = self.depth_near.get();
        let mut far = self.depth_far.get();
        let pipeline_id = self.global().pipeline_id();

        let (raf_sender, raf_receiver) = unbounded();
        let (wakeup_sender, wakeup_receiver) = unbounded();
        *self.raf_wakeup_sender.borrow_mut() = Some(wakeup_sender);
        let mut needs_inputs = false;

        // The render loop at native headset frame rate is implemented using a dedicated thread.
        // Every loop iteration syncs pose data with the HMD, submits the pixels to the display and waits for Vsync.
        // Both the requestAnimationFrame call of a VRDisplay in the JavaScript thread and the VRSyncPoses call
        // in the Webrender thread are executed in parallel. This allows to get some JavaScript code executed ahead.
        // while the render thread is syncing the VRFrameData to be used for the current frame.
        // This thread runs until the user calls ExitPresent, the tab is closed or some unexpected error happened.
        thread::Builder::new()
            .name("WebVR_RAF".into())
            .spawn(move || {
                // Initialize compositor
                if let Some(ref api_sender) = api_sender {
                    api_sender
                        .send_vr(WebVRCommand::Create(display_id))
                        .unwrap();
                }
                loop {
                    if let Some(ref api_sender) = api_sender {
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
                        let msg = WebVRCommand::SyncPoses(
                            display_id,
                            near,
                            far,
                            needs_inputs,
                            sync_sender.clone(),
                        );
                        api_sender.send_vr(msg).unwrap();
                    } else {
                        let _ = wakeup_receiver.recv();
                        let sender = raf_sender.clone();
                        let this = address.clone();
                        let task = Box::new(task!(flush_renderstate_queue: move || {
                            let this = this.root();
                            this.process_renderstate_queue();
                            sender.send(Ok(this.vr_raf_update())).unwrap();
                        }));
                        js_sender
                            .send(CommonScriptMsg::Task(
                                WebVREvent,
                                task,
                                Some(pipeline_id),
                                TaskSourceName::DOMManipulation,
                            ))
                            .unwrap();
                    }

                    // Wait until both SyncPoses & RAF ends
                    if let Ok(update) = raf_receiver.recv().unwrap() {
                        near = update.depth_near;
                        far = update.depth_far;
                        needs_inputs = update.needs_inputs;
                        if update.context_id != context_id {
                            if let Some(ref api_sender) = update.api_sender {
                                api_sender
                                    .send_vr(WebVRCommand::Create(display_id))
                                    .unwrap();
                            }
                            if let Some(ref api_sender) = api_sender {
                                // shut down old vr compositor
                                api_sender
                                    .send_vr(WebVRCommand::Release(display_id))
                                    .unwrap();
                            }
                            context_id = update.context_id;
                        }

                        api_sender = update.api_sender;
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
        let xr = self.global().as_window().Navigator().Xr();
        xr.deactivate_session();
        *self.frame_data_receiver.borrow_mut() = None;
        self.has_raf_thread.set(false);
        if let Some(api_sender) = self.api_sender() {
            let display_id = self.display.borrow().display_id;
            api_sender
                .send_vr(WebVRCommand::Release(display_id))
                .unwrap();
        }
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
                Ok(pose) => {
                    *self.frame_data.borrow_mut() = pose.frame.block();
                    if self.initialized_inputs.get() {
                        let inputs = self.input_sources.borrow();
                        for (id, state) in pose.gamepads {
                            if let Some(input) = inputs.get(&id) {
                                input.update_state(state);
                            }
                        }
                    }
                    VRFrameDataStatus::Synced
                },
                Err(()) => VRFrameDataStatus::Exit,
            }
        } else {
            VRFrameDataStatus::Exit
        };

        self.frame_data_status.set(status);
    }

    fn handle_raf(&self, end_sender: &VRRAFUpdateSender) {
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

        self.process_renderstate_queue();
        match self.frame_data_status.get() {
            VRFrameDataStatus::Synced => {
                // Sync succeeded. Notify RAF thread.
                end_sender.send(Ok(self.vr_raf_update())).unwrap();
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

    /// Initialize XRInputSources
    fn initialize_inputs(&self) {
        if self.initialized_inputs.get() {
            return;
        }
        self.initialized_inputs.set(true);

        let (sender, receiver) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
        let display = self.display.borrow().display_id;
        self.webvr_thread()
            .send(WebVRMsg::GetGamepadsForDisplay(display, sender))
            .unwrap();
        match receiver.recv().unwrap() {
            Ok(gamepads) => {
                let global = self.global();
                let session = self
                    .xr_session
                    .get()
                    .expect("initialize_inputs called on a VR session");
                let roots: Vec<_> = gamepads
                    .into_iter()
                    .map(|g| {
                        (
                            g.1.gamepad_id,
                            XRInputSource::new(&global, &session, g.0, g.1),
                        )
                    })
                    .collect();

                let mut inputs = self.input_sources.borrow_mut();
                for (id, root) in &roots {
                    inputs.insert(*id, Dom::from_ref(&root));
                }
            },
            Err(_) => {},
        }
    }

    pub fn get_input_sources(&self) -> Vec<DomRoot<XRInputSource>> {
        self.initialize_inputs();
        self.input_sources
            .borrow()
            .iter()
            .map(|(_, x)| DomRoot::from_ref(&**x))
            .collect()
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
