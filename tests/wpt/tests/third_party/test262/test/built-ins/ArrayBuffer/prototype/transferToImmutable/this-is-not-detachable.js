// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: >
  Throws a TypeError exception when `this` is already detached or immutable
info: |
  ArrayBuffer.prototype.transferToImmutable ( [ newLength ] )
  1. Let O be the this value.
  2. Return ? ArrayBufferCopyAndDetach(O, newLength, ~immutable~).

  ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )
  1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
  2. If IsSharedArrayBuffer(arrayBuffer) is true, throw a TypeError exception.
  3. If newLength is undefined, then
     a. Let newByteLength be arrayBuffer.[[ArrayBufferByteLength]].
  4. Else,
     a. Let newByteLength be ? ToIndex(newLength).
  5. If IsDetachedBuffer(arrayBuffer) is true, throw a TypeError exception.
  6. If IsImmutableBuffer(arrayBuffer) is true, throw a TypeError exception.
features: [immutable-arraybuffer]
includes: [compareArray.js, detachArrayBuffer.js]
---*/

var calls = [];
var newLength = {
  valueOf() {
    calls.push("newLength.valueOf");
    return 1;
  }
};

var detached = new ArrayBuffer(8);
$DETACHBUFFER(detached);
var immutable = (new ArrayBuffer(8)).transferToImmutable();

var badReceivers = [
  ["detached ArrayBuffer", detached],
  ["immutable ArrayBuffer", immutable]
];

for (var i = 0; i < badReceivers.length; i++) {
  var label = badReceivers[i][0];
  var value = badReceivers[i][1];
  calls = [];
  assert.throws(TypeError, function() {
    value.transferToImmutable(newLength);
  }, label);
  assert.compareArray(calls, ["newLength.valueOf"],
    "[" + label + "] Must read arguments before verifying detachability.");
}

calls = [];
var becomesDetached = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  becomesDetached.transferToImmutable({
    valueOf() {
      calls.push("newLength.valueOf");
      $DETACHBUFFER(becomesDetached);
      return 1;
    }
  });
}, "becomes detached");
assert.compareArray(calls, ["newLength.valueOf"],
  "[becomes detached] Must read arguments before verifying detachability.");
