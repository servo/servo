/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FileReaderBinding::{self, FileReaderConstants, FileReaderMethods};
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::InvalidState;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{Root, JS, MutNullableHeap};
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{reflect_dom_object, Reflectable};
use dom::event::{EventHelpers, EventCancelable, EventBubbles};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::blob::{Blob, BlobHelpers};
use dom::domexception::{DOMException, DOMErrorName};
use dom::progressevent::ProgressEvent;
use encoding::all::UTF_8;
use encoding::types::{EncodingRef, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use hyper::mime::{Mime, Attr};
use script_task::{ScriptChan, ScriptMsg, Runnable, ScriptPort};
use std::cell::{Cell, RefCell};
use std::sync::mpsc::Receiver;
use util::str::DOMString;
use util::task::spawn_named;
use rustc_serialize::base64::{Config, ToBase64, CharacterSet, Newline};

#[derive(PartialEq, Clone, Copy, JSTraceable)]
pub enum FileReaderFunction {
    ReadAsText,
    ReadAsDataUrl,
}

pub type TrustedFileReader = Trusted<FileReader>;

pub struct ReadData {
    pub bytes: Receiver<Option<Vec<u8>>>,
    pub blobtype: DOMString,
    pub label: Option<DOMString>,
    pub function: FileReaderFunction
}

impl ReadData {
    pub fn new(bytes: Receiver<Option<Vec<u8>>>, blobtype: DOMString,
               label: Option<DOMString>, function: FileReaderFunction) -> ReadData {
        ReadData {
            bytes: bytes,
            blobtype: blobtype,
            label: label,
            function: function,
        }
    }
}

#[derive(Clone)]
pub struct BlobBody {
    pub bytes: Vec<u8>,
    pub blobtype: DOMString,
    pub label: Option<DOMString>,
    pub function: FileReaderFunction
}

impl BlobBody {
    pub fn new(bytes: Vec<u8>, blobtype: DOMString,
               label: Option<DOMString>, function: FileReaderFunction) -> BlobBody {
        BlobBody {
            bytes: bytes,
            blobtype: blobtype,
            label: label,
            function: function,
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
    pub fn process_read_error(filereader: TrustedFileReader, gen_id: GenerationId, error: DOMErrorName) {
        let fr = filereader.root();

        macro_rules! return_on_abort(
            () => (
                if gen_id != fr.generation_id.get() {
                    return
                }
            );
        );

        return_on_abort!();
        // Step 1
        fr.change_ready_state(FileReaderReadyState::Done);
        *fr.result.borrow_mut() = None;

        let global = fr.global.root();
        let exception = DOMException::new(global.r(), error);
        fr.error.set(Some(JS::from_rooted(&exception)));

        fr.dispatch_progress_event("error".to_owned(), 0, None);
        return_on_abort!();
        // Step 3
        fr.dispatch_progress_event("loadend".to_owned(), 0, None);
        return_on_abort!();
        // Step 4
        fr.terminate_ongoing_reading();
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn process_read_data(filereader: TrustedFileReader, gen_id: GenerationId) {
        let fr = filereader.root();

        macro_rules! return_on_abort(
            () => (
                if gen_id != fr.generation_id.get() {
                    return
                }
            );
        );
        return_on_abort!();
        //FIXME Step 7 send current progress
        fr.dispatch_progress_event("progress".to_owned(), 0, None);
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn process_read(filereader: TrustedFileReader, gen_id: GenerationId) {
        let fr = filereader.root();

        macro_rules! return_on_abort(
            () => (
                if gen_id != fr.generation_id.get() {
                    return
                }
            );
        );
        return_on_abort!();
        // Step 6
        fr.dispatch_progress_event("loadstart".to_owned(), 0, None);
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub fn process_read_eof(filereader: TrustedFileReader, gen_id: GenerationId, data: Option<BlobBody>) {
        let fr = filereader.root();

        macro_rules! return_on_abort(
            () => (
                if gen_id != fr.generation_id.get() {
                    return
                }
            );
        );

        return_on_abort!();
        // Step 8.1
        fr.change_ready_state(FileReaderReadyState::Done);
        // Step 8.2
        let output = match data {
            Some(blob_body) => {
                match blob_body.function {
                    FileReaderFunction::ReadAsDataUrl =>
                        FileReader::perform_readasdataurl(blob_body),
                    FileReaderFunction::ReadAsText =>
                        FileReader::perform_readastext(blob_body),
                }
            },
            None => {
                Ok(None)
            }
        };

        //FIXME handle error if error is possible
        *fr.result.borrow_mut() = output.unwrap();

        // Step 8.3
        fr.dispatch_progress_event("load".to_owned(), 0, None);
        return_on_abort!();
        // Step 8.4
        if fr.ready_state.get() != FileReaderReadyState::Loading {
            fr.dispatch_progress_event("loadend".to_owned(), 0, None);
        }
        return_on_abort!();
        // Step 9
        fr.terminate_ongoing_reading();
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    fn perform_readastext(blob_body: BlobBody)
        -> Result<Option<DOMString>, DOMErrorName> {

        let blob_label = &blob_body.label;
        let blob_type = &blob_body.blobtype;
        let blob_bytes = &blob_body.bytes[..];

        //https://w3c.github.io/FileAPI/#encoding-determination
        // Steps 1 & 2 & 3
        let mut encoding = blob_label.as_ref()
            .map(|string| &**string)
            .and_then(encoding_from_whatwg_label);

        // Step 4 & 5
        encoding = encoding.or_else(|| {
            let resultmime = blob_type.parse::<Mime>().ok();
            resultmime.and_then(|Mime(_, _, ref parameters)| {
                parameters.iter()
                    .find(|&&(ref k, _)| &Attr::Charset == k)
                    .and_then(|&(_, ref v)| encoding_from_whatwg_label(&v.to_string()))
            })
        });

        // Step 6
        let enc = encoding.unwrap_or(UTF_8 as EncodingRef);

        let convert = blob_bytes;
        // Step 7
        let output = enc.decode(convert, DecoderTrap::Replace).unwrap();
        Ok(Some(output))
    }

    //https://w3c.github.io/FileAPI/#dfn-readAsDataURL
    fn perform_readasdataurl(blob_body: BlobBody)
        -> Result<Option<DOMString>, DOMErrorName> {
        let config = Config {
            char_set: CharacterSet::UrlSafe,
            newline: Newline::LF,
            pad: true,
            line_length: None
        };
        let base64 = blob_body.bytes.to_base64(config);

        let output = if blob_body.blobtype.is_empty() {
            format!("data:base64,{}", base64)
        } else {
            format!("data:{};base64,{}", blob_body.blobtype, base64)
        };

        Ok(Some(output))
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
    //https://w3c.github.io/FileAPI/#dfn-readAsDataURL
    fn ReadAsDataURL(self, blob: &Blob) -> ErrorResult {
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

        let load_data = ReadData::new(bytes, type_, None, FileReaderFunction::ReadAsDataUrl);

        self.read(load_data, global.r())
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    fn ReadAsText(self, blob: &Blob, label:Option<DOMString>) -> ErrorResult {
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

        let load_data = ReadData::new(bytes, type_, label, FileReaderFunction::ReadAsText);

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

        let global = self.global.root();
        let exception = DOMException::new(global.r(), DOMErrorName::AbortError);
        self.error.set(Some(JS::from_rooted(&exception)));

        self.terminate_ongoing_reading();
        // Steps 5 & 6
        self.dispatch_progress_event("abort".to_owned(), 0, None);
        self.dispatch_progress_event("loadend".to_owned(), 0, None);
    }

    fn GetError(self) -> Option<Root<DOMException>> {
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
    fn dispatch_progress_event(self, type_: DOMString, loaded: u64, total: Option<u64>);
    fn terminate_ongoing_reading(self);
    fn read(self, read_data: ReadData,  global: GlobalRef) -> ErrorResult;
    fn change_ready_state(self, state: FileReaderReadyState);
}

impl<'a> PrivateFileReaderHelpers for &'a FileReader {
    fn dispatch_progress_event(self, type_: DOMString, loaded: u64, total: Option<u64>) {

        let global = self.global.root();
        let progressevent = ProgressEvent::new(global.r(),
            type_, EventBubbles::DoesNotBubble, EventCancelable::NotCancelable,
            total.is_some(), loaded, total.unwrap_or(0));

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

        let script_chan = global.script_chan();

        spawn_named("file reader async operation".to_owned(), move || {
            perform_annotated_read_operation(gen_id, read_data, fr, script_chan)
        });
        Ok(())
    }

    fn change_ready_state(self, state: FileReaderReadyState) {
        self.ready_state.set(state);
    }
}

#[derive(Clone)]
pub enum FileReaderEvent {
    ProcessRead(TrustedFileReader, GenerationId),
    ProcessReadData(TrustedFileReader, GenerationId, DOMString),
    ProcessReadError(TrustedFileReader, GenerationId, DOMErrorName),
    ProcessReadEOF(TrustedFileReader, GenerationId, Option<BlobBody>)
}

impl Runnable for FileReaderEvent {
    fn handler(self: Box<FileReaderEvent>) {
        let file_reader_event = *self;
        match file_reader_event {
            FileReaderEvent::ProcessRead(filereader, gen_id) => {
                FileReader::process_read(filereader, gen_id);
            },
            FileReaderEvent::ProcessReadData(filereader, gen_id, _) => {
                FileReader::process_read_data(filereader, gen_id);
            },
            FileReaderEvent::ProcessReadError(filereader, gen_id, error) => {
                FileReader::process_read_error(filereader, gen_id, error);
            },
            FileReaderEvent::ProcessReadEOF(filereader, gen_id, blob_body) => {
                FileReader::process_read_eof(filereader, gen_id, blob_body);
            }
        }
    }
}

// https://w3c.github.io/FileAPI/#task-read-operation
fn perform_annotated_read_operation(gen_id: GenerationId, read_data: ReadData,
    filereader: TrustedFileReader, script_chan: Box<ScriptChan + Send>) {
    let chan = &script_chan;
    // Step 4
    let task = box FileReaderEvent::ProcessRead(filereader.clone(), gen_id);
    chan.send(ScriptMsg::RunnableMsg(task)).unwrap();

    let task = box FileReaderEvent::ProcessReadData(filereader.clone(),
        gen_id, DOMString::new());
    chan.send(ScriptMsg::RunnableMsg(task)).unwrap();

    let output = match read_data.bytes.recv() {
        Ok(bytes) => bytes,
        Err(_) => {
            let task = box FileReaderEvent::ProcessReadError(filereader,
                gen_id, DOMErrorName::NotFoundError);
            chan.send(ScriptMsg::RunnableMsg(task)).unwrap();
            return;
        }
    };

    let blobtype = read_data.blobtype.clone();
    let label = read_data.label.clone();

    let blob_body = output.map(|bytes| {
        BlobBody::new(bytes, blobtype, label, read_data.function)
    });

    let task = box FileReaderEvent::ProcessReadEOF(filereader, gen_id, blob_body);
    chan.send(ScriptMsg::RunnableMsg(task)).unwrap();
}
