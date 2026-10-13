// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: Throws a TypeError exception when `this` is a SharedArrayBuffer
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
features: [SharedArrayBuffer, ArrayBuffer, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var fn = ArrayBuffer.prototype.sliceToImmutable;
assert.sameValue(typeof fn, "function", "Method must exist.");

var calls = [];
var start = {
  valueOf() {
    calls.push("start.valueOf");
    return 0;
  }
};
var end = {
  valueOf() {
    calls.push("end.valueOf");
    return 1;
  }
};

var sab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  fn.call(sab, start, end);
});
assert.compareArray(calls, [],
  "Must verify non-SharedArrayBuffer receiver before reading arguments.");
