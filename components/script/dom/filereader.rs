/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileReaderBinding;
use dom::bindings::codegen::Bindings::FileReaderBinding::{FileReaderConstants, FileReaderMethods};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::InvalidState;
use dom::bindings::error::Error::{ Syntax, Abort, Timeout};
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary, Rootable};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::reflect_dom_object;
use dom::event::{Event, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::blob::Blob;
use dom::blob::BlobHelpers;
use dom::domexception::DOMException;
use dom::progressevent::ProgressEvent;
use encoding::all::UTF_8;
use encoding::types::{EncodingRef, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use script_task::{ScriptChan, ScriptMsg, Runnable, ScriptPort};
use std::cell::{Cell, RefCell};
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, Sender, Receiver};
use util::str::DOMString;
use util::task::spawn_named;

pub enum FileReaderFunction {
    ArrayBuffer,
    Text,
    DataUrl,
}

pub struct ReadData {
    pub bytes: Option<Vec<u8>>,
    pub blobtype: DOMString,
    pub label: Option<DOMString>,
    pub function: FileReaderFunction
}

impl ReadData {
    pub fn new(bytes: Option<Vec<u8>>,blobtype: DOMString, label: Option<DOMString>, function: FileReaderFunction) -> ReadData {
        ReadData {
            bytes: bytes,
            blobtype: blobtype,
            label: label,
            function: function
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[jstraceable]
pub struct GenerationId(u32);

pub type ReadConsumer = Box<AsyncResultTarget+Send>;

pub type FileReaderTask = Sender<ControlMsg>;

pub enum ControlMsg {
    Read(ReadData,ReadConsumer),
    Exit
}

struct FileReaderContext {
    fr: Trusted<FileReader>,
    gen_id: GenerationId
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
                fr.r().process_data_available(self.gen_id, payload);
            }

            fn reading_complete(&self, status: Result<DOMString, String>){
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

        let listener = box ReadingListener {
            context: context,
            script_chan: script_chan,
        };
        filereader_task.send(ControlMsg::Read(read_data, listener)).unwrap();
    }
}

impl<'a> FileReaderMethods for JSRef<'a, FileReader> {
    event_handler!(loadstart, GetOnloadstart, SetOnloadstart);
    event_handler!(progress, GetOnprogress, SetOnprogress);
    event_handler!(load, GetOnload, SetOnload);
    event_handler!(abort, GetOnabort, SetOnabort);
    event_handler!(error, GetOnerror, SetOnerror);
    event_handler!(loadend, GetOnloadend, SetOnloadend);

    fn ReadAsArrayBuffer(self,blob: JSRef<Blob>) -> ErrorResult {
        let global = self.global.root();
        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {//1. 
            return Err(InvalidState);
        }
        
        self.change_ready_state(FileReaderReadyState::Loading);//3. 

        let bytes = blob.read_out_buffer();
        let type_ = blob.read_out_type();


        let load_data = ReadData::new(bytes.clone(),type_,None,FileReaderFunction::ArrayBuffer);

        self.read(load_data,global.r())
    }

    fn ReadAsText(self,blob: JSRef<Blob>,label:Option<DOMString>) -> ErrorResult {
        let global = self.global.root();
        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {//1. ReadAsText
            return Err(InvalidState);
        }
        
        self.change_ready_state(FileReaderReadyState::Loading);//3. ReadAsText

        let bytes = blob.read_out_buffer();
        let type_ = blob.read_out_type();


        let load_data = ReadData::new(bytes.clone(),type_,label,FileReaderFunction::Text);

        self.read(load_data,global.r())
    }

    fn ReadAsDataURL(self,blob: JSRef<Blob>) -> ErrorResult {
        //println!("{}", "Run ReadAsDataURL");
        let global = self.global.root();
        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {//1.
            return Err(InvalidState);
        }

        self.change_ready_state(FileReaderReadyState::Loading);//3.

        let bytes = blob.read_out_buffer();
        let type_ = blob.read_out_type();

        let load_data = ReadData::new(bytes.clone(),type_,None,FileReaderFunction::DataUrl);

        self.read(load_data,global.r())
    }

    fn Abort(self) {
        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {
            self.change_ready_state(FileReaderReadyState::Done);
        }

        *self.result.borrow_mut() = None;

        self.terminate_ongoing_reading();
        
        self.dispatch_result_progress_event("abort".to_owned());
        self.dispatch_result_progress_event("loadend".to_owned());
    }

    fn GetError(self) -> Option<Temporary<DOMException>> {
        None
    }

    fn GetResult(self) -> Option<DOMString> {
        match self.ready_state.get() {
            FileReaderReadyState::Done | FileReaderReadyState::Loading => self.result.borrow().clone(),
            _ => None
        }
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
    fn process_result_complete(self, gen_id: GenerationId, status: Result<DOMString, String>);
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
        spawn_named("FileReaderManager".to_owned(), move || {
            FileReaderManager::new(setup_port).start();
        });
        setup_chan
    }

    fn read(self, read_data: ReadData, global: GlobalRef) -> ErrorResult {
        let fr = Trusted::new(global.get_cx(), self, global.script_chan());

        let context = Arc::new(Mutex::new(FileReaderContext {
            fr: fr,
            gen_id: self.generation_id.get()
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
            FileReaderProgress::Reading(_,_) =>{
                self.dispatch_result_progress_event("progress".to_owned());//7.
            },
            FileReaderProgress::Done(_,s) => {
                self.dispatch_result_progress_event("progress".to_owned());
                return_if_fetch_was_terminated!();
                self.change_ready_state(FileReaderReadyState::Done);//8.1.
                return_if_fetch_was_terminated!();
                *self.result.borrow_mut() = Some(s);
                self.dispatch_result_progress_event("load".to_owned());//8.3
                return_if_fetch_was_terminated!();
                if self.ready_state.get() as u16 != FileReaderReadyState::Loading as u16 {//8.4
                    self.dispatch_result_progress_event("loadend".to_owned());
                }
            },
            FileReaderProgress::Errored(_, e) => {
                let errormsg = match e {
                    Abort => "abort",
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
        self.ready_state.set(state);
    }

    fn process_start(self, gen_id: GenerationId) {
        self.process_partial_result(FileReaderProgress::Start(gen_id));        
    }

    fn process_data_available(self, gen_id: GenerationId, payload: DOMString) {
        self.process_partial_result(FileReaderProgress::Reading(gen_id, payload));
    }

    fn process_result_complete(self, gen_id: GenerationId, status: Result<DOMString, String>) {
        match status {
            Ok(s) => {
                self.process_partial_result(FileReaderProgress::Done(gen_id,s));
            },
            Err(_) => {
                self.process_partial_result(FileReaderProgress::Errored(gen_id, Syntax));
            }
        }
    }

    fn dispatch_result_progress_event(self, type_: DOMString) {
        self.dispatch_progress_event(type_, 0, None);
    }
}

struct FileReaderManager {
    filereader_client: Receiver<ControlMsg>,
}

impl FileReaderManager {
    fn new(filereader_client: Receiver<ControlMsg>) -> FileReaderManager {
        FileReaderManager {
            filereader_client: filereader_client,
        }
    }
}


impl FileReaderManager {
    fn start(&mut self) {
        loop {
            match self.filereader_client.recv().unwrap() {
              ControlMsg::Read(read_data, consumer) => {
                self.read(read_data, consumer);
                break
              }
              ControlMsg::Exit => {
                break
              }
            }
        }
    }

    fn read(&mut self, read_data: ReadData, consumer: ReadConsumer) {
        let progress = start_reading(consumer);
        progress.invoke(ResultAction::DataAvailable(DOMString::new()));
        match read_data.function {
            FileReaderFunction::Text => self.readText(read_data, progress),
            FileReaderFunction::DataUrl => self.readDataUrl(read_data, progress),
            _ => {
                //println!("Run read of FileReaderManager: {}", "Not Implemented Function");
                progress.invoke(ResultAction::ResultComplete(Err(DOMString::from_str("Not Implemented Function"))))
            }
        }
    }

    fn readDataUrl(&mut self, read_data: ReadData, progress: ReadConsumer) {
        //println!("Run readDataUrl of FileReaderManager: {}", "Not Implemented Function");
        progress.invoke(ResultAction::ResultComplete(Err(DOMString::from_str("Not Implemented Function"))))
    }

    fn readText(&mut self, read_data: ReadData, progress: ReadConsumer) {
        let encoding = if read_data.label.is_some() {
            encoding_from_whatwg_label(&read_data.label.unwrap())
        } else {
            Some(UTF_8 as EncodingRef)
        };

        let enc = match encoding {
            Some(code) => code,
            None => {
                progress.invoke(ResultAction::ResultComplete(Err(DOMString::from_str("Wrong Encoding"))));
                return;
            } 
        };
        let input = match read_data.bytes {
            Some(bytes) => bytes,
            None => {
                progress.invoke(ResultAction::ResultComplete(Ok(DOMString::new())));
                return;
            }
        };

        progress.invoke(ResultAction::DataAvailable(DOMString::new()));
        let (_, convert) = input.split_at(0);

        let output = enc.decode(convert, DecoderTrap::Strict);
        match output {
            Ok(s) => {
                progress.invoke(ResultAction::ResultComplete(Ok(s)));
            },
            Err(_) => progress.invoke(ResultAction::ResultComplete(Err(DOMString::from_str("Decoding failed"))))
        };
    }
}

/// For use by loaders in responding to a Load message.
fn start_reading(start_chan: ReadConsumer)  -> ReadConsumer {
    start_reading_opt(start_chan).ok().unwrap()
}

/// For use by loaders in responding to a Load message.
fn start_reading_opt(start_chan: ReadConsumer) -> Result<ReadConsumer, ()> {
    start_chan.invoke(ResultAction::StartReading);
    Ok(start_chan)
}

pub enum ResultAction {
    /// Invoke headers_available
    StartReading,
    /// Invoke data_available
    DataAvailable(DOMString),
    /// Invoke response_complete
    ResultComplete(Result<DOMString, String>)
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
    fn invoke(&self, action: ResultAction);
}

pub trait AsyncReadingListener {
    fn start_reading(&self);
    fn data_available(&self, payload: DOMString);
    fn reading_complete(&self, status: Result<DOMString, String>);
}

pub struct ReadingListener<T: AsyncReadingListener + Send + 'static> {
    pub context: Arc<Mutex<T>>,
    pub script_chan: Box<ScriptChan+Send>,
}

impl<T: AsyncReadingListener + PreInvoke + Send + 'static> AsyncResultTarget for ReadingListener<T> {
    fn invoke(&self, action: ResultAction) {
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

#[derive(Clone)]
pub enum FileReaderProgress {
    Start(GenerationId),
    Reading(GenerationId, DOMString),
    Done(GenerationId, DOMString),
    Errored(GenerationId, Error),
}

impl FileReaderProgress {
    fn generation_id(&self) -> GenerationId {
        match *self {
            FileReaderProgress::Reading(id, _) |
            FileReaderProgress::Start(id) |
            FileReaderProgress::Done(id, _) |
            FileReaderProgress::Errored(id, _) => id
        }
    }
}
