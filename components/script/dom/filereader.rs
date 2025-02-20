/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr;

use base64::Engine;
use dom_struct::dom_struct;
use encoding_rs::{Encoding, UTF_8};
use js::jsapi::{Heap, JSObject};
use js::jsval::{self, JSVal};
use js::rust::HandleObject;
use js::typedarray::{ArrayBuffer, CreateWith};
use mime::{self, Mime};
use servo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::Bindings::FileReaderBinding::{
    FileReaderConstants, FileReaderMethods,
};
use crate::dom::bindings::codegen::UnionTypes::StringOrObject;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::blob::Blob;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::progressevent::ProgressEvent;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext};
use crate::task::TaskOnce;

#[allow(dead_code)]
pub(crate) enum FileReadingTask {
    ProcessRead(TrustedFileReader, GenerationId),
    ProcessReadData(TrustedFileReader, GenerationId),
    ProcessReadError(TrustedFileReader, GenerationId, DOMErrorName),
    ProcessReadEOF(TrustedFileReader, GenerationId, ReadMetaData, Vec<u8>),
}

impl TaskOnce for FileReadingTask {
    fn run_once(self) {
        self.handle_task(CanGc::note());
    }
}

impl FileReadingTask {
    pub(crate) fn handle_task(self, can_gc: CanGc) {
        use self::FileReadingTask::*;

        match self {
            ProcessRead(reader, gen_id) => FileReader::process_read(reader, gen_id, can_gc),
            ProcessReadData(reader, gen_id) => {
                FileReader::process_read_data(reader, gen_id, can_gc)
            },
            ProcessReadError(reader, gen_id, error) => {
                FileReader::process_read_error(reader, gen_id, error, can_gc)
            },
            ProcessReadEOF(reader, gen_id, metadata, blob_contents) => {
                FileReader::process_read_eof(reader, gen_id, metadata, blob_contents, can_gc)
            },
        }
    }
}
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum FileReaderFunction {
    Text,
    DataUrl,
    ArrayBuffer,
}

pub(crate) type TrustedFileReader = Trusted<FileReader>;

#[derive(Clone, MallocSizeOf)]
pub(crate) struct ReadMetaData {
    pub(crate) blobtype: String,
    pub(crate) label: Option<String>,
    pub(crate) function: FileReaderFunction,
}

impl ReadMetaData {
    pub(crate) fn new(
        blobtype: String,
        label: Option<String>,
        function: FileReaderFunction,
    ) -> ReadMetaData {
        ReadMetaData {
            blobtype,
            label,
            function,
        }
    }
}

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct GenerationId(u32);

#[repr(u16)]
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum FileReaderReadyState {
    Empty = FileReaderConstants::EMPTY,
    Loading = FileReaderConstants::LOADING,
    Done = FileReaderConstants::DONE,
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum FileReaderResult {
    ArrayBuffer(#[ignore_malloc_size_of = "mozjs"] Heap<JSVal>),
    String(DOMString),
}

pub(crate) struct FileReaderSharedFunctionality;

impl FileReaderSharedFunctionality {
    pub(crate) fn dataurl_format(blob_contents: &[u8], blob_type: String) -> DOMString {
        let base64 = base64::engine::general_purpose::STANDARD.encode(blob_contents);

        let dataurl = if blob_type.is_empty() {
            format!("data:base64,{}", base64)
        } else {
            format!("data:{};base64,{}", blob_type, base64)
        };

        DOMString::from(dataurl)
    }

    pub(crate) fn text_decode(
        blob_contents: &[u8],
        blob_type: &str,
        blob_label: &Option<String>,
    ) -> DOMString {
        //https://w3c.github.io/FileAPI/#encoding-determination
        // Steps 1 & 2 & 3
        let mut encoding = blob_label
            .as_ref()
            .map(|string| string.as_bytes())
            .and_then(Encoding::for_label);

        // Step 4 & 5
        encoding = encoding.or_else(|| {
            let resultmime = blob_type.parse::<Mime>().ok();
            resultmime.and_then(|mime| {
                mime.params()
                    .find(|(ref k, _)| &mime::CHARSET == k)
                    .and_then(|(_, ref v)| Encoding::for_label(v.as_ref().as_bytes()))
            })
        });

        // Step 6
        let enc = encoding.unwrap_or(UTF_8);

        let convert = blob_contents;
        // Step 7
        let (output, _, _) = enc.decode(convert);
        DOMString::from(output)
    }
}

#[dom_struct]
pub(crate) struct FileReader {
    eventtarget: EventTarget,
    ready_state: Cell<FileReaderReadyState>,
    error: MutNullableDom<DOMException>,
    result: DomRefCell<Option<FileReaderResult>>,
    generation_id: Cell<GenerationId>,
}

impl FileReader {
    pub(crate) fn new_inherited() -> FileReader {
        FileReader {
            eventtarget: EventTarget::new_inherited(),
            ready_state: Cell::new(FileReaderReadyState::Empty),
            error: MutNullableDom::new(None),
            result: DomRefCell::new(None),
            generation_id: Cell::new(GenerationId(0)),
        }
    }

    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<FileReader> {
        reflect_dom_object_with_proto(Box::new(FileReader::new_inherited()), global, proto, can_gc)
    }

    //https://w3c.github.io/FileAPI/#dfn-error-steps
    pub(crate) fn process_read_error(
        filereader: TrustedFileReader,
        gen_id: GenerationId,
        error: DOMErrorName,
        can_gc: CanGc,
    ) {
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

        let exception = DOMException::new(&fr.global(), error, can_gc);
        fr.error.set(Some(&exception));

        fr.dispatch_progress_event(atom!("error"), 0, None, can_gc);
        return_on_abort!();
        // Step 3
        fr.dispatch_progress_event(atom!("loadend"), 0, None, can_gc);
        return_on_abort!();
        // Step 4
        fr.terminate_ongoing_reading();
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub(crate) fn process_read_data(
        filereader: TrustedFileReader,
        gen_id: GenerationId,
        can_gc: CanGc,
    ) {
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
        fr.dispatch_progress_event(atom!("progress"), 0, None, can_gc);
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub(crate) fn process_read(filereader: TrustedFileReader, gen_id: GenerationId, can_gc: CanGc) {
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
        fr.dispatch_progress_event(atom!("loadstart"), 0, None, can_gc);
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    pub(crate) fn process_read_eof(
        filereader: TrustedFileReader,
        gen_id: GenerationId,
        data: ReadMetaData,
        blob_contents: Vec<u8>,
        can_gc: CanGc,
    ) {
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

        match data.function {
            FileReaderFunction::DataUrl => {
                FileReader::perform_readasdataurl(&fr.result, data, &blob_contents)
            },
            FileReaderFunction::Text => {
                FileReader::perform_readastext(&fr.result, data, &blob_contents)
            },
            FileReaderFunction::ArrayBuffer => {
                let _ac = enter_realm(&*fr);
                FileReader::perform_readasarraybuffer(
                    &fr.result,
                    GlobalScope::get_cx(),
                    data,
                    &blob_contents,
                )
            },
        };

        // Step 8.3
        fr.dispatch_progress_event(atom!("load"), 0, None, can_gc);
        return_on_abort!();
        // Step 8.4
        if fr.ready_state.get() != FileReaderReadyState::Loading {
            fr.dispatch_progress_event(atom!("loadend"), 0, None, can_gc);
        }
        return_on_abort!();
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    fn perform_readastext(
        result: &DomRefCell<Option<FileReaderResult>>,
        data: ReadMetaData,
        blob_bytes: &[u8],
    ) {
        let blob_label = &data.label;
        let blob_type = &data.blobtype;

        let output = FileReaderSharedFunctionality::text_decode(blob_bytes, blob_type, blob_label);
        *result.borrow_mut() = Some(FileReaderResult::String(output));
    }

    //https://w3c.github.io/FileAPI/#dfn-readAsDataURL
    fn perform_readasdataurl(
        result: &DomRefCell<Option<FileReaderResult>>,
        data: ReadMetaData,
        bytes: &[u8],
    ) {
        let output = FileReaderSharedFunctionality::dataurl_format(bytes, data.blobtype);

        *result.borrow_mut() = Some(FileReaderResult::String(output));
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsArrayBuffer
    #[allow(unsafe_code)]
    fn perform_readasarraybuffer(
        result: &DomRefCell<Option<FileReaderResult>>,
        cx: JSContext,
        _: ReadMetaData,
        bytes: &[u8],
    ) {
        unsafe {
            rooted!(in(*cx) let mut array_buffer = ptr::null_mut::<JSObject>());
            assert!(
                ArrayBuffer::create(*cx, CreateWith::Slice(bytes), array_buffer.handle_mut())
                    .is_ok()
            );

            *result.borrow_mut() = Some(FileReaderResult::ArrayBuffer(Heap::default()));

            if let Some(FileReaderResult::ArrayBuffer(ref mut heap)) = *result.borrow_mut() {
                heap.set(jsval::ObjectValue(array_buffer.get()));
            };
        }
    }
}

impl FileReaderMethods<crate::DomTypeHolder> for FileReader {
    // https://w3c.github.io/FileAPI/#filereaderConstrctr
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<FileReader>> {
        Ok(FileReader::new(global, proto, can_gc))
    }

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

    // https://w3c.github.io/FileAPI/#dfn-readAsArrayBuffer
    fn ReadAsArrayBuffer(&self, blob: &Blob) -> ErrorResult {
        self.read(FileReaderFunction::ArrayBuffer, blob, None)
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsDataURL
    fn ReadAsDataURL(&self, blob: &Blob) -> ErrorResult {
        self.read(FileReaderFunction::DataUrl, blob, None)
    }

    // https://w3c.github.io/FileAPI/#dfn-readAsText
    fn ReadAsText(&self, blob: &Blob, label: Option<DOMString>) -> ErrorResult {
        self.read(FileReaderFunction::Text, blob, label)
    }

    // https://w3c.github.io/FileAPI/#dfn-abort
    fn Abort(&self, can_gc: CanGc) {
        // Step 2
        if self.ready_state.get() == FileReaderReadyState::Loading {
            self.change_ready_state(FileReaderReadyState::Done);
        }
        // Steps 1 & 3
        *self.result.borrow_mut() = None;

        let exception = DOMException::new(&self.global(), DOMErrorName::AbortError, can_gc);
        self.error.set(Some(&exception));

        self.terminate_ongoing_reading();
        // Steps 5 & 6
        self.dispatch_progress_event(atom!("abort"), 0, None, can_gc);
        self.dispatch_progress_event(atom!("loadend"), 0, None, can_gc);
    }

    // https://w3c.github.io/FileAPI/#dfn-error
    fn GetError(&self) -> Option<DomRoot<DOMException>> {
        self.error.get()
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/FileAPI/#dfn-result
    fn GetResult(&self, _: JSContext) -> Option<StringOrObject> {
        self.result.borrow().as_ref().map(|r| match *r {
            FileReaderResult::String(ref string) => StringOrObject::String(string.clone()),
            FileReaderResult::ArrayBuffer(ref arr_buffer) => {
                let result = RootedTraceableBox::new(Heap::default());
                unsafe {
                    result.set((*arr_buffer.ptr.get()).to_object());
                }
                StringOrObject::Object(result)
            },
        })
    }

    // https://w3c.github.io/FileAPI/#dfn-readyState
    fn ReadyState(&self) -> u16 {
        self.ready_state.get() as u16
    }
}

impl FileReader {
    fn dispatch_progress_event(&self, type_: Atom, loaded: u64, total: Option<u64>, can_gc: CanGc) {
        let progressevent = ProgressEvent::new(
            &self.global(),
            type_,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            total.is_some(),
            loaded,
            total.unwrap_or(0),
            can_gc,
        );
        progressevent.upcast::<Event>().fire(self.upcast(), can_gc);
    }

    fn terminate_ongoing_reading(&self) {
        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
    }

    /// <https://w3c.github.io/FileAPI/#readOperation>
    fn read(
        &self,
        function: FileReaderFunction,
        blob: &Blob,
        label: Option<DOMString>,
    ) -> ErrorResult {
        // Step 1
        if self.ready_state.get() == FileReaderReadyState::Loading {
            return Err(Error::InvalidState);
        }

        // Step 2
        self.change_ready_state(FileReaderReadyState::Loading);

        // Step 3
        *self.result.borrow_mut() = None;

        let type_ = blob.Type();

        let load_data = ReadMetaData::new(String::from(type_), label.map(String::from), function);

        let GenerationId(prev_id) = self.generation_id.get();
        self.generation_id.set(GenerationId(prev_id + 1));
        let gen_id = self.generation_id.get();

        // Step 10, in parallel, wait on stream promises to resolve and queue tasks.

        // TODO: follow the spec which requires implementing blob `get_stream`,
        // see https://github.com/servo/servo/issues/25209

        // Currently bytes are first read "sync", and then the appropriate tasks are queued.

        // Read the blob bytes "sync".
        let blob_contents = blob.get_bytes().unwrap_or_else(|_| vec![]);

        let filereader = Trusted::new(self);
        let global = self.global();
        let task_manager = global.task_manager();
        let task_source = task_manager.file_reading_task_source();

        // Queue tasks as appropriate.
        task_source.queue(FileReadingTask::ProcessRead(filereader.clone(), gen_id));

        if !blob_contents.is_empty() {
            task_source.queue(FileReadingTask::ProcessReadData(filereader.clone(), gen_id));
        }

        task_source.queue(FileReadingTask::ProcessReadEOF(
            filereader,
            gen_id,
            load_data,
            blob_contents,
        ));

        Ok(())
    }

    fn change_ready_state(&self, state: FileReaderReadyState) {
        self.ready_state.set(state);
    }
}
