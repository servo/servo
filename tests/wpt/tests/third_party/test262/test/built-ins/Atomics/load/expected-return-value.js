// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.load
description: >
  Atomics.load returns the value that existed at the
  index prior to the operation.
info: |
  Atomics.load( typedArray, index, value )

  1. Return ? AtomicLoad(typedArray, index).

  AtomicLoad( typedArray, index )

  1. Let buffer be ? ValidateSharedIntegerTypedArray(typedArray).
  2. Let i be ? ValidateAtomicAccess(typedArray, index).
  3. Let arrayTypeName be typedArray.[[TypedArrayName]].
  4. Let elementSize be the Number value of the Element Size value
      specified in Table 56 for arrayTypeName.
  5. Let elementType be the String value of the Element Type value
      in Table 56 for arrayTypeName.
  6. Let offset be typedArray.[[ByteOffset]].
  7. Let indexedPosition be (i × elementSize) + offset.
  8. Return GetValueFromBuffer(buffer, indexedPosition, elementType,
      true, "SeqCst").

features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);
const update = 0b00000001000000001000000010000001;

assert.sameValue(Atomics.load(i32a, 0), 0, 'Atomics.load(i32a, 0) returns 0');

i32a[0] = update;

assert.sameValue(
  Atomics.load(i32a, 0),
  update,
  'Atomics.load(i32a, 0) returns the value of `update` (0b00000001000000001000000010000001)'
);

