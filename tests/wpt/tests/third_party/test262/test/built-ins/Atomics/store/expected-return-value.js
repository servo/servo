// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.store
description: >
  Atomics.store returns the newly stored value
info: |
  Atomics.store( typedArray, index, value )

  ...
  3. Let v be ? ToInteger(value).
  ...
  9. Perform SetValueInBuffer(buffer, indexedPosition,
                              elementType, v, true, "SeqCst").
  10. Return v.

features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);
const update = 0b00000001000000001000000010000001;

assert.sameValue(
  Atomics.store(i32a, 0, update),
  update,
  'Atomics.store(i32a, 0, update) returns the value of `update` (0b00000001000000001000000010000001)'
);
assert.sameValue(
  i32a[0],
  update,
  'The value of i32a[0] equals the value of `update` (0b00000001000000001000000010000001)'
);
