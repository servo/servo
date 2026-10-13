// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-arraybuffer.prototype.immutable
description: Throws a TypeError exception when `this` is not an object
info: |
  get ArrayBuffer.prototype.immutable
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
features: [ArrayBuffer, immutable-arraybuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, "immutable"
).get;

assert.sameValue(typeof getter, "function", "Getter must exist.");

var badReceivers = [
  ["undefined", undefined],
  ["null", null],
  ["number", 42],
  ["string", "1"],
  ["true", true],
  ["false", false],
  typeof Symbol === "undefined" ? undefined : ["unique symbol", Symbol("s")],
  typeof Symbol === "undefined" || !Symbol.for ? undefined : ["registered symbol", Symbol.for("s")],
  typeof BigInt === "undefined" ? undefined : ["bigint", BigInt(1)]
];

for (var i = 0; i < badReceivers.length; i++) {
  if (!badReceivers[i]) continue;
  var label = badReceivers[i][0];
  var value = badReceivers[i][1];
  assert.throws(TypeError, function() {
    getter.call(value);
  }, label);
}
