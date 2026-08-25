// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: >
  Throws a TypeError exception when `this` does not have a [[ArrayBufferData]]
  internal slot
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
features: [DataView, Int8Array, ArrayBuffer, immutable-arraybuffer]
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
    fn.call(value, start, end);
  }, label);
  assert.compareArray(calls, [],
    "[" + label + " receiver] Must verify internal slots before reading arguments.");
}

calls = [];
assert.throws(TypeError, function() {
  ArrayBuffer.prototype.sliceToImmutable(start, end);
}, "invoked as prototype method");
assert.compareArray(calls, [],
  "[invoked as prototype method] Must verify internal slots before reading arguments.");

calls = [];
assert.throws(TypeError, function() {
  fn(start, end);
}, "invoked as function");
assert.compareArray(calls, [],
  "[invoked as function] Must verify internal slots before reading arguments.");
