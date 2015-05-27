/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileReaderBinding;
use dom::bindings::codegen::Bindings::FileReaderBinding::{FileReaderConstants, FileReaderMethods};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::{InvalidState, InvalidAccess};
use dom::bindings::error::Error::{Network, Syntax, Security, Abort, Timeout};
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, Temporary, Rootable};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::blob::Blob;
use dom::blob::BlobHelpers;
use dom::domexception::DOMException;
use dom::progressevent::ProgressEvent;
use script_task::{ScriptChan, ScriptMsg, Runnable, ScriptPort};
use std::cell::{Cell, RefCell};
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, Sender, TryRecvError, Receiver};
use util::str::DOMString;
use util::task::spawn_named;

pub struct ReadData {
    pub bytes: Option<Vec<u8>>,
}

impl ReadData {
    pub fn new(bytes: Option<Vec<u8>>) -> ReadData {
        ReadData {
            bytes: bytes
        }
    }
}


struct FileReaderManager {
    filereader_client: Receiver<ControlMsg>,
    filereader_task: Sender<ControlMsg>
}

impl FileReaderManager {
    fn new(filereader_client: Receiver<ControlMsg>,
           filereader_task: Sender<ControlMsg>) -> FileReaderManager {
        FileReaderManager {
            filereader_client: filereader_client,
            filereader_task: filereader_task,
        }
    }
}


impl FileReaderManager {
    fn start(&mut self) {
        loop {
            match self.filereader_client.recv().unwrap() {
              ControlMsg::Read(load_data, consumer) => {
                self.read(load_data, consumer)
              }
              ControlMsg::Exit => {
                break
              }
            }
        }
    }

    fn read(&mut self, mut read_data: ReadData, consumer: ReadConsumer) {
        let progress = start_reading(consumer);
        progress.send(ProgressMsg::Payload(DOMString::new())).unwrap();
        progress.send(ProgressMsg::Payload(DOMString::new())).unwrap();
    }
}

/// For use by loaders in responding to a Load message.
fn start_reading(start_chan: ReadConsumer)  -> ProgressSender {
    start_reading_opt(start_chan).ok().unwrap()
}

/// For use by loaders in responding to a Load message.
fn start_reading_opt(start_chan: ReadConsumer) -> Result<ProgressSender, ()> {
    match start_chan {
        ReadConsumer::Channel(start_chan) => {
            Err(())
        }
        ReadConsumer::Listener(target) => {
            target.invoke_with_listener(ResultAction::StartReading);
            Ok(ProgressSender::Listener(target))
        }
    }
}


#[derive(PartialEq, Clone, Copy)]
#[jstraceable]
pub struct GenerationId(u32);

struct FileReaderContext {
    fr: Trusted<FileReader>,
    gen_id: GenerationId,
    buf: RefCell<DOMString>,
    sync_status: RefCell<Option<ErrorResult>>,
}

#[derive(Clone)]
pub enum FileReaderProgress {
    Start(GenerationId),
    Reading(GenerationId, DOMString),
    Done(GenerationId),
    Errored(GenerationId, Error),
}

impl FileReaderProgress {
    fn generation_id(&self) -> GenerationId {
        match *self {
            FileReaderProgress::Reading(id, _) |
            FileReaderProgress::Start(id) |
            FileReaderProgress::Done(id) |
            FileReaderProgress::Errored(id, _) => id
        }
    }
}


#[derive(PartialEq,Debug)]
pub enum ProgressMsg {
    Payload(DOMString),
    Done(Result<(), String>)
}

pub enum ProgressSender {
    Channel(Sender<ProgressMsg>),
    Listener(Box<AsyncResultTarget>),
}

impl ProgressSender {
    //XXXjdm return actual error
    pub fn send(&self, msg: ProgressMsg) -> Result<(), ()> {
        match *self {
            ProgressSender::Channel(ref c) => c.send(msg).map_err(|_| ()),
            ProgressSender::Listener(ref b) => {
                let action = match msg {
                    ProgressMsg::Payload(buf) => ResultAction::DataAvailable(buf),
                    ProgressMsg::Done(status) => ResultAction::ResultComplete(status),
                };
                b.invoke_with_listener(action);
                Ok(())
            }
        }
    }
}

pub struct ReadResult {
    /// Port for reading data.
    pub progress_port: Receiver<ProgressMsg>,
}

pub enum ReadConsumer {
    Channel(Sender<ReadResult>),
    Listener(Box<AsyncResultTarget + Send>),
}

pub type FileReaderTask = Sender<ControlMsg>;

pub enum ControlMsg {
    Read(ReadData,ReadConsumer),
    Exit
}

pub enum ResultAction {
    /// Invoke headers_available
    StartReading,
    /// Invoke data_available
    DataAvailable(DOMString),
    /// Invoke response_complete
    ResultComplete(Result<(), String>)
}

impl ResultAction {
    /// Execute the default action on a provided listener.
    pub fn process(self, listener: &AsyncReadingListener) {
        match self {
            ResultAction::StartReading => listener.start_reading(),
            ResultAction::DataAvailable(d) => listener.data_available(d),
            ResultAction::ResultComplete(r) => listener.reading_complete(r),
        }
    }
}
pub trait AsyncResultTarget {
    fn invoke_with_listener(&self, action: ResultAction);
}

pub trait AsyncReadingListener {
    fn start_reading(&self);
    fn data_available(&self, payload: DOMString);
    fn reading_complete(&self, status: Result<(), String>);
}

pub struct ReadingListener<T: AsyncReadingListener + Send + 'static> {
    pub context: Arc<Mutex<T>>,
    pub script_chan: Box<ScriptChan+Send>,
}

impl<T: AsyncReadingListener + PreInvoke + Send + 'static> AsyncResultTarget for ReadingListener<T> {
    fn invoke_with_listener(&self, action: ResultAction) {
        self.script_chan.send(ScriptMsg::RunnableMsg(box ListenerRunnable {
            context: self.context.clone(),
            action: action,
        })).unwrap();
    }
}

pub trait PreInvoke {
    fn should_invoke(&self) -> bool {
        true
    }
}
/// A runnable for moving the async network events between threads.
struct ListenerRunnable<T: AsyncReadingListener + PreInvoke + Send> {
    context: Arc<Mutex<T>>,
    action: ResultAction,
}

impl<T: AsyncReadingListener + PreInvoke + Send> Runnable for ListenerRunnable<T> {
    fn handler(self: Box<ListenerRunnable<T>>) {
        let this = *self;
        let context = this.context.lock().unwrap();
        if context.should_invoke() {
            this.action.process(&*context);
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
#[jstraceable]
pub enum FileReaderReadyState {
    Empty = FileReaderConstants::EMPTY,
    Loading = FileReaderConstants::LOADING,
    Done = FileReaderConstants::DONE,
}

#[dom_struct]
pub struct FileReader {
    eventtarget: EventTarget,
    global: GlobalField,
    ready_state: Cell<FileReaderReadyState>,
    error: RefCell<Option<DOMException>>,
    result: RefCell<Option<DOMString>>,
    generation_id: Cell<GenerationId>,
    filereader_task: RefCell<Option<FileReaderTask>>,
    abort_target: RefCell<Option<Box<ScriptChan+Send>>>,
    //result: Option<UnionTypes::StringOrArrayBuffer> 
}

impl FileReader {
    pub fn new_inherited(global: GlobalRef) -> FileReader {
        FileReader { 
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::FileReader),//?
            global: GlobalField::from_rooted(&global),
            ready_state: Cell::new(FileReaderReadyState::Empty),
            error: RefCell::new(None),
            result: RefCell::new(None),
            generation_id: Cell::new(GenerationId(0)),
            filereader_task: RefCell::new(None),
            abort_target: RefCell::new(None),
        }
    }

    pub fn new(global: GlobalRef) -> Temporary<FileReader> {
        reflect_dom_object(box FileReader::new_inherited( global),
                           global, FileReaderBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Temporary<FileReader>> {
        Ok(FileReader::new(global))
    }

    fn initiate_async_fr(context: Arc<Mutex<FileReaderContext>>, script_chan: Box<ScriptChan+Send>, filereader_task: FileReaderTask, read_data: ReadData) {
        impl AsyncReadingListener for FileReaderContext {
            fn data_available(&self, payload: DOMString){
                let fr = self.fr.to_temporary().root();
                fr.r().process_data_available(self.gen_id, self.buf.borrow().clone());
            }

            fn reading_complete(&self, status: Result<(), String>){
                let fr = self.fr.to_temporary().root();
                fr.r().process_result_complete(self.gen_id, status);
            }
            fn start_reading(&self){
                let fr = self.fr.to_temporary().root();
                fr.r().process_start(self.gen_id);
            }
        }

        impl PreInvoke for FileReaderContext {
            fn should_invoke(&self) -> bool {
                let fr = self.fr.to_temporary().root();
                fr.r().generation_id.get() == self.gen_id
            }
        }

        let listener = box ReadingListener {//replace
            context: context,
            script_chan: script_chan,
        };
        filereader_task.send(ControlMsg::Read(read_data, ReadConsumer::Listener(listener))).unwrap();
    }
}

impl<'a> FileReaderMethods for JSRef<'a, FileReader> {
    event_handler!(loadstart, GetOnloadstart, SetOnloadstart);
    event_handler!(progress, GetOnprogress, SetOnprogress);
    event_handler!(load, GetOnload, SetOnload);
    event_handler!(abort, GetOnabort, SetOnabort);
    event_handler!(error, GetOnerror, SetOnerror);
    event_handler!(loadend, GetOnloadend, SetOnloadend);

    /*fn ReadAsArrayBuffer(self,blob: JSRef<Blob>){
        
    }*/

    fn ReadAsText(self,blob: JSRef<Blob>,label:Option<DOMString>) {
        let global = self.global.root();
        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {//1. ReadAsText
            //throw DOMException
        }


        self.ready_state.set(FileReaderReadyState::Loading);//3. ReadAsText

        let bytes = blob.read_out_buffer();

        let mut load_data = ReadData::new(bytes.clone());

        self.read(load_data,global.r());

    }

    fn ReadAsDataURL(self,blob: JSRef<Blob>) {
        
    }

    fn Abort(self) {
        let global = self.global.root();

        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {
            self.ready_state.set(FileReaderReadyState::Done);
        }

        *self.result.borrow_mut() = None;

        //end tasks ?

        //terminate reading alg
        self.terminate_ongoing_reading();
        
        self.dispatch_result_progress_event("abort".to_owned());
        self.dispatch_result_progress_event("loadend".to_owned());
    }

    fn GetError(self) -> Option<Temporary<DOMException>> {
        None
        //self.error.borrow()
    }

    fn GetResult(self) -> Option<DOMString> {
        None
        //self.result.borrow()
    }

    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }
}

trait PrivateFileReaderHelpers {
    fn dispatch_progress_event(self, type_: DOMString, loaded: u64, total: Option<u64>);
    fn terminate_ongoing_reading(self);
    fn read(self, read_data: ReadData,  global: GlobalRef) -> ErrorResult;
    fn process_data_available(self, gen_id: GenerationId, payload: DOMString);
    fn process_start(self, gen_id: GenerationId);
    fn process_result_complete(self, gen_id: GenerationId, status: Result<(), String>) -> ErrorResult;
    fn process_partial_result(self, progress: FileReaderProgress);
    fn dispatch_result_progress_event(self, type_:DOMString);
    fn change_ready_state(self, state: FileReaderReadyState);
    fn new_filereader_task(self) -> FileReaderTask;
}

impl<'a> PrivateFileReaderHelpers for JSRef<'a, FileReader> {

    fn dispatch_progress_event(self, type_: DOMString, loaded: u64, total: Option<u64>) {
        let global = self.global.root();
        let progressevent = ProgressEvent::new(global.r(),
                                               type_, false, false,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0)).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        let event: JSRef<Event> = EventCast::from_ref(progressevent.r());
        event.fire(target);
    }

    fn terminate_ongoing_reading(self) {
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
    }
    
    fn new_filereader_task(self) -> FileReaderTask {
        let (setup_chan, setup_port) = channel();
        let setup_chan_clone = setup_chan.clone();
        spawn_named("FileReaderManager".to_owned(), move || {
            FileReaderManager::new(setup_port, setup_chan_clone).start();
        });
        setup_chan
    }

    fn read(self, read_data: ReadData, global: GlobalRef) -> ErrorResult {

        let fr = Trusted::new(global.get_cx(), self, global.script_chan());

        let context = Arc::new(Mutex::new(FileReaderContext {
            fr: fr,
            gen_id: self.generation_id.get(),
            buf: RefCell::new(DOMString::new()),
            sync_status: RefCell::new(None),
        }));
        
        let script_chan = global.script_chan();

        *self.abort_target.borrow_mut() = Some(script_chan.clone());

        let filereader_task = self.new_filereader_task();
        *self.filereader_task.borrow_mut() = Some(filereader_task.clone());

        FileReader::initiate_async_fr(context.clone(), script_chan, filereader_task, read_data);
        Ok(())
    }

    fn process_partial_result(self, progress: FileReaderProgress) {
        let msg_id = progress.generation_id();

        // Aborts processing if abort() or open() was called
        // (including from one of the event handlers called below)
        macro_rules! return_if_fetch_was_terminated(
            () => (
                if msg_id != self.generation_id.get() {
                    return
                }
            );
        );

        // Ignore message if it belongs to a terminated fetch
        return_if_fetch_was_terminated!();
        match progress {
            FileReaderProgress::Start(_)=>{
                self.dispatch_result_progress_event("loadstart".to_owned());//6.
            },
            FileReaderProgress::Reading(_,partial) =>{
                self.dispatch_result_progress_event("progress".to_owned());//7.
            },
            FileReaderProgress::Done(_) => {
                self.dispatch_result_progress_event("progress".to_owned());
                return_if_fetch_was_terminated!();
                self.change_ready_state(FileReaderReadyState::Done);//8.1.
                return_if_fetch_was_terminated!();
                self.dispatch_result_progress_event("load".to_owned());//8.3
                return_if_fetch_was_terminated!();
                if(self.ready_state.get() as u16 != FileReaderReadyState::Loading as u16){//8.4
                    self.dispatch_result_progress_event("loadend".to_owned());
                }
            },
            FileReaderProgress::Errored(_, e) => {

                let errormsg = match e {
                    Abort => "abort",
                    Timeout => "timeout",
                    _ => "error",
                };
                self.dispatch_result_progress_event("progress".to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_result_progress_event(errormsg.to_owned());
                return_if_fetch_was_terminated!();
                self.dispatch_result_progress_event("loadend".to_owned());
            }
        }
    }

    fn change_ready_state(self, state: FileReaderReadyState) {

    }

    fn process_start(self, gen_id: GenerationId) {
        self.process_partial_result(FileReaderProgress::Start(gen_id));        
    }

    fn process_data_available(self, gen_id: GenerationId, payload: DOMString) {
        self.process_partial_result(FileReaderProgress::Reading(gen_id, payload));
    }

    fn process_result_complete(self, gen_id: GenerationId, status: Result<(), String>) -> ErrorResult {
        match status {
            Ok(()) => {
                self.process_partial_result(FileReaderProgress::Done(gen_id));
                Ok(())
            },
            Err(_) => {
                self.process_partial_result(FileReaderProgress::Errored(gen_id, Syntax));
                Err(Syntax)
            }
        }
    }

    fn dispatch_result_progress_event(self, type_: DOMString) {
        self.dispatch_progress_event(type_, 0, None);
    }
}
