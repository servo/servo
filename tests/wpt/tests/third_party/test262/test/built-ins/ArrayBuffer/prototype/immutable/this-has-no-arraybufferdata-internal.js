// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-arraybuffer.prototype.immutable
description: >
  Throws a TypeError exception when `this` does not have a [[ArrayBufferData]]
  internal slot
info: |
  get ArrayBuffer.prototype.immutable
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
features: [DataView, Int8Array, ArrayBuffer, immutable-arraybuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, "immutable"
).get;

assert.sameValue(typeof getter, "function", "Getter must exist.");

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
  assert.throws(TypeError, function() {
    getter.call(value);
  }, label);
}

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.immutable;
}, "invoked as prototype property access");

assert.throws(TypeError, function() {
  getter();
}, "invoked as function");
