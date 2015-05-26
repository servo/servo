//use dom::bindings::codegen::UnionTypes;
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
use dom::domexception::DOMException;
use script_task::Runnable;
use script_task::ScriptMsg;
use std::cell::{Cell, RefCell};
use util::str::DOMString;

pub struct GenerationId(u32);

struct FileReaderContext {
    fr: Trusted<FileReader>,
    blob: Trusted<Blob>,
    gen_id: GenerationId,
    buf: RefCell<Vec<u8>>,
    sync_status: RefCell<Option<ErrorResult>>,
}

#[derive(Clone)]
pub enum FileReaderProgress {
    /// Partial progress (after receiving headers), containing portion of the response
    Loading(GenerationId),
    /// Loading is done
    Done(GenerationId),
    /// There was an error (only Abort, Timeout or Network is used)
    Errored(GenerationId, Error),
}
impl FileReaderProgress {
    fn generation_id(&self) -> GenerationId {
        match *self {
            FileReaderProgress::Loading(id) |
            FileReaderProgress::Done(id) |
            FileReaderProgress::Errored(id, _) => id
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
    result: RefCell<Option<DOMString>>
    //result: Option<UnionTypes::StringOrArrayBuffer> 
}

impl FileReader {
    pub fn new_inherited(global: GlobalRef, state: FileReaderReadyState) -> FileReader {
        FileReader { 
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::FileReader),//?
            global: GlobalField::from_rooted(&global),
            ready_state: Cell::new(state),
            error: RefCell::new(None),
            result: RefCell::new(None),
        }
    }

    pub fn new(global: GlobalRef, state: FileReaderReadyState) -> Temporary<FileReader> {
        reflect_dom_object(box FileReader::new_inherited( global,state),
                           global, FileReaderBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Temporary<FileReader>> {
        Ok(FileReader::new(global,FileReaderReadyState::Empty))
    }

    fn initiate_async_read(context: Arc<Mutex<FileReaderContext>>,
                          script_chan: Box<ScriptChan+Send>,
                          resource_task: ResourceTask) {        
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

    fn ReadAsText(self,blob: JSRef<Blob>,label:Option<DOMString>){
        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {//1. ReadAsText
            //throw DOMException
        }


        self.ready_state.set(FileReaderReadyState::Loading);//3. ReadAsText
/*
        let global_root = self.global.root();
        let addr: Trusted<FileReader> = Trusted::new(global_root.r().get_cx(), self, global_root.r().script_chan().clone());
        let open_task = box FileReaderTaskHandler::new(addr.clone(), FileReaderTask::Open, blob.bytes);
        global_root.r().script_chan().send(ScriptMsg::RunnableMsg(open_task)).unwrap();//4. ReadAsText
*/

    }

    fn ReadAsDataURL(self,blob: JSRef<Blob>){
        
    }

    fn Abort(self){
        let global = self.global.root();

        if self.ready_state.get() as u16 == FileReaderReadyState::Loading as u16 {
            self.ready_state.set(FileReaderReadyState::Done);
        }

        *self.result.borrow_mut() = None;

        //end tasks ?

        //terminate reading alg
        
        self.dispatch_response_progress_event("abort".to_owned(),0,None);
        self.dispatch_response_progress_event("abort".to_owned(),0,None);
    }

    fn GetError(self) -> Option<Temporary<DOMException>>{
        self.error.borrow()
    }

    fn GetResult(self) -> Option<DOMString>{
        self.result.borrow()
    }

    fn ReadyState(self) -> u16{
        self.ready_state.get() as u16
    }

}

trait PrivateFileReaderHelpers {
    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>);
}

impl<'a> PrivateFileReaderHelpers for JSRef<'a, FileReader> {

    fn dispatch_progress_event(self, upload: bool, type_: DOMString, loaded: u64, total: Option<u64>) {
        let global = self.global.root();
        let upload_target = self.upload.root();
        let progressevent = ProgressEvent::new(global.r(),
                                               type_, false, false,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0)).root();
        let target: JSRef<EventTarget> = if upload {
            EventTargetCast::from_ref(upload_target.r())
        } else {
            EventTargetCast::from_ref(self)
        };
        let event: JSRef<Event> = EventCast::from_ref(progressevent.r());
        event.fire(target);
    }
}


pub enum FileReaderTask {
    Open,
    Close,
}

pub struct FileReaderTaskHandler {
    addr: Trusted<FileReader>,
    task: FileReaderTask,
    bytes: Option<Vec<u8>>
//    blob: JSRef<Blob>
}

impl FileReaderTaskHandler {
    pub fn new(addr: Trusted<FileReader>, task: FileReaderTask,bytes: Option<Vec<u8>>) -> FileReaderTaskHandler {
        FileReaderTaskHandler {
            addr: addr,
            task: task,
            bytes: bytes
//            blob: blob
        }
    }

    fn dispatch_open(&self) {
        let fr = self.addr.to_temporary().root(); //Get root
        let fr = fr.r(); //Get filereader reference
        let global = fr.global.root();

        let event = Event::new(global.r(),//6.ReadAsText
            "loadstart".to_owned(),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(fr);
        event.r().fire(target);

        let event = Event::new(global.r(),//7.ReadAsText
            "progress".to_owned(),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(fr);
        event.r().fire(target);


    }

    fn dispatch_close(&self) {
        let fr = self.addr.to_temporary().root(); //Get root
        let fr = fr.r(); //Get filereader reference
        let global = fr.global.root();

    }
}

impl Runnable for FileReaderTaskHandler {
    fn handler(self: Box<FileReaderTaskHandler>) {
        match self.task {
            FileReaderTask::Open => {
                self.dispatch_open();
            }
            FileReaderTask::Close => {
                self.dispatch_close();
            }
        }
    }
}

