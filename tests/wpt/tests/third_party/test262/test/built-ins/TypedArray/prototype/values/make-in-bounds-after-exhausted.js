// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.values
description: >
  Iterator is still exhausted when typedarray is changed to in-bounds.
features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(3, {maxByteLength: 5});
let ta = new Int8Array(rab);

// Ensure the TypedArray is correctly initialised.
assert.sameValue(ta.length, 3);
assert.sameValue(ta.byteOffset, 0);

ta[0] = 11;
ta[1] = 22;
ta[2] = 33;

let it = ta.values();
let r;

// Fetch the first value.
r = it.next();
assert.sameValue(r.done, false);
assert.sameValue(r.value, 11);

// Resize buffer to zero.
rab.resize(0);

// TypedArray is now out-of-bounds.
assert.sameValue(ta.length, 0);
assert.sameValue(ta.byteOffset, 0);

// Resize buffer to zero.
rab.resize(0);

// Attempt to fetch the next value. This exhausts the iterator.
r = it.next();
assert.sameValue(r.done, true);
assert.sameValue(r.value, undefined);

// Resize buffer so the typed array is again in-bounds.
rab.resize(5);

// TypedArray is now in-bounds.
assert.sameValue(ta.length, 5);
assert.sameValue(ta.byteOffset, 0);

// Attempt to fetch another value from an already exhausted iterator.
r = it.next();
assert.sameValue(r.done, true);
assert.sameValue(r.value, undefined);
