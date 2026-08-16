// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: >
  Throws a TypeError exception when `this` does not have a [[ArrayBufferData]]
  internal slot
info: |
  ArrayBuffer.prototype.transferToImmutable ( [ newLength ] )
  1. Let O be the this value.
  2. Return ? ArrayBufferCopyAndDetach(O, newLength, ~immutable~).

  ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )
  1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
features: [DataView, Int8Array, ArrayBuffer, immutable-arraybuffer]
includes: [compareArray.js]
---*/

var fn = ArrayBuffer.prototype.transferToImmutable;
assert.sameValue(typeof fn, "function", "Method must exist.");

var calls = [];
var newLength = {
  valueOf() {
    calls.push("newLength.valueOf");
    return 1;
  }
};

var badReceivers = [
  ["plain object", {}],
  ["array", []],
  ["function", function(){}],
  ["ArrayBuffer.prototype", ArrayBuffer.prototype],
  ["TypedArray", new Int8Array(8)],
  ["DataView", new DataView(new ArrayBuffer(8), 0)]
];

for (var i = 0; i < badReceivers.length; i++) {
  var label = badReceivers[i][0];
  var value = badReceivers[i][1];
  calls = [];
  assert.throws(TypeError, function() {
    fn.call(value, newLength);
  }, label);
  assert.compareArray(calls, [],
    "[" + label + " receiver] Must verify internal slots before reading arguments.");
}

calls = [];
assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToImmutable(newLength);
}, "invoked as prototype method");
assert.compareArray(calls, [],
  "[invoked as prototype method] Must verify internal slots before reading arguments.");

calls = [];
assert.throws(TypeError, function() {
  fn(newLength);
}, "invoked as function");
assert.compareArray(calls, [],
  "[invoked as function] Must verify internal slots before reading arguments.");
