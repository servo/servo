// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.sub
description: >
  Atomics.and returns the value that existed at the
  index prior to the operation.
info: |
  Atomics.sub( typedArray, index, value )

  1. Return ? AtomicReadModifyWrite(typedArray, index, value, subtract).

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
const update = 0b00000001000000001000000010000001;

i32a[0] = update;

assert.sameValue(
  Atomics.sub(i32a, 0, update),
  update,
  'Atomics.sub(i32a, 0, update) returns the value of `update` (0b00000001000000001000000010000001)'
);
assert.sameValue(i32a[0], 0, 'The value of i32a[0] is 0');
