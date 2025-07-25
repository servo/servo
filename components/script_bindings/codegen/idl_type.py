# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# pyrefly: ignore

from WebIDL import IDLType

class Tags:
    bool = IDLType.Tags.bool,
    int8 = IDLType.Tags.int8,
    int16 = IDLType.Tags.int16,
    int32 = IDLType.Tags.int32,
    int64 = IDLType.Tags.int64,
    uint8 = IDLType.Tags.uint8,
    uint16 = IDLType.Tags.uint16,
    uint32 = IDLType.Tags.uint32,
    uint64 = IDLType.Tags.uint64,
    unrestricted_float = IDLType.Tags.unrestricted_float,
    float = IDLType.Tags.float,
    unrestricted_double = IDLType.Tags.unrestricted_double,
    double = IDLType.Tags.double,
    int8array = IDLType.Tags.int8array,
    uint8array = IDLType.Tags.uint8array,
    int16array = IDLType.Tags.int16array,
    uint16array = IDLType.Tags.uint16array,
    int32array = IDLType.Tags.int32array,
    uint32array = IDLType.Tags.uint32array,
    float32array = IDLType.Tags.float32array,
    float64array = IDLType.Tags.float64array,
    arrayBuffer = IDLType.Tags.arrayBuffer,
    arrayBufferView = IDLType.Tags.arrayBufferView,
    uint8clampedarray = IDLType.Tags.uint8clampedarray,
    usvstring = IDLType.Tags.usvstring
    domstring = IDLType.Tags.domstring
    bytestring = IDLType.Tags.bytestring

builtinNames = {
    IDLType.Tags.bool: 'bool',
    IDLType.Tags.int8: 'i8',
    IDLType.Tags.int16: 'i16',
    IDLType.Tags.int32: 'i32',
    IDLType.Tags.int64: 'i64',
    IDLType.Tags.uint8: 'u8',
    IDLType.Tags.uint16: 'u16',
    IDLType.Tags.uint32: 'u32',
    IDLType.Tags.uint64: 'u64',
    IDLType.Tags.unrestricted_float: 'f32',
    IDLType.Tags.float: 'Finite<f32>',
    IDLType.Tags.unrestricted_double: 'f64',
    IDLType.Tags.double: 'Finite<f64>',
    IDLType.Tags.int8array: 'Int8Array',
    IDLType.Tags.uint8array: 'Uint8Array',
    IDLType.Tags.int16array: 'Int16Array',
    IDLType.Tags.uint16array: 'Uint16Array',
    IDLType.Tags.int32array: 'Int32Array',
    IDLType.Tags.uint32array: 'Uint32Array',
    IDLType.Tags.float32array: 'Float32Array',
    IDLType.Tags.float64array: 'Float64Array',
    IDLType.Tags.arrayBuffer: 'ArrayBuffer',
    IDLType.Tags.arrayBufferView: 'ArrayBufferView',
    IDLType.Tags.uint8clampedarray: 'Uint8ClampedArray',
}

numericTags = [
    IDLType.Tags.int8, IDLType.Tags.uint8,
    IDLType.Tags.int16, IDLType.Tags.uint16,
    IDLType.Tags.int32, IDLType.Tags.uint32,
    IDLType.Tags.int64, IDLType.Tags.uint64,
    IDLType.Tags.unrestricted_float,
    IDLType.Tags.unrestricted_double
]
