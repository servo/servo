/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::FileReaderBinding::{self, FileReaderConstants, FileReaderMethods};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::{GlobalField, GlobalRef};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::blob::Blob;
use dom::domexception::{DOMErrorName, DOMException};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::progressevent::ProgressEvent;
use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, EncodingRef};
use hyper::mime::{Attr, Mime};
use rustc_serialize::base64::{CharacterSet, Config, Newline, ToBase64};
use script_task::ScriptTaskEventCategory::FileRead;
use script_task::{CommonScriptMsg, Runnable, ScriptChan, ScriptPort};
use std::cell::Cell;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use util::str::DOMString;
use util::task::spawn_named;

#[derive(PartialEq, Clone, Copy, JSTraceable, HeapSizeOf)]
pub enum FileReaderFunction {
    ReadAsText,
    ReadAsDataUrl,
}

pub type TrustedFileReader = Trusted<FileReader>;

#[derive(Clone, HeapSizeOf)]
pub struct ReadMetaData {
    pub blobtype: String,
    pub label: Option<String>,
    pub function: FileReaderFunction
}

impl ReadMetaData {
    pub fn new(blobtype: String,
               label: Option<String>, function: FileReaderFunction) -> ReadMetaData {
        ReadMetaData {
            blobtype: blobtype,
            label: label,
            function: function,
        }
    }
}

#[derive(PartialEq, Clone, Copy, JSTraceable, HeapSizeOf)]
pub struct GenerationId(u32);

#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq, JSTraceable, HeapSizeOf)]
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
    result: DOMRefCell<Option<DOMString>>,
    generation_id: Cell<GenerationId>,
}

impl FileReader {
    pub fn new_inherited(global: GlobalRef) -> FileReader {
        FileReader {
            eventtarget: EventTarget::new_inherited(),//?
            global: GlobalField::from_rooted(&global),
            ready_state: Cell::new(FileReaderReadyState::Empty),
            error: MutNullableHeap::new(None),
            result: DOMRefCell::new(None),
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
        fr.error.set(Some(&exception));

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
    pub fn process_read_eof(filereader: TrustedFileReader, gen_id: GenerationId,
                            data: ReadMetaData, blob_contents: Vec<u8>) {
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
        let output = match data.function {
            FileReaderFunction::ReadAsDataUrl =>
                FileReader::perform_readasdataurl(data, blob_contents),
            FileReaderFunction::ReadAsText =>
                FileReader::perform_readastext(data, blob_contents),
        };

        *fr.result.borrow_mut() = Some(output);

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
    fn perform_readastext(data: ReadMetaData, blob_contents: Vec<u8>)
        -> DOMString {

        let blob_label = &data.label;
        let blob_type = &data.blobtype;
        let blob_bytes = &blob_contents[..];

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
        DOMString::from(output)
    }

    //https://w3c.github.io/FileAPI/#dfn-readAsDataURL
    fn perform_readasdataurl(data: ReadMetaData, blob_contents: Vec<u8>)
        -> DOMString {
        let config = Config {
            char_set: CharacterSet::UrlSafe,
            newline: Newline::LF,
            pad: true,
            line_length: None
        };
        let base64 = blob_contents.to_base64(config);

        let output = if data.blobtype.is_empty() {
            format!("data:base64,{}", base64)
        } else {
            format!("data:{};base64,{}", data.blobtype, base64)
        };

        DOMString::from(output)
    }
}

impl FileReaderMethods for FileReader {
    // https://w3c.github.io/FileAPI/#dfn-onloadstart
    event_handler!(loadstart, GetOnloadstart, SetOnloadstart);

    // https://w3c.github.io/FileAPI/#dfn-onprogress
    event_handler!(progress, GetOnprogress, SetOnprogress);

    // https://w3c.github.io/FileAPI/#dfn-onload
    event_handler!(load, GetOnload, SetOnload);

    // https://w3c.github.io/FileAPI/#dfn-onabort
    event_handler!(abort, GetOnabort, SetOnabort);

    // https://w3c.github.io/FileAPI/#dfn-onerror
    event_handler!(error, GetOnerror, SetOnerror);

    // https://w3c.github.io/FileAPI/#dfn-onloadend
    event_handler!(loadend, GetOnloadend, SetOnloadend);

    //TODO https://w3c.github.io/FileAPI/#dfn-readAsArrayBuffer
    // https://w3c.github.io/FileAPI/#dfn-readAsDataURL
    fn ReadAsDataURL(&self, blob: &Blob) -> ErrorResult {
        self.read(FileReaderFunction::ReadAsDataUrl, blob, None)
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    fn ReadAsText(&self, blob: &Blob, label: Option<DOMString>) -> ErrorResult {
        self.read(FileReaderFunction::ReadAsText, blob, label)
    }

    // https://w3c.github.io/FileAPI/#dfn-abort
    fn Abort(&self) {
        // Step 2
        if self.ready_state.get() == FileReaderReadyState::Loading {
            self.change_ready_state(FileReaderReadyState::Done);
        }
        // Steps 1 & 3
        *self.result.borrow_mut() = None;

        let global = self.global.root();
        let exception = DOMException::new(global.r(), DOMErrorName::AbortError);
        self.error.set(Some(&exception));

        self.terminate_ongoing_reading();
        // Steps 5 & 6
        self.dispatch_progress_event("abort".to_owned(), 0, None);
        self.dispatch_progress_event("loadend".to_owned(), 0, None);
    }

    // https://w3c.github.io/FileAPI/#dfn-error
    fn GetError(&self) -> Option<Root<DOMException>> {
        self.error.get()
    }

    // https://w3c.github.io/FileAPI/#dfn-result
    fn GetResult(&self) -> Option<DOMString> {
        self.result.borrow().clone()
    }

    // https://w3c.github.io/FileAPI/#dfn-readyState
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }
}


impl FileReader {
    fn dispatch_progress_event(&self, type_: String, loaded: u64, total: Option<u64>) {

        let global = self.global.root();
        let progressevent = ProgressEvent::new(global.r(),
            DOMString::from(type_), EventBubbles::DoesNotBubble, EventCancelable::NotCancelable,
            total.is_some(), loaded, total.unwrap_or(0));
        progressevent.upcast::<Event>().fire(self.upcast());
    }

    fn terminate_ongoing_reading(&self) {
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
    }

    fn read(&self, function: FileReaderFunction, blob: &Blob, label: Option<DOMString>) -> ErrorResult {
        let root = self.global.root();
        let global = root.r();
        // Step 1
        if self.ready_state.get() == FileReaderReadyState::Loading {
            return Err(Error::InvalidState);
        }
        // Step 2
        if blob.IsClosed() {
            let global = self.global.root();
            let exception = DOMException::new(global.r(), DOMErrorName::InvalidStateError);
            self.error.set(Some(&exception));

            self.dispatch_progress_event("error".to_owned(), 0, None);
            return Ok(());
        }

        // Step 3
        self.change_ready_state(FileReaderReadyState::Loading);

        // Step 4
        let (send, bytes) = mpsc::channel();
        blob.read_out_buffer(send);
        let type_ = blob.Type();

        let load_data = ReadMetaData::new(String::from(type_), label.map(String::from), function);

        let fr = Trusted::new(global.get_cx(), self, global.script_chan());
        let gen_id = self.generation_id.get();

        let script_chan = global.script_chan();

        spawn_named("file reader async operation".to_owned(), move || {
            perform_annotated_read_operation(gen_id, load_data, bytes, fr, script_chan)
        });
        Ok(())
    }

    fn change_ready_state(&self, state: FileReaderReadyState) {
        self.ready_state.set(state);
    }
}

#[derive(Clone)]
pub enum FileReaderEvent {
    ProcessRead(TrustedFileReader, GenerationId),
    ProcessReadData(TrustedFileReader, GenerationId),
    ProcessReadError(TrustedFileReader, GenerationId, DOMErrorName),
    ProcessReadEOF(TrustedFileReader, GenerationId, ReadMetaData, Vec<u8>)
}

impl Runnable for FileReaderEvent {
    fn handler(self: Box<FileReaderEvent>) {
        let file_reader_event = *self;
        match file_reader_event {
            FileReaderEvent::ProcessRead(filereader, gen_id) => {
                FileReader::process_read(filereader, gen_id);
            },
            FileReaderEvent::ProcessReadData(filereader, gen_id) => {
                FileReader::process_read_data(filereader, gen_id);
            },
            FileReaderEvent::ProcessReadError(filereader, gen_id, error) => {
                FileReader::process_read_error(filereader, gen_id, error);
            },
            FileReaderEvent::ProcessReadEOF(filereader, gen_id, data, blob_contents) => {
                FileReader::process_read_eof(filereader, gen_id, data, blob_contents);
            }
        }
    }
}

// https://w3c.github.io/FileAPI/#task-read-operation
fn perform_annotated_read_operation(gen_id: GenerationId, data: ReadMetaData, blob_contents: Receiver<Vec<u8>>,
    filereader: TrustedFileReader, script_chan: Box<ScriptChan + Send>) {
    let chan = &script_chan;
    // Step 4
    let task = box FileReaderEvent::ProcessRead(filereader.clone(), gen_id);
    chan.send(CommonScriptMsg::RunnableMsg(FileRead, task)).unwrap();

    let task = box FileReaderEvent::ProcessReadData(filereader.clone(), gen_id);
    chan.send(CommonScriptMsg::RunnableMsg(FileRead, task)).unwrap();

    let bytes = match blob_contents.recv() {
        Ok(bytes) => bytes,
        Err(_) => {
            let task = box FileReaderEvent::ProcessReadError(filereader,
                gen_id, DOMErrorName::NotFoundError);
            chan.send(CommonScriptMsg::RunnableMsg(FileRead, task)).unwrap();
            return;
        }
    };

    let task = box FileReaderEvent::ProcessReadEOF(filereader, gen_id, data, bytes);
    chan.send(CommonScriptMsg::RunnableMsg(FileRead, task)).unwrap();
}
