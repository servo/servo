// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: Throws a TypeError exception when `this` is not an object
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
features: [ArrayBuffer, immutable-arraybuffer]
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
    fn.call(value, start, end);
  }, label);
  assert.compareArray(calls, [],
    "[" + label + " receiver] Must verify internal slots before reading arguments.");
}
