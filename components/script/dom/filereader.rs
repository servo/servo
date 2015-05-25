//use dom::bindings::codegen::UnionTypes;
use dom::bindings::codegen::Bindings::FileReaderBinding;
use dom::bindings::codegen::Bindings::FileReaderBinding::{FileReaderConstants, FileReaderMethods};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::Fallible;
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
    fn new_inherited(global: GlobalRef, state: FileReaderReadyState) -> FileReader {
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

        let global_root = self.global.root();
        let addr: Trusted<FileReader> = Trusted::new(global_root.r().get_cx(), self, global_root.r().script_chan().clone());
        let open_task = box FileReaderTaskHandler::new(addr.clone(), FileReaderTask::Open);
        global_root.r().script_chan().send(ScriptMsg::RunnableMsg(open_task)).unwrap();//4. ReadAsText


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

        let event = Event::new(global.r(), "abort".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        event.r().fire(target);

        let event = Event::new(global.r(), "loadend".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        event.r().fire(target);

    }

    fn GetError(self) -> Option<Temporary<DOMException>>{
        return None;
    }

    fn GetResult(self) -> Option<DOMString>{
        return None;
    }

    fn ReadyState(self) -> u16{
        return self.ready_state.get() as u16;
    }

}


pub enum FileReaderTask {
    Open,
    Close,
}

pub struct FileReaderTaskHandler {
    addr: Trusted<FileReader>,
    task: FileReaderTask,
//    blob: Blob
}

impl FileReaderTaskHandler {
    pub fn new(addr: Trusted<FileReader>, task: FileReaderTask) -> FileReaderTaskHandler {
        FileReaderTaskHandler {
            addr: addr,
            task: task
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

