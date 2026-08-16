// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: Throws a TypeError exception when `this` is not an object
info: |
  ArrayBuffer.prototype.transferToImmutable ( [ newLength ] )
  1. Let O be the this value.
  2. Return ? ArrayBufferCopyAndDetach(O, newLength, ~immutable~).

  ArrayBufferCopyAndDetach ( arrayBuffer, newLength, preserveResizability )
  1. Perform ? RequireInternalSlot(arrayBuffer, [[ArrayBufferData]]).
features: [ArrayBuffer, immutable-arraybuffer]
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
  ["undefined", undefined],
  ["null", null],
  ["number", 42],
  ["string", "1"],
  ["true", true],
  ["false", false],
  typeof Symbol === "undefined" ? undefined : ["unique symbol", Symbol("1")],
  typeof Symbol === "undefined" || !Symbol.for ? undefined : ["registered symbol", Symbol.for("1")],
  typeof BigInt === "undefined" ? undefined : ["bigint", BigInt(1)]
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
