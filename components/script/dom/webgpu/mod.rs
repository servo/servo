/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::error::{Error, Fallible};

use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer as BufferSource;

pub(crate) mod gpu;
pub(crate) mod gpuadapter;
pub(crate) mod gpuadapterinfo;
pub(crate) mod gpubindgroup;
pub(crate) mod gpubindgrouplayout;
pub(crate) mod gpubuffer;
pub(crate) mod gpubufferusage;
pub(crate) mod gpucanvascontext;
pub(crate) mod gpucolorwrite;
pub(crate) mod gpucommandbuffer;
pub(crate) mod gpucommandencoder;
pub(crate) mod gpucompilationinfo;
pub(crate) mod gpucompilationmessage;
pub(crate) mod gpucomputepassencoder;
pub(crate) mod gpucomputepipeline;
pub(crate) mod gpuconvert;
pub(crate) mod gpudevice;
pub(crate) mod gpudevicelostinfo;
pub(crate) mod gpuerror;
pub(crate) mod gpuinternalerror;
pub(crate) mod gpumapmode;
pub(crate) mod gpuoutofmemoryerror;
pub(crate) mod gpupipelineerror;
#[expect(dead_code)]
pub(crate) mod gpupipelinelayout;
pub(crate) mod gpuqueryset;
pub(crate) mod gpuqueue;
pub(crate) mod gpurenderbundle;
pub(crate) mod gpurenderbundleencoder;
pub(crate) mod gpurenderpassencoder;
pub(crate) mod gpurenderpipeline;
pub(crate) mod gpusampler;
pub(crate) mod gpushadermodule;
pub(crate) mod gpushaderstage;
pub(crate) mod gpusupportedfeatures;
pub(crate) mod gpusupportedlimits;
pub(crate) mod gputexture;
pub(crate) mod gputextureusage;
pub(crate) mod gputextureview;
pub(crate) mod gpuuncapturederrorevent;
pub(crate) mod gpuvalidationerror;
#[expect(dead_code)]
pub(crate) mod identityhub;
pub(crate) mod wgsllanguagefeatures;

/// Validates and slices a buffer source according to WebGPU spec.
///
/// This is described twice in the spec:
/// <https://www.w3.org/TR/webgpu/#dom-gpubindingcommandsmixin-setimmediates>
/// <https://www.w3.org/TR/webgpu/#dom-gpuqueue-writebuffer>
pub(super) fn validate_and_slice_buffer_source(
    data: &BufferSource,
    data_offset: u64,
    data_size: Option<u64>,
) -> Fallible<&[u8]> {
    // Step 1: If data is an ArrayBuffer or DataView, let the element type be "byte". Otherwise, data is a TypedArray; let the element type be the type of the TypedArray.
    let sizeof_element: u64 = match data {
        BufferSource::ArrayBufferView(d) => d.get_array_type().byte_size().unwrap_or(1) as u64,
        BufferSource::ArrayBuffer(_) => 1,
    };
    // Step 2: Let dataElementCount be the size of data, in elements.
    #[expect(unsafe_code)]
    let data_slice = unsafe {
        match data {
            BufferSource::ArrayBufferView(d) => d.as_slice(),
            BufferSource::ArrayBuffer(d) => d.as_slice(),
        }
    };
    let data_element_count = data_slice.len() as u64 / sizeof_element;
    // Step 3: If dataSize is missing, let contentsSize be dataElementCount − dataOffset. Otherwise, let contentsSize be dataSize.
    let content_size = if let Some(data_size) = data_size {
        data_size
    } else {
        data_element_count.checked_sub(data_offset).ok_or_else(|| {
            Error::Operation(Some(format!("data offset {data_offset} is out of bounds")))
        })?
    };
    // Step 4: If any of the following conditions are unsatisfied, throw an OperationError and return.
    // contentsSize ≥ 0 is implied by the type
    // dataOffset + contentsSize ≤ dataSize.
    #[allow(clippy::nonminimal_bool)]
    // expect does not work correctly due to lint being available above MSRV
    if !(data_offset.checked_add(content_size).ok_or_else(|| {
        Error::Operation(Some(format!(
            "data offset {data_offset} + contentSize {content_size} is too big"
        )))
    })? <= data_element_count)
    {
        return Err(Error::Operation(Some(format!(
            "data offset {data_offset} + content size {content_size} is bigger then content size {data_element_count}"
        ))));
    }
    // contentsSize, converted to bytes, is a multiple of 4 bytes.
    if !(content_size
        .checked_mul(sizeof_element)
        .ok_or_else(|| {
            Error::Operation(Some(format!(
                "content size {content_size} * sizeof element {sizeof_element} is too big"
            )))
        })?
        .is_multiple_of(4))
    {
        return Err(Error::Operation(Some(format!(
            "content size {content_size} * sizeof element {sizeof_element} is not a multiple of 4"
        ))));
    }
    // Step 5: Let dataContents be a copy of the bytes held by the buffer source data.
    // Step 6: Let contents be the contentsSize elements of dataContents starting at an offset of dataOffset elements.
    // Step 7: Let contentsBytes be contentsSize converted to bytes.
    Ok(
        &data_slice[(data_offset as usize) * (sizeof_element as usize)..
            ((data_offset + content_size) as usize) * (sizeof_element as usize)],
    )
}
