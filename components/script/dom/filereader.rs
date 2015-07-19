/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileReaderBinding;
use dom::bindings::codegen::Bindings::FileReaderBinding::{FileReaderConstants, FileReaderMethods};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::error::Error::InvalidState;
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
use std::sync::mpsc::{channel, Receiver};
use util::str::DOMString;
use util::task::spawn_named;

pub type TrustedFileReader = Trusted<FileReader>;

pub struct ReadData {
    pub bytes: Receiver<Option<Vec<u8>>>,
    pub blobtype: DOMString,
    pub label: Option<DOMString>
}

impl ReadData {
    pub fn new(bytes: Receiver<Option<Vec<u8>>>, blobtype: DOMString,
               label: Option<DOMString>) -> ReadData {
        ReadData {
            bytes: bytes,
            blobtype: blobtype,
            label: label
        }
    }
}

#[derive(PartialEq, Clone, Copy, JSTraceable)]
pub struct GenerationId(u32);

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, JSTraceable)]
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
    pub fn new_inherited(global: GlobalRef) -> FileReader {
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
        reflect_dom_object(box FileReader::new_inherited(global),
                           global, FileReaderBinding::Wrap)
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<FileReader>> {
        Ok(FileReader::new(global))
    }

    //https://w3c.github.io/FileAPI/#dfn-error-steps
    pub fn process_read_error(filereader: TrustedFileReader, gen_id: GenerationId, error: Error) {
        let fr = filereader.root();
        // Step 1
        fr.change_ready_state(FileReaderReadyState::Done);
        *fr.result.borrow_mut() = None;
        //FIXME set error attribute
        fr.dispatch_progress_event(gen_id, "error".to_owned(), 0, None);
        // Step 3
        fr.dispatch_progress_event(gen_id, "loadend".to_owned(), 0, None);
        // Step 4
        fr.terminate_ongoing_reading();
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn process_read_data(filereader: TrustedFileReader, gen_id: GenerationId, payload: DOMString) {
        let fr = filereader.root();
        // Step 7
        fr.dispatch_progress_event(gen_id, "progress".to_owned(), 0, None);
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn process_read(filereader: TrustedFileReader, gen_id: GenerationId) {
        // Step 6
        fr.thread_dispatch_progress_event(gen_id, "loadstart".to_owned(), 0, None);
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn process_read_eof(filereader: TrustedFileReader, gen_id: GenerationId, data: DOMString) {
        let fr = filereader.root();

        fr.dispatch_progress_event(gen_id, "progress".to_owned(), 0, None);
        // Step 8.1
        fr.change_ready_state(FileReaderReadyState::Done);
        // Step 8.2
        *fr.result.borrow_mut() = Some(data);
        // Step 8.3
        fr.dispatch_progress_event(gen_id, "load".to_owned(), 0, None);
        // Step 8.4
        if fr.ready_state.get() as u16 != FileReaderReadyState::Loading as u16 {
            fr.dispatch_progress_event(gen_id, "loadend".to_owned(), 0, None);
        }
        // Step 9 ?
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn handle_read(filereader: TrustedFileReader, gen_id: GenerationId, read_data: ReadData) {
        // Step 4
        FileReader::process_read(filereader.clone(),gen_id);
        println!("{}", "test2");

        FileReader::process_read_data(filereader.clone(), gen_id, DOMString::new());
        println!("{}", "test3");
        let encoding = match read_data.label {
            Some(e) => encoding_from_whatwg_label(&e),
            None => Some(UTF_8 as EncodingRef)
        };
        println!("{}", "test4");

        let enc = match encoding {
            Some(code) => code,
            None => {
                FileReader::process_read_error(filereader.clone(), gen_id, Error::NotSupported);
                return;
            }
        };
        let bytes = match read_data.bytes.recv() {
            Ok(data) => data,
            Err(_) => {
                FileReader::process_read_error(filereader.clone(), gen_id, Error::NotFound);
                return;
            }
        };
        let input = match bytes {
            Some(bytes) => bytes,
            None => {
                FileReader::process_read_eof(filereader.clone(), gen_id, DOMString::new());
                return;
            }
        };
        println!("{}", "test5");
        // Step 5
        FileReader::process_read_data(filereader.clone(), gen_id, DOMString::new());
        let (_, convert) = input.split_at(0);

        let output = enc.decode(convert, DecoderTrap::Strict);
        match output {
            Ok(s) => {
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

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    fn ReadAsText(self,blob: &Blob,label:Option<DOMString>) -> ErrorResult {
        let global = self.global.root();
        // Step 1
        if self.ready_state.get() == FileReaderReadyState::Loading {
            return Err(InvalidState);
        }
        //TODO STEP 2 if isClosed implemented in Blob

        // Step 3
        self.change_ready_state(FileReaderReadyState::Loading);

        let bytes = blob.read_out_buffer();
        let type_ = blob.read_out_type();

        let load_data = ReadData::new(bytes, type_, label);

        self.read(load_data, global.r())
    }


    // https://w3c.github.io/FileAPI/#dfn-abort
    fn Abort(self) {
        // Step 2
        if self.ready_state.get() == FileReaderReadyState::Loading {
            self.change_ready_state(FileReaderReadyState::Done);
        }
        // Steps 1 & 3
        *self.result.borrow_mut() = None;

        self.terminate_ongoing_reading();
        // Steps 5 & 6
        self.dispatch_progress_event(self.generation_id.get(), "abort".to_owned(), 0, None);
        self.dispatch_progress_event(self.generation_id.get(), "loadend".to_owned(), 0, None);
    }

    fn GetError(self) -> Option<Root<DOMException>> {
        //FIXME Return the current error state
        self.error.get().map(|error| error.root())
    }

    fn GetResult(self) -> Option<DOMString> {
        self.result.borrow().clone()
    }

    fn ReadyState(self) -> u16 {
        self.ready_state.get() as u16
    }
}

trait PrivateFileReaderHelpers {
    fn dispatch_progress_event(self, gen_id: GenerationId, type_: DOMString, loaded: u64, total: Option<u64>);
    fn terminate_ongoing_reading(self);
    fn read(self, read_data: ReadData,  global: GlobalRef) -> ErrorResult;
    fn change_ready_state(self, state: FileReaderReadyState);
}

impl<'a> PrivateFileReaderHelpers for &'a FileReader {

    fn dispatch_progress_event(self, gen_id: GenerationId, type_: DOMString, loaded: u64, total: Option<u64>) {
        //let GenerationId(cur_id) = self.generation_id.get();
        //let GenerationId(thread_id) = gen_id;

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
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
    }

    fn read(self, read_data: ReadData, global: GlobalRef) -> ErrorResult {

        let fr = Trusted::new(global.get_cx(), self, global.script_chan());
        let gen_id = self.generation_id.get();

        let task = box FileReaderHandler::new(gen_id, read_data, fr);

        let (setup_chan, setup_port) = channel();

        spawn_named("FileReaderHandler".to_owned(), move || {
            loop {
                match setup_port.recv() {
                    Ok(s) => {
                        match s {
                            ScriptMsg::RunnableMsg(task) =>{
                                task.handler();
                                break
                            },
                            _ => {
                                panic!("Unexpected message");
                            }
                        }
                    },
                    Err(_) => {}
                }
            }
        });
        setup_chan.send(ScriptMsg::RunnableMsg(task)).unwrap();
        Ok(())
    }

    fn change_ready_state(self, state: FileReaderReadyState) {
        self.ready_state.set(state);
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
        FileReader::handle_read(this.filereader, this.gen_id, this.read_data);
    }
}
