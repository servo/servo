/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use encoding_rs::UTF_8;
use js::jsapi::JSObject;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use js::typedarray::{ArrayBufferU8, Uint8};
use net_traits::filemanager_thread::RelativePos;
use rustc_hash::FxHashMap;
use script_bindings::reflector::{
    Reflector, reflect_dom_object_with_proto, reflect_dom_object_with_proto_and_cx,
};
use servo_base::id::{BlobId, BlobIndex};
use servo_constellation_traits::{BlobData, BlobImpl};
use uuid::Uuid;

use crate::dom::bindings::buffer_source::create_buffer_source;
use crate::dom::bindings::codegen::Bindings::BlobBinding;
use crate::dom::bindings::codegen::Bindings::BlobBinding::BlobMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferOrArrayBufferViewOrBlobOrString;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::serializable::Serializable;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredData;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::stream::readablestream::ReadableStream;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/FileAPI/#dfn-Blob>
#[dom_struct]
pub(crate) struct Blob {
    reflector_: Reflector,
    #[no_trace]
    blob_id: BlobId,
}

impl Blob {
    pub(crate) fn new(global: &GlobalScope, blob_impl: BlobImpl, can_gc: CanGc) -> DomRoot<Blob> {
        Self::new_with_proto(global, None, blob_impl, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
        can_gc: CanGc,
    ) -> DomRoot<Blob> {
        let dom_blob = reflect_dom_object_with_proto(
            Box::new(Blob::new_inherited(&blob_impl)),
            global,
            proto,
            can_gc,
        );
        global.track_blob(&dom_blob, blob_impl);
        dom_blob
    }

    fn new_with_proto_and_cx(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blob_impl: BlobImpl,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<Blob> {
        let dom_blob = reflect_dom_object_with_proto_and_cx(
            Box::new(Blob::new_inherited(&blob_impl)),
            global,
            proto,
            cx,
        );
        global.track_blob(&dom_blob, blob_impl);
        dom_blob
    }

    pub(crate) fn new_inherited(blob_impl: &BlobImpl) -> Blob {
        Blob {
            reflector_: Reflector::new(),
            blob_id: blob_impl.blob_id(),
        }
    }

    /// Get a slice to inner data, this might incur synchronous read and caching
    pub(crate) fn get_bytes(&self) -> Result<Vec<u8>, ()> {
        self.global().get_blob_bytes(&self.blob_id)
    }

    /// Get a copy of the type_string
    pub(crate) fn type_string(&self) -> String {
        self.global().get_blob_type_string(&self.blob_id)
    }

    /// Get a FileID representing the Blob content,
    /// used by URL.createObjectURL
    pub(crate) fn get_blob_url_id(&self) -> Uuid {
        self.global().get_blob_url_id(&self.blob_id)
    }

    /// <https://w3c.github.io/FileAPI/#blob-get-stream>
    pub(crate) fn get_stream(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Fallible<DomRoot<ReadableStream>> {
        self.global().get_blob_stream(cx, &self.blob_id)
    }
}

impl Serializable for Blob {
    type Index = BlobIndex;
    type Data = BlobImpl;

    /// <https://w3c.github.io/FileAPI/#ref-for-serialization-steps>
    fn serialize(&self) -> Result<(BlobId, BlobImpl), ()> {
        let blob_id = self.blob_id;

        // 1. Get a clone of the blob impl.
        let blob_impl = self.global().serialize_blob(&blob_id);

        // We clone the data, but the clone gets its own Id.
        let new_blob_id = blob_impl.blob_id();

        Ok((new_blob_id, blob_impl))
    }

    /// <https://w3c.github.io/FileAPI/#ref-for-deserialization-steps>
    fn deserialize(
        owner: &GlobalScope,
        serialized: BlobImpl,
        can_gc: CanGc,
    ) -> Result<DomRoot<Self>, ()> {
        let deserialized_blob = Blob::new(owner, serialized, can_gc);
        Ok(deserialized_blob)
    }

    fn serialized_storage<'a>(
        reader: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<BlobId, Self::Data>> {
        match reader {
            StructuredData::Reader(r) => &mut r.blob_impls,
            StructuredData::Writer(w) => &mut w.blobs,
        }
    }
}

/// <https://w3c.github.io/FileAPI/#convert-line-endings-to-native>
fn convert_line_endings_to_native(s: &[u8]) -> Vec<u8> {
    let native_line_ending: &[u8] = if cfg!(target_os = "windows") {
        // Step 2. If the underlying platform’s conventions are to represent newlines
        // as a carriage return and line feed sequence,
        // set native line ending to the code point U+000D CR followed by the code point U+000A LF.
        b"\r\n"
    } else {
        // Step 1. Let native line ending be the code point U+000A LF.
        b"\n"
    };

    let len = s.len();
    // Step 3. Set result to the empty string.
    let mut result = Vec::with_capacity(len);

    // Step 4. Let position be a position variable for s, initially pointing at the start of s.
    let mut position = 0;

    // <https://infra.spec.whatwg.org/#collect-a-sequence-of-code-points>
    let collect_a_sequence_of_code_points = |position: &mut usize| -> &[u8] {
        let start = *position;
        while *position < len && s[*position] != b'\r' && s[*position] != b'\n' {
            *position += 1;
        }
        &s[start..*position]
    };

    // Step 5: Let token be the result of collecting a sequence of code points
    // that are not equal to U+000A LF or U+000D CR from s given position.
    // Step 6: Append token to result.
    result.extend_from_slice(collect_a_sequence_of_code_points(&mut position));

    // Step 7: While position is not past the end of s:
    while position < len {
        let byte = s[position];
        // Step 7.1: If the code point at position within s equals U+000D CR:
        if byte == b'\r' {
            // Step 7.1.1: Append native line ending to result.
            result.extend_from_slice(native_line_ending);
            // Step 7.1.2: Advance position by 1.
            position += 1;
            // Step 7.1.3: If position is not past the end of s and the code point
            // at position within s equals U+000A LF, advance position by 1.
            if position < len && s[position] == b'\n' {
                position += 1;
            }
        }
        // Step 7.2: Otherwise, if the code point at position within s equals U+000A LF:
        else if byte == b'\n' {
            // Advance position by 1 and append native line ending to result.
            position += 1;
            result.extend_from_slice(native_line_ending);
        }

        // Step 7.3: Let token be the result of collecting a sequence of code points
        // that are not equal to U+000A LF or U+000D CR from s given position.
        // Step 7.4: Append token to result.
        result.extend_from_slice(collect_a_sequence_of_code_points(&mut position));
    }

    // Step 8: Return result.
    result
}

/// <https://w3c.github.io/FileAPI/#process-blob-parts>
#[expect(unsafe_code)]
pub(crate) fn process_blob_parts(
    mut blobparts: Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>,
    endings: BlobBinding::EndingType,
) -> Result<Vec<u8>, ()> {
    // Step 1. Let bytes be an empty sequence of bytes.
    let mut bytes = vec![];
    // Step 2. For each blobpart in blobparts:
    for blobpart in &mut blobparts {
        match blobpart {
            // Step 2.1. If blobpart is a USVString, run the following substeps:
            ArrayBufferOrArrayBufferViewOrBlobOrString::String(s) => {
                // Step 2.1.1. Let s be blobpart.
                // Step 2.1.2. If the endings member of options is "native",
                // set s to the result of converting line endings to native of blobpart.
                if endings == BlobBinding::EndingType::Native {
                    let converted = convert_line_endings_to_native(&s.as_bytes());
                    // Step 2.1.3. Append the result of UTF-8 encoding s to bytes.
                    bytes.extend(converted);
                } else {
                    // Step 2.1.3: Append the result of UTF-8 encoding s to bytes.
                    bytes.extend_from_slice(&s.as_bytes());
                }
            },
            // Step 2.2. If element is a BufferSource,
            // get a copy of the bytes held by the buffer source,
            // and append those bytes to bytes.
            ArrayBufferOrArrayBufferViewOrBlobOrString::ArrayBuffer(a) => unsafe {
                let array_bytes = a.as_slice();
                bytes.extend(array_bytes);
            },
            ArrayBufferOrArrayBufferViewOrBlobOrString::ArrayBufferView(a) => unsafe {
                let view_bytes = a.as_slice();
                bytes.extend(view_bytes);
            },
            // Step 2.3. If element is a Blob, append the bytes it represents to bytes.
            ArrayBufferOrArrayBufferViewOrBlobOrString::Blob(b) => {
                let blob_bytes = b.get_bytes().unwrap_or(vec![]);
                bytes.extend(blob_bytes);
            },
        }
    }

    // Step 3. Return bytes.
    Ok(bytes)
}

impl BlobMethods<crate::DomTypeHolder> for Blob {
    // https://w3c.github.io/FileAPI/#constructorBlob
    #[expect(non_snake_case)]
    fn Constructor(
        cx: &mut js::context::JSContext,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        blobParts: Option<Vec<ArrayBufferOrArrayBufferViewOrBlobOrString>>,
        blobPropertyBag: &BlobBinding::BlobPropertyBag,
    ) -> Fallible<DomRoot<Blob>> {
        let bytes: Vec<u8> = match blobParts {
            None => Vec::new(),
            Some(blobparts) => match process_blob_parts(blobparts, blobPropertyBag.endings) {
                Ok(bytes) => bytes,
                Err(_) => return Err(Error::InvalidCharacter(None)),
            },
        };

        let type_string = normalize_type_string(&blobPropertyBag.type_.str());
        let blob_impl = BlobImpl::new_from_bytes(bytes, type_string);

        Ok(Blob::new_with_proto_and_cx(global, proto, blob_impl, cx))
    }

    /// <https://w3c.github.io/FileAPI/#dfn-size>
    fn Size(&self) -> u64 {
        self.global().get_blob_size(&self.blob_id)
    }

    /// <https://w3c.github.io/FileAPI/#dfn-type>
    fn Type(&self) -> DOMString {
        DOMString::from(self.type_string())
    }

    // <https://w3c.github.io/FileAPI/#blob-get-stream>
    fn Stream(&self, cx: &mut js::context::JSContext) -> Fallible<DomRoot<ReadableStream>> {
        self.get_stream(cx)
    }

    /// <https://w3c.github.io/FileAPI/#slice-method-algo>
    fn Slice(
        &self,
        cx: &mut js::context::JSContext,
        start: Option<i64>,
        end: Option<i64>,
        content_type: Option<DOMString>,
    ) -> DomRoot<Blob> {
        let global = self.global();
        let type_string = normalize_type_string(&content_type.unwrap_or_default().str());

        // If our parent is already a sliced blob then we reference the data from the grandparent instead,
        // to keep the blob ancestry chain short.
        let (parent, range) = match *global.get_blob_data(&self.blob_id) {
            BlobData::Sliced(grandparent, parent_range) => {
                let range = RelativePos {
                    start: parent_range.start + start.unwrap_or_default(),
                    end: end.map(|end| end + parent_range.start).or(parent_range.end),
                };
                (grandparent, range)
            },
            _ => (self.blob_id, RelativePos::from_opts(start, end)),
        };

        let blob_impl = BlobImpl::new_sliced(range, parent, type_string);
        Blob::new(&global, blob_impl, CanGc::from_cx(cx))
    }

    /// <https://w3c.github.io/FileAPI/#text-method-algo>
    fn Text(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let global = self.global();
        let p = Promise::new_in_realm(cx);
        let id = self.get_blob_url_id();
        global.read_file_async(
            id,
            p.clone(),
            Box::new(|cx, promise, bytes| match bytes {
                Ok(b) => {
                    let (text, _) = UTF_8.decode_with_bom_removal(&b);
                    let text = DOMString::from(text);
                    promise.resolve_native(&text, CanGc::from_cx(cx));
                },
                Err(e) => {
                    promise.reject_error(e, CanGc::from_cx(cx));
                },
            }),
        );
        p
    }

    /// <https://w3c.github.io/FileAPI/#arraybuffer-method-algo>
    fn ArrayBuffer(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);

        // 1. Let stream be the result of calling get stream on this.
        let stream = self.get_stream(cx);

        // 2. Let reader be the result of getting a reader from stream.
        //    If that threw an exception, return a new promise rejected with that exception.
        let reader = match stream.and_then(|s| s.acquire_default_reader(CanGc::from_cx(cx))) {
            Ok(reader) => reader,
            Err(error) => {
                promise.reject_error(error, CanGc::from_cx(cx));
                return promise;
            },
        };

        // 3. Let promise be the result of reading all bytes from stream with reader.
        let success_promise = promise.clone();
        let failure_promise = promise.clone();
        reader.read_all_bytes(
            cx,
            Rc::new(move |cx, bytes| {
                rooted!(&in(cx) let mut js_object = ptr::null_mut::<JSObject>());
                // 4. Return the result of transforming promise by a fulfillment handler that returns a new
                //    [ArrayBuffer]
                let array_buffer = create_buffer_source::<ArrayBufferU8>(
                    cx.into(),
                    bytes,
                    js_object.handle_mut(),
                    CanGc::from_cx(cx),
                )
                .expect("Converting input to ArrayBufferU8 should never fail");
                success_promise.resolve_native(&array_buffer, CanGc::from_cx(cx));
            }),
            Rc::new(move |cx, value| {
                failure_promise.reject(cx.into(), value, CanGc::from_cx(cx));
            }),
        );

        promise
    }

    /// <https://w3c.github.io/FileAPI/#dom-blob-bytes>
    fn Bytes(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let p = Promise::new_in_realm(cx);

        // 1. Let stream be the result of calling get stream on this.
        let stream = self.get_stream(cx);

        // 2. Let reader be the result of getting a reader from stream.
        //    If that threw an exception, return a new promise rejected with that exception.
        let reader = match stream.and_then(|s| s.acquire_default_reader(CanGc::from_cx(cx))) {
            Ok(r) => r,
            Err(e) => {
                p.reject_error(e, CanGc::from_cx(cx));
                return p;
            },
        };

        // 3. Let promise be the result of reading all bytes from stream with reader.
        let p_success = p.clone();
        let p_failure = p.clone();
        reader.read_all_bytes(
            cx,
            Rc::new(move |cx, bytes| {
                rooted!(&in(cx) let mut js_object = ptr::null_mut::<JSObject>());
                let arr = create_buffer_source::<Uint8>(
                    cx.into(),
                    bytes,
                    js_object.handle_mut(),
                    CanGc::from_cx(cx),
                )
                .expect("Converting input to uint8 array should never fail");
                p_success.resolve_native(&arr, CanGc::from_cx(cx));
            }),
            Rc::new(move |cx, v| {
                p_failure.reject(cx.into(), v, CanGc::from_cx(cx));
            }),
        );
        p
    }
}

/// Get the normalized, MIME-parsable type string
/// <https://w3c.github.io/FileAPI/#dfn-type>
/// XXX: We will relax the restriction here,
/// since the spec has some problem over this part.
/// see <https://github.com/w3c/FileAPI/issues/43>
pub(crate) fn normalize_type_string(s: &str) -> String {
    if is_ascii_printable(s) {
        s.to_ascii_lowercase()
        // match s_lower.parse() as Result<Mime, ()> {
        // Ok(_) => s_lower,
        // Err(_) => "".to_string()
    } else {
        "".to_string()
    }
}

fn is_ascii_printable(string: &str) -> bool {
    // Step 5.1 in Sec 5.1 of File API spec
    // <https://w3c.github.io/FileAPI/#constructorBlob>
    string.chars().all(|c| ('\x20'..='\x7E').contains(&c))
}
