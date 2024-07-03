/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use base::id::{PipelineId, PipelineNamespace};
use crossbeam_channel::Receiver;
use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::panic::maybe_resume_unwind;
use js::rust::{HandleValue, ParentRuntime};
use net_traits::request::{
    CredentialsMode, Destination, ParserMetadata, RequestBuilder as NetRequestInit,
};
use net_traits::IpcSend;
use script_traits::WorkerGlobalScopeInit;
use servo_url::{MutableOrigin, ServoUrl};
use time::precise_time_ns;
use uuid::Uuid;

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::ImageBitmapBinding::{
    ImageBitmapOptions, ImageBitmapSource,
};
use crate::dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use crate::dom::bindings::codegen::Bindings::VoidFunctionBinding::VoidFunction;
use crate::dom::bindings::codegen::Bindings::WorkerBinding::WorkerType;
use crate::dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use crate::dom::bindings::codegen::UnionTypes::{RequestOrUSVString, StringOrFunction};
use crate::dom::bindings::error::{report_pending_exception, Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::crypto::Crypto;
use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use crate::dom::globalscope::GlobalScope;
use crate::dom::identityhub::Identities;
use crate::dom::performance::Performance;
use crate::dom::promise::Promise;
use crate::dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use crate::dom::window::{base64_atob, base64_btoa};
use crate::dom::workerlocation::WorkerLocation;
use crate::dom::workernavigator::WorkerNavigator;
use crate::fetch;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{
    get_reports, CommonScriptMsg, ContextForRequestInterrupt, JSContext, Runtime, ScriptChan,
    ScriptPort,
};
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::file_reading::FileReadingTaskSource;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::performance_timeline::PerformanceTimelineTaskSource;
use crate::task_source::port_message::PortMessageQueue;
use crate::task_source::remote_event::RemoteEventTaskSource;
use crate::task_source::timer::TimerTaskSource;
use crate::task_source::websocket::WebsocketTaskSource;
use crate::timers::{IsInterval, TimerCallback};

pub fn prepare_workerscope_init(
    global: &GlobalScope,
    devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,
    worker_id: Option<WorkerId>,
) -> WorkerGlobalScopeInit {
    let init = WorkerGlobalScopeInit {
        resource_threads: global.resource_threads().clone(),
        mem_profiler_chan: global.mem_profiler_chan().clone(),
        to_devtools_sender: global.devtools_chan().cloned(),
        time_profiler_chan: global.time_profiler_chan().clone(),
        from_devtools_sender: devtools_sender,
        script_to_constellation_chan: global.script_to_constellation_chan().clone(),
        scheduler_chan: global.scheduler_chan().clone(),
        worker_id: worker_id.unwrap_or_else(|| WorkerId(Uuid::new_v4())),
        pipeline_id: global.pipeline_id(),
        origin: global.origin().immutable().clone(),
        creation_url: global.creation_url().clone(),
        is_headless: global.is_headless(),
        user_agent: global.get_user_agent(),
        inherited_secure_context: Some(global.is_secure_context()),
    };

    init
}

// https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface
#[dom_struct]
pub struct WorkerGlobalScope {
    globalscope: GlobalScope,

    worker_name: DOMString,
    worker_type: WorkerType,

    #[no_trace]
    worker_id: WorkerId,
    #[no_trace]
    worker_url: DomRefCell<ServoUrl>,
    #[ignore_malloc_size_of = "Arc"]
    closing: Arc<AtomicBool>,
    #[ignore_malloc_size_of = "Defined in js"]
    runtime: DomRefCell<Option<Runtime>>,
    location: MutNullableDom<WorkerLocation>,
    navigator: MutNullableDom<WorkerNavigator>,

    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    #[no_trace]
    /// Optional `IpcSender` for sending the `DevtoolScriptControlMsg`
    /// to the server from within the worker
    from_devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,

    #[ignore_malloc_size_of = "Defined in std"]
    #[no_trace]
    /// This `Receiver` will be ignored later if the corresponding
    /// `IpcSender` doesn't exist
    from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,

    navigation_start_precise: u64,
    performance: MutNullableDom<Performance>,
}

impl WorkerGlobalScope {
    #[allow(clippy::too_many_arguments)]
    pub fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_name: DOMString,
        worker_type: WorkerType,
        worker_url: ServoUrl,
        runtime: Runtime,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        closing: Arc<AtomicBool>,
        gpu_id_hub: Arc<Identities>,
    ) -> Self {
        // Install a pipeline-namespace in the current thread.
        PipelineNamespace::auto_install();
        Self {
            globalscope: GlobalScope::new_inherited(
                init.pipeline_id,
                init.to_devtools_sender,
                init.mem_profiler_chan,
                init.time_profiler_chan,
                init.script_to_constellation_chan,
                init.scheduler_chan,
                init.resource_threads,
                MutableOrigin::new(init.origin),
                init.creation_url,
                runtime.microtask_queue.clone(),
                init.is_headless,
                init.user_agent,
                gpu_id_hub,
                init.inherited_secure_context,
            ),
            worker_id: init.worker_id,
            worker_name,
            worker_type,
            worker_url: DomRefCell::new(worker_url),
            closing,
            runtime: DomRefCell::new(Some(runtime)),
            location: Default::default(),
            navigator: Default::default(),
            from_devtools_sender: init.from_devtools_sender,
            from_devtools_receiver,
            navigation_start_precise: precise_time_ns(),
            performance: Default::default(),
        }
    }

    /// Clear various items when the worker event-loop shuts-down.
    pub fn clear_js_runtime(&self, cx_for_interrupt: ContextForRequestInterrupt) {
        // Ensure parent thread can no longer request interrupt
        // using our JSContext that will soon be destroyed
        cx_for_interrupt.revoke();
        self.upcast::<GlobalScope>()
            .remove_web_messaging_and_dedicated_workers_infra();

        // Drop the runtime.
        let runtime = self.runtime.borrow_mut().take();
        drop(runtime);
    }

    pub fn runtime_handle(&self) -> ParentRuntime {
        self.runtime
            .borrow()
            .as_ref()
            .unwrap()
            .prepare_for_new_child()
    }

    pub fn from_devtools_sender(&self) -> Option<IpcSender<DevtoolScriptControlMsg>> {
        self.from_devtools_sender.clone()
    }

    pub fn from_devtools_receiver(&self) -> &Receiver<DevtoolScriptControlMsg> {
        &self.from_devtools_receiver
    }

    #[allow(unsafe_code)]
    pub fn get_cx(&self) -> JSContext {
        unsafe { JSContext::from_ptr(self.runtime.borrow().as_ref().unwrap().cx()) }
    }

    pub fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    pub fn get_url(&self) -> Ref<ServoUrl> {
        self.worker_url.borrow()
    }

    pub fn set_url(&self, url: ServoUrl) {
        *self.worker_url.borrow_mut() = url;
    }

    pub fn get_worker_id(&self) -> WorkerId {
        self.worker_id
    }

    pub fn task_canceller(&self) -> TaskCanceller {
        TaskCanceller {
            cancelled: self.closing.clone(),
        }
    }

    pub fn pipeline_id(&self) -> PipelineId {
        self.globalscope.pipeline_id()
    }
}

impl WorkerGlobalScopeMethods for WorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-self
    fn Self_(&self) -> DomRoot<WorkerGlobalScope> {
        DomRoot::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-location
    fn Location(&self) -> DomRoot<WorkerLocation> {
        self.location
            .or_init(|| WorkerLocation::new(self, self.worker_url.borrow().clone()))
    }

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    error_event_handler!(error, GetOnerror, SetOnerror);

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
    fn ImportScripts(&self, url_strings: Vec<DOMString>) -> ErrorResult {
        let mut urls = Vec::with_capacity(url_strings.len());
        for url in url_strings {
            let url = self.worker_url.borrow().join(&url);
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Error::Syntax),
            };
        }

        rooted!(in(self.runtime.borrow().as_ref().unwrap().cx()) let mut rval = UndefinedValue());
        for url in urls {
            let global_scope = self.upcast::<GlobalScope>();
            let request = NetRequestInit::new(url.clone(), global_scope.get_referrer())
                .destination(Destination::Script)
                .credentials_mode(CredentialsMode::Include)
                .parser_metadata(ParserMetadata::NotParserInserted)
                .use_url_credentials(true)
                .origin(global_scope.origin().immutable().clone())
                .pipeline_id(Some(self.upcast::<GlobalScope>().pipeline_id()))
                .referrer_policy(None);

            let (url, source) = match fetch::load_whole_resource(
                request,
                &global_scope.resource_threads().sender(),
                global_scope,
            ) {
                Err(_) => return Err(Error::Network),
                Ok((metadata, bytes)) => (metadata.final_url, String::from_utf8(bytes).unwrap()),
            };

            let result = self.runtime.borrow().as_ref().unwrap().evaluate_script(
                self.reflector().get_jsobject(),
                &source,
                url.as_str(),
                1,
                rval.handle_mut(),
            );

            maybe_resume_unwind();

            match result {
                Ok(_) => (),
                Err(_) => {
                    if self.is_closing() {
                        // Don't return JSFailed as we might not have
                        // any pending exceptions.
                        println!("evaluate_script failed (terminated)");
                    } else {
                        println!("evaluate_script failed");
                        return Err(Error::JSFailed);
                    }
                },
            }
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker-navigator
    fn Navigator(&self) -> DomRoot<WorkerNavigator> {
        self.navigator.or_init(|| WorkerNavigator::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dfn-Crypto
    fn Crypto(&self) -> DomRoot<Crypto> {
        self.upcast::<GlobalScope>().crypto()
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa
    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob
    fn Atob(&self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    fn SetTimeout(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            timeout,
            IsInterval::NonInterval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.upcast::<GlobalScope>()
            .clear_timeout_or_interval(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            callback,
            args,
            timeout,
            IsInterval::Interval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-queuemicrotask
    fn QueueMicrotask(&self, callback: Rc<VoidFunction>) {
        self.upcast::<GlobalScope>()
            .queue_function_as_microtask(callback);
    }

    // https://html.spec.whatwg.org/multipage/#dom-createimagebitmap
    fn CreateImageBitmap(
        &self,
        image: ImageBitmapSource,
        options: &ImageBitmapOptions,
    ) -> Rc<Promise> {
        let p = self
            .upcast::<GlobalScope>()
            .create_image_bitmap(image, options);
        p
    }

    #[allow(crown::unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#fetch-method
    fn Fetch(
        &self,
        input: RequestOrUSVString,
        init: RootedTraceableBox<RequestInit>,
        comp: InRealm,
    ) -> Rc<Promise> {
        fetch::Fetch(self.upcast(), input, init, comp)
    }

    // https://w3c.github.io/hr-time/#the-performance-attribute
    fn Performance(&self) -> DomRoot<Performance> {
        self.performance.or_init(|| {
            let global_scope = self.upcast::<GlobalScope>();
            Performance::new(global_scope, self.navigation_start_precise)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-origin
    fn Origin(&self) -> USVString {
        USVString(
            self.upcast::<GlobalScope>()
                .origin()
                .immutable()
                .ascii_serialization(),
        )
    }

    // https://w3c.github.io/webappsec-secure-contexts/#dom-windoworworkerglobalscope-issecurecontext
    fn IsSecureContext(&self) -> bool {
        self.upcast::<GlobalScope>().is_secure_context()
    }
}

impl WorkerGlobalScope {
    #[allow(unsafe_code)]
    pub fn execute_script(&self, source: DOMString) {
        let _aes = AutoEntryScript::new(self.upcast());
        let cx = self.runtime.borrow().as_ref().unwrap().cx();
        rooted!(in(cx) let mut rval = UndefinedValue());
        match self.runtime.borrow().as_ref().unwrap().evaluate_script(
            self.reflector().get_jsobject(),
            &source,
            self.worker_url.borrow().as_str(),
            1,
            rval.handle_mut(),
        ) {
            Ok(_) => (),
            Err(_) => {
                if self.is_closing() {
                    println!("evaluate_script failed (terminated)");
                } else {
                    // TODO: An error needs to be dispatched to the parent.
                    // https://github.com/servo/servo/issues/6422
                    println!("evaluate_script failed");
                    unsafe {
                        let ar = enter_realm(self);
                        report_pending_exception(cx, true, InRealm::Entered(&ar));
                    }
                }
            },
        }
    }

    pub fn script_chan(&self) -> Box<dyn ScriptChan + Send> {
        let dedicated = self.downcast::<DedicatedWorkerGlobalScope>();
        let service_worker = self.downcast::<ServiceWorkerGlobalScope>();
        if let Some(dedicated) = dedicated {
            dedicated.script_chan()
        } else if let Some(service_worker) = service_worker {
            return service_worker.script_chan();
        } else {
            panic!("need to implement a sender for SharedWorker")
        }
    }

    pub fn dom_manipulation_task_source(&self) -> DOMManipulationTaskSource {
        DOMManipulationTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        FileReadingTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn networking_task_source(&self) -> NetworkingTaskSource {
        NetworkingTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn performance_timeline_task_source(&self) -> PerformanceTimelineTaskSource {
        PerformanceTimelineTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn port_message_queue(&self) -> PortMessageQueue {
        PortMessageQueue(self.script_chan(), self.pipeline_id())
    }

    pub fn timer_task_source(&self) -> TimerTaskSource {
        TimerTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn remote_event_task_source(&self) -> RemoteEventTaskSource {
        RemoteEventTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn websocket_task_source(&self) -> WebsocketTaskSource {
        WebsocketTaskSource(self.script_chan(), self.pipeline_id())
    }

    pub fn new_script_pair(&self) -> (Box<dyn ScriptChan + Send>, Box<dyn ScriptPort + Send>) {
        let dedicated = self.downcast::<DedicatedWorkerGlobalScope>();
        if let Some(dedicated) = dedicated {
            dedicated.new_script_pair()
        } else {
            panic!("need to implement a sender for SharedWorker/ServiceWorker")
        }
    }

    /// Process a single event as if it were the next event
    /// in the queue for this worker event-loop.
    /// Returns a boolean indicating whether further events should be processed.
    #[allow(unsafe_code)]
    pub fn process_event(&self, msg: CommonScriptMsg) -> bool {
        if self.is_closing() {
            return false;
        }
        match msg {
            CommonScriptMsg::Task(_, task, _, _) => task.run_box(),
            CommonScriptMsg::CollectReports(reports_chan) => {
                let cx = self.get_cx();
                let path_seg = format!("url({})", self.get_url());
                let reports = unsafe { get_reports(*cx, path_seg) };
                reports_chan.send(reports);
            },
        }
        true
    }

    pub fn close(&self) {
        self.closing.store(true, Ordering::SeqCst);
    }
}
