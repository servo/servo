// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfer
description: Throws a TypeError if `this` is immutable.
info: |
  ArrayBuffer.prototype.transfer ( [ newLength ] )
  1. Let O be the this value.
  2. Return ? ArrayBufferCopyAndDetach(O, newLength, ~preserve-resizability~).

  ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )
  1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
  2. If IsSharedArrayBuffer(arrayBuffer) is true, throw a TypeError exception.
  3. If newLength is undefined, then
     a. Let newByteLength be arrayBuffer.[[ArrayBufferByteLength]].
  4. Else,
     a. Let newByteLength be ? ToIndex(newLength).
  5. If IsDetachedBuffer(arrayBuffer) is true, throw a TypeError exception.
  6. If IsImmutableBuffer(arrayBuffer) is true, throw a TypeError exception.
features: [arraybuffer-transfer, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var calls = [];

var ab = (new ArrayBuffer(4)).transferToImmutable();
assert.throws(TypeError, function() {
  ab.transfer();
});
assert.throws(TypeError, function() {
  ab.transfer({
    valueOf() {
      calls.push("newLength.valueOf");
      return 1;
    }
  });
});
assert.compareArray(calls, ["newLength.valueOf"],
  "Must read newLength before verifying mutability.");
