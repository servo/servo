// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.values
description: >
  Calling next on an out-of-bounds typedarray throws no error when iterator exhausted.
features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(3, {maxByteLength: 5});
let ta = new Int8Array(rab, 1);

// Ensure the TypedArray is correctly initialised.
assert.sameValue(ta.length, 2);
assert.sameValue(ta.byteOffset, 1);

ta[0] = 11;
ta[1] = 22;

let it = ta.values();
let r;

// Fetch the first value.
r = it.next();
assert.sameValue(r.done, false);
assert.sameValue(r.value, 11);

// Fetch the second value.
r = it.next();
assert.sameValue(r.done, false);
assert.sameValue(r.value, 22);

// Iterator is now exhausted.
r = it.next();
assert.sameValue(r.done, true);
assert.sameValue(r.value, undefined);

// Resize buffer to zero.
rab.resize(0);

// TypedArray is now out-of-bounds.
assert.sameValue(ta.length, 0);
assert.sameValue(ta.byteOffset, 0);

// Calling next doesn't throw an error.
r = it.next();
assert.sameValue(r.done, true);
assert.sameValue(r.value, undefined);
