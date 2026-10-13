// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: >
  Throws a TypeError exception when `this` is already detached
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
features: [immutable-arraybuffer]
includes: [compareArray.js, detachArrayBuffer.js]
---*/

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

var detached = new ArrayBuffer(8);
$DETACHBUFFER(detached);
assert.throws(TypeError, function() {
  detached.sliceToImmutable(start, end);
});
assert.compareArray(calls, [],
  "Must verify non-detachment before reading arguments.");
