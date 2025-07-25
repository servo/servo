# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# pyrefly: ignore

from WebIDL import IDLType

# WARN: This maps the value defined at:
# https://github.com/servo/servo/blob/main/third_party/WebIDL/WebIDL.py#L2525
# Be aware that missing Tags will break the build.
class Tags:
    # Integer types
    int8 = IDLType.Tags.int8
    uint8 = IDLType.Tags.uint8
    int16 = IDLType.Tags.int16
    uint16 = IDLType.Tags.uint16
    int32 = IDLType.Tags.int32
    uint32 = IDLType.Tags.uint32
    int64 = IDLType.Tags.int64
    uint64 = IDLType.Tags.uint64

    # Floating-point types
    unrestricted_float = IDLType.Tags.unrestricted_float
    float = IDLType.Tags.float
    unrestricted_double = IDLType.Tags.unrestricted_double
    double = IDLType.Tags.double

    # Boolean
    bool = IDLType.Tags.bool

    # String types
    domstring = IDLType.Tags.domstring
    bytestring = IDLType.Tags.bytestring
    usvstring = IDLType.Tags.usvstring
    utf8string = IDLType.Tags.utf8string
    jsstring = IDLType.Tags.jsstring

    # Special types
    any = IDLType.Tags.any
    undefined = IDLType.Tags.undefined
    object = IDLType.Tags.object

    # Typed arrays
    int8array = IDLType.Tags.int8array
    uint8array = IDLType.Tags.uint8array
    int16array = IDLType.Tags.int16array
    uint16array = IDLType.Tags.uint16array
    int32array = IDLType.Tags.int32array
    uint32array = IDLType.Tags.uint32array
    float32array = IDLType.Tags.float32array
    float64array = IDLType.Tags.float64array
    uint8clampedarray = IDLType.Tags.uint8clampedarray

    # Buffer and views
    arrayBuffer = IDLType.Tags.arrayBuffer
    arrayBufferView = IDLType.Tags.arrayBufferView

    # Structured types
    dictionary = IDLType.Tags.dictionary
    enum = IDLType.Tags.enum
    callback = IDLType.Tags.callback

    # Composite/Generic types
    union = IDLType.Tags.union
    sequence = IDLType.Tags.sequence
    record = IDLType.Tags.record
    promise = IDLType.Tags.promise
    observablearray = IDLType.Tags.observablearray

    # Interface reference
    interface = IDLType.Tags.interface

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
