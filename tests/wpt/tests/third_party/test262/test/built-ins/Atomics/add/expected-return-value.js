// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.add
description: >
  Atomics.add returns the value that existed at the
  index prior to the operation.
info: |
  Atomics.add( typedArray, index, value )

  1. Return ? AtomicReadModifyWrite(typedArray, index, value, add).

  AtomicReadModifyWrite( typedArray, index, value, op )

  ...
  9. Return GetModifySetValueInBuffer(buffer, indexedPosition,
                                      elementType, v, op).


  GetModifySetValueInBuffer( arrayBuffer,
    byteIndex, type, value, op [ , isLittleEndian ] )

  ...
  16. Return RawBytesToNumber(type, rawBytesRead, isLittleEndian).

features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);
const newValue = 0b00000001000000001000000010000001;

assert.sameValue(Atomics.add(i32a, 0, newValue), 0, 'Atomics.add(i32a, 0, newValue) returns 0');
assert.sameValue(i32a[0], newValue, 'The value of i32a[0] equals the value of `newValue` (0b00000001000000001000000010000001)');
