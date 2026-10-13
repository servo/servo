// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: Throws a TypeError exception when `this` is a SharedArrayBuffer
info: |
  ArrayBuffer.prototype.transferToImmutable ( [ newLength ] )
  1. Let O be the this value.
  2. Return ? ArrayBufferCopyAndDetach(O, newLength, ~immutable~).

  ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )
  1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
  2. If IsSharedArrayBuffer(arrayBuffer) is true, throw a TypeError exception.
features: [SharedArrayBuffer, ArrayBuffer, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var fn = ArrayBuffer.prototype.transferToImmutable;
assert.sameValue(typeof fn, "function", "Method must exist.");

var calls = [];

var sab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  fn.call(sab, {
    valueOf() {
      calls.push("newLength.valueOf");
      return 1;
    }
  });
});
assert.compareArray(calls, [],
  "Must verify non-SharedArrayBuffer receiver before reading arguments.");
