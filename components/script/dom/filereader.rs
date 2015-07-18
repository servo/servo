/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileReaderBinding;
use dom::bindings::codegen::Bindings::FileReaderBinding::{FileReaderConstants, FileReaderMethods};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::InvalidState;
use dom::bindings::error::Error::Abort;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{Root, JS, MutNullableHeap};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{reflect_dom_object, Reflectable};
use dom::event::EventHelpers;
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
use util::str::DOMString;

pub type TrustedFileReader = Trusted<FileReader>;

pub struct ReadData {
    pub bytes: Option<Vec<u8>>,
    pub blobtype: DOMString,
    pub label: Option<DOMString>
}

impl ReadData {
    pub fn new(bytes: Option<Vec<u8>>, blobtype: DOMString,
               label: Option<DOMString>) -> ReadData {
        ReadData {
            bytes: bytes,
            blobtype: blobtype,
            label: label
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
#[derive_JSTraceable]
pub struct GenerationId(u32);

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[derive_JSTraceable]
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
    error: MutNullableHeap<JS<DOMException>>,
    result: RefCell<Option<DOMString>>,
    generation_id: Cell<GenerationId>,
}

impl FileReader {
    pub fn new_inherited( global: GlobalRef) -> FileReader {
        FileReader {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::FileReader),//?
            global: GlobalField::from_rooted(&global),
            ready_state: Cell::new(FileReaderReadyState::Empty),
            error: MutNullableHeap::new(None),
            result: RefCell::new(None),
            generation_id: Cell::new(GenerationId(0)),
        }
    }

    pub fn new(global: GlobalRef) -> Root<FileReader> {
        reflect_dom_object( box FileReader::new_inherited( global),
                           global, FileReaderBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<FileReader>> {
        Ok(FileReader::new(global))
    }

    pub fn process_read_error(filereader: TrustedFileReader, gen_id: GenerationId, error: Error) {
        let fr = filereader.root();
        fr.process_partial_result(FileReaderProgress::Error(gen_id, error));
    }

    pub fn process_read_data(filereader: TrustedFileReader, gen_id: GenerationId, payload: DOMString) {
        let fr = filereader.root();
        fr.process_partial_result(FileReaderProgress::Reading(gen_id, payload));
    }

    pub fn process_read(filereader: TrustedFileReader, gen_id: GenerationId) {
        let fr = filereader.root();
        fr.process_partial_result(FileReaderProgress::Start(gen_id));
    }

    pub fn process_read_eof(filereader: TrustedFileReader, gen_id: GenerationId, data: DOMString) {
        let fr = filereader.root();
        fr.process_partial_result(FileReaderProgress::EOF(gen_id, data));
    }

    pub fn handle_read(filereader: TrustedFileReader, gen_id: GenerationId, read_data: ReadData) {
        // STEP 4 https://w3c.github.io/FileAPI/#dfn-readAsText
        FileReader::process_read(filereader.clone(),gen_id);

        // STEP 5 https://w3c.github.io/FileAPI/#dfn-readAsText
        FileReader::process_read_data(filereader.clone(), gen_id, DOMString::new());
        let encoding = match read_data.label {
            Some(e) => encoding_from_whatwg_label(&e),
            None => Some(UTF_8 as EncodingRef)
        };

        let enc = match encoding {
            Some(code) => code,
            None => {
                FileReader::process_read_error(filereader.clone(), gen_id, Error::NotSupported);
                return;
            }
        };
        let input = match read_data.bytes {
            Some(bytes) => bytes,
            None => {
                FileReader::process_read_eof(filereader.clone(), gen_id, DOMString::new());
                return;
            }
        };
        //STEP 5
        FileReader::process_read_data(filereader.clone(), gen_id, DOMString::new());
        let (_, convert) = input.split_at(0);

        let output = enc.decode(convert, DecoderTrap::Strict);
        match output {
            Ok(s) => {
                FileReader::process_read_data(filereader.clone(), gen_id, s.clone());
                FileReader::process_read_eof(filereader.clone(), gen_id, s.clone());
            },
            Err(_) => FileReader::process_read_error(filereader.clone(), gen_id, Error::InvalidCharacter)
        };
    }
}

impl<'a> FileReaderMethods for &'a FileReader {
    event_handler!(loadstart, GetOnloadstart, SetOnloadstart);
    event_handler!(progress, GetOnprogress, SetOnprogress);
    event_handler!(load, GetOnload, SetOnload);
    event_handler!(abort, GetOnabort, SetOnabort);
    event_handler!(error, GetOnerror, SetOnerror);
    event_handler!(loadend, GetOnloadend, SetOnloadend);

    //TODO https://w3c.github.io/FileAPI/#dfn-readAsArrayBuffer
    //TODO https://w3c.github.io/FileAPI/#dfn-readAsDataURL

    //https://w3c.github.io/FileAPI/#dfn-readAsText
    fn ReadAsText(self,blob: &Blob,label:Option<DOMString>) -> ErrorResult {
        let global = self.global.root();
        //STEP 1
        if self.ready_state.get() == FileReaderReadyState::Loading {
            return Err(InvalidState);
        }
        //FIXME STEP 2 if isClosed implemented in Blob

        //STEP 3
        self.change_ready_state(FileReaderReadyState::Loading);

        let bytes = blob.read_out_buffer();
        let type_ = blob.read_out_type();

        let bytes_data = match bytes.recv() {
            Ok(e) => e,
            Err(_) => {
                return Err(InvalidState);
            }
        };
        let load_data = ReadData::new(bytes_data,type_,label);

        self.read(load_data,global.r())
    }


    //https://w3c.github.io/FileAPI/#dfn-abort
    fn Abort(self) {
        // STEP 2
        if self.ready_state.get() == FileReaderReadyState::Loading {
            self.change_ready_state(FileReaderReadyState::Done);
        }
        // STEP 1 & 3
        *self.result.borrow_mut() = None;

        self.terminate_ongoing_reading();
        // STEP 5 & 6
        self.dispatch_result_progress_event("abort".to_owned());
        self.dispatch_result_progress_event("loadend".to_owned());
    }

    fn GetError(self) -> Option<Root<DOMException>> {
        //FIXME Return the current error state
        //self.error.get().root()
        None
    }

    fn GetResult(self) -> Option<DOMString> {
        self.result.borrow().clone()
    }

    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }
}

trait PrivateFileReaderHelpers {
    fn dispatch_progress_event(self, type_: DOMString, loaded: u64, total: Option<u64>);
    fn terminate_ongoing_reading(self);
    fn read(self, read_data: ReadData,  global: GlobalRef) -> ErrorResult;
    fn process_partial_result(self, progress: FileReaderProgress);
    fn dispatch_result_progress_event(self, type_:DOMString);
    fn change_ready_state(self, state: FileReaderReadyState);
}

impl<'a> PrivateFileReaderHelpers for &'a FileReader {

    fn dispatch_progress_event(self, type_: DOMString, loaded: u64, total: Option<u64>) {
        let global = self.global.root();
        let progressevent = ProgressEvent::new(global.r(),
                                               type_, false, false,
                                               total.is_some(), loaded,
                                               total.unwrap_or(0));
        let target = EventTargetCast::from_ref(self);
        let event = EventCast::from_ref(progressevent.r());
        event.fire(target);
    }

    fn terminate_ongoing_reading(self) {
        // STEP 4
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
    }

    fn read(self, read_data: ReadData, global: GlobalRef) -> ErrorResult {

        let fr = Trusted::new(global.get_cx(), self, global.script_chan());
        let gen_id = self.generation_id.get();

        let task = box FileReaderHandler::new(gen_id, read_data, fr);
        global.script_chan().send(ScriptMsg::RunnableMsg(task)).unwrap();
        Ok(())
    }

    fn process_partial_result(self, progress: FileReaderProgress) {
        let msg_id = progress.generation_id();

        macro_rules! return_if_reading_was_terminated(
            () => (
                if msg_id != self.generation_id.get() {
                    return
                }
            );
        );

        return_if_reading_was_terminated!();
        match progress {
            FileReaderProgress::Start(_)=>{
                // STEP 6 https://w3c.github.io/FileAPI/#dfn-readAsText
                self.dispatch_result_progress_event("loadstart".to_owned());
            },
            FileReaderProgress::Reading(_,_) =>{
                // STEP 7 https://w3c.github.io/FileAPI/#dfn-readAsText
                self.dispatch_result_progress_event("progress".to_owned());
            },
            FileReaderProgress::EOF(_,s) => {
                self.dispatch_result_progress_event("progress".to_owned());
                return_if_reading_was_terminated!();
                // STEP 8.1 https://w3c.github.io/FileAPI/#dfn-readAsText
                self.change_ready_state(FileReaderReadyState::Done);
                return_if_reading_was_terminated!();
                // STEP 8.2 https://w3c.github.io/FileAPI/#dfn-readAsText
                *self.result.borrow_mut() = Some(s);
                return_if_reading_was_terminated!();
                // STEP 8.3 https://w3c.github.io/FileAPI/#dfn-readAsText
                self.dispatch_result_progress_event("load".to_owned());
                return_if_reading_was_terminated!();
                // STEP 8.4 https://w3c.github.io/FileAPI/#dfn-readAsText
                if self.ready_state.get() as u16 != FileReaderReadyState::Loading as u16 {
                    self.dispatch_result_progress_event("loadend".to_owned());
                }
                // STEP 9 https://w3c.github.io/FileAPI/#dfn-readAsText
            },
            FileReaderProgress::Error(_, e) => {
                let errormsg = match e {
                    Abort => "abort",
                    _ => "error",
                };
                return_if_reading_was_terminated!();
                // STEP 1
                self.change_ready_state(FileReaderReadyState::Done);
                return_if_reading_was_terminated!();
                *self.result.borrow_mut() = None;
                //FIXME set error attribute
                return_if_reading_was_terminated!();
                self.dispatch_result_progress_event(errormsg.to_owned());
                return_if_reading_was_terminated!();
                // STEP 3
                self.dispatch_result_progress_event("loadend".to_owned());
                // STEP 4
            }
        }
    }

    fn change_ready_state(self, state: FileReaderReadyState) {
        self.ready_state.set(state);
    }

    fn dispatch_result_progress_event(self, type_: DOMString) {
        self.dispatch_progress_event(type_, 0, None);
    }
}

pub struct FileReaderHandler {
    gen_id: GenerationId,
    read_data: ReadData,
    filereader: TrustedFileReader,
}

impl FileReaderHandler {
    pub fn new(gen_id: GenerationId, read_data: ReadData, filereader: TrustedFileReader) -> FileReaderHandler {
        FileReaderHandler {
            gen_id: gen_id,
            read_data: read_data,
            filereader: filereader,
        }
    }

}

impl Runnable for FileReaderHandler {
    fn handler(self: Box<FileReaderHandler>) {
        let this = *self;
        FileReader::handle_read(this.filereader,this.gen_id,this.read_data);
    }
}

#[derive(Clone)]
pub enum FileReaderProgress {
    Start(GenerationId),
    Reading(GenerationId, DOMString),
    EOF(GenerationId, DOMString),
    Error(GenerationId, Error),
}

impl FileReaderProgress {
    fn generation_id(&self) -> GenerationId {
        match *self {
            FileReaderProgress::Start(id) |
            FileReaderProgress::Reading(id, _) |
            FileReaderProgress::EOF(id, _) |
            FileReaderProgress::Error(id, _) => id
        }
    }
}
