// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.exchange
description: >
  Atomics.and returns the value that existed at the
  index prior to the operation.
info: |
  Atomics.exchange( typedArray, index, value )

  1. Return ? AtomicReadModifyWrite(typedArray, index, value, second).

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

assert.sameValue(
  Atomics.exchange(i32a, 0, update),
  0,
  'Atomics.exchange(i32a, 0, update) returns 0'
);
assert.sameValue(
  i32a[0],
  update,
  'The value of i32a[0] equals the value of `update` (0b00000001000000001000000010000001)'
);
