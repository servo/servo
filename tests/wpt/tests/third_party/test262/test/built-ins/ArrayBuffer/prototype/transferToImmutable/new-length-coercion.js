// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.transfertoimmutable
description: Requested length is coerced to an integer and checked for validity
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

  ToIndex ( value )
  1. Let integer be ? ToIntegerOrInfinity(value).
  2. If integer is not in the inclusive interval from 0 to 2**53 - 1, throw a RangeError exception.
  3. Return integer.

  ToIntegerOrInfinity ( argument )
  1. Let number be ? ToNumber(argument).
  2. If number is one of NaN, +0𝔽, or -0𝔽, return 0.
  3. If number is +∞𝔽, return +∞.
  4. If number is -∞𝔽, return -∞.
  5. Return truncate(ℝ(number)).
features: [immutable-arraybuffer]
includes: [compareArray.js]
---*/

var ab = new ArrayBuffer(8);
assert.sameValue(ab.transferToImmutable().byteLength, 8,
  "Must default to receiver byteLength.");
ab = new ArrayBuffer(8);
assert.sameValue(ab.transferToImmutable(undefined).byteLength, 8,
  "Must default undefined to receiver byteLength.");

var goodLengths = [
  // Unmodified integral numbers
  [0, 0],
  [1, 1],
  [10, 10],
  // Truncated integral numbers
  [0.9, 0],
  [1.9, 1],
  [-0.9, 0],
  // Coerced to integral numbers
  [-0, 0],
  [null, 0],
  [false, 0],
  [true, 1],
  ["", 0],
  ["8", 8],
  ["+9", 9],
  ["10e0", 10],
  ["+1.1E+1", 11],
  ["+.12e2", 12],
  ["130e-1", 13],
  ["0b1110", 14],
  ["0XF", 15],
  ["0xf", 15],
  ["0o20", 16],
  // Coerced to NaN and mapped to 0
  [NaN, 0],
  ["7up", 0],
  ["1_0", 0],
  ["0x00_ff", 0]
];

for (var i = 0; i < goodLengths.length; i++) {
  var rawLength = goodLengths[i][0];
  var intLength = goodLengths[i][1];
  var ab = new ArrayBuffer(8);
  assert.sameValue(ab.transferToImmutable(rawLength).byteLength, intLength,
    "transferToImmutable(" + formatSimpleValue(rawLength) + ").byteLength");
}

var whitespace = "\t\v\f\uFEFF\u3000\n\r\u2028\u2029";
for (var i = 0; i < goodLengths.length; i++) {
  var rawLength = goodLengths[i][0];
  var intLength = goodLengths[i][1];
  if (typeof rawLength === "number") {
    rawLength = isNegativeZero(rawLength) ? "-0" : String(rawLength);
  } else if (typeof rawLength !== "string") {
    continue;
  }
  var paddedLength = whitespace + rawLength + whitespace;
  var ab = new ArrayBuffer(8);
  assert.sameValue(ab.transferToImmutable(paddedLength).byteLength, intLength,
    "transferToImmutable(" + formatSimpleValue(paddedLength) + ").byteLength");
}

for (var i = 0; i < goodLengths.length; i++) {
  var rawLength = goodLengths[i][0];
  var intLength = goodLengths[i][1];
  var calls = [];
  var badValueOf = false;
  var badToString = false;
  var objLength = {
    valueOf() {
      calls.push("valueOf");
      return badValueOf ? {} : rawLength;
    },
    toString() {
      calls.push("toString");
      return badToString ? {} : rawLength;
    }
  };

  var ab = new ArrayBuffer(8);
  assert.sameValue(ab.transferToImmutable(objLength).byteLength, intLength,
    "transferToImmutable({ valueOf: () => " + formatSimpleValue(rawLength) + " }).byteLength");
  assert.compareArray(calls, ["valueOf"],
    "transferToImmutable({ valueOf: () => " + formatSimpleValue(rawLength) + " }) calls");

  badValueOf = true;
  calls = [];
  ab = new ArrayBuffer(8);
  assert.sameValue(ab.transferToImmutable(objLength).byteLength, intLength,
    "transferToImmutable({ toString: () => " + formatSimpleValue(rawLength) + " }).byteLength");
  assert.compareArray(calls, ["valueOf", "toString"],
    "transferToImmutable({ toString: () => " + formatSimpleValue(rawLength) + " }) calls");

  badToString = true;
  if (typeof Symbol === undefined || !Symbol.toPrimitive) continue;
  calls = [];
  objLength[Symbol.toPrimitive] = function(hint) {
    calls.push("Symbol.toPrimitive(" + hint + ")");
    return rawLength;
  };
  ab = new ArrayBuffer(8);
  assert.sameValue(ab.transferToImmutable(objLength).byteLength, intLength,
    "transferToImmutable({ [Symbol.toPrimitive]: () => " + formatSimpleValue(rawLength) + " }).byteLength");
  assert.compareArray(calls, ["Symbol.toPrimitive(number)"],
    "transferToImmutable({ [Symbol.toPrimitive]: () => " + formatSimpleValue(rawLength) + " }) calls");
}

var badLengths = [
  // Out of range numbers
  [-1, RangeError],
  [9007199254740992, RangeError], // Math.pow(2, 53) = 9007199254740992
  [Infinity, RangeError],
  [-Infinity, RangeError],
  // non-numbers
  typeof Symbol === undefined ? undefined : [Symbol("1"), TypeError],
  typeof Symbol === undefined || !Symbol.for ? undefined : [Symbol.for("1"), TypeError],
  typeof BigInt === undefined ? undefined : [BigInt(1), TypeError],
];

for (var i = 0; i < badLengths.length; i++) {
  if (!badLengths[i]) continue;
  var rawLength = badLengths[i][0];
  var expectedErr = badLengths[i][1];
  var ab = new ArrayBuffer(8);
  assert.throws(expectedErr, function() {
    ab.transferToImmutable(rawLength);
  }, "transferToImmutable(" + formatSimpleValue(rawLength) + ")");
}

for (var i = 0; i < badLengths.length; i++) {
  if (!badLengths[i]) continue;
  var rawLength = badLengths[i][0];
  var expectedErr = badLengths[i][1];
  if (typeof rawLength !== "number") continue;
  var paddedLength = whitespace + rawLength + whitespace;
  var ab = new ArrayBuffer(8);
  assert.throws(expectedErr, function() {
    ab.transferToImmutable(paddedLength);
  }, "transferToImmutable(" + formatSimpleValue(paddedLength) + ")");
}

for (var i = 0; i < badLengths.length; i++) {
  if (!badLengths[i]) continue;
  var rawLength = badLengths[i][0];
  var expectedErr = badLengths[i][1];
  var calls = [];
  var badValueOf = false;
  var badToString = false;
  var objLength = {
    valueOf() {
      calls.push("valueOf");
      return badValueOf ? {} : rawLength;
    },
    toString() {
      calls.push("toString");
      return badToString ? {} : rawLength;
    }
  };

  var ab = new ArrayBuffer(8);
  assert.throws(expectedErr, function() {
    ab.transferToImmutable(objLength);
  }, "transferToImmutable({ valueOf: () => " + formatSimpleValue(rawLength) + " })");
  assert.compareArray(calls, ["valueOf"],
    "transferToImmutable({ valueOf: () => " + formatSimpleValue(rawLength) + " }) calls");

  badValueOf = true;
  calls = [];
  ab = new ArrayBuffer(8);
  assert.throws(expectedErr, function() {
    ab.transferToImmutable(objLength);
  }, "transferToImmutable({ toString: () => " + formatSimpleValue(rawLength) + " })");
  assert.compareArray(calls, ["valueOf", "toString"],
    "transferToImmutable({ toString: () => " + formatSimpleValue(rawLength) + " }) calls");

  badToString = true;
  if (typeof Symbol === undefined || !Symbol.toPrimitive) continue;
  calls = [];
  objLength[Symbol.toPrimitive] = function(hint) {
    calls.push("Symbol.toPrimitive(" + hint + ")");
    return rawLength;
  };
  ab = new ArrayBuffer(8);
  assert.throws(expectedErr, function() {
    ab.transferToImmutable(objLength);
  }, "transferToImmutable({ [Symbol.toPrimitive]: () => " + formatSimpleValue(rawLength) + " })");
  assert.compareArray(calls, ["Symbol.toPrimitive(number)"],
    "transferToImmutable({ [Symbol.toPrimitive]: () => " + formatSimpleValue(rawLength) + " }) calls");
}

var calls = [];
var objLength = {
  toString() {
    calls.push("toString");
    return {};
  },
  valueOf() {
    calls.push("valueOf");
    return {};
  }
};
ab = new ArrayBuffer(8);
assert.throws(TypeError, function() {
  ab.transferToImmutable(objLength);
}, "transferToImmutable(badOrdinaryToPrimitive)");
assert.compareArray(calls, ["valueOf", "toString"],
  "transferToImmutable(badOrdinaryToPrimitive) calls");
if (typeof Symbol !== undefined && Symbol.toPrimitive) {
  calls = [];
  objLength[Symbol.toPrimitive] = function(hint) {
    calls.push("Symbol.toPrimitive(" + hint + ")");
    return {};
  };
  ab = new ArrayBuffer(8);
  assert.throws(TypeError, function() {
    ab.transferToImmutable(objLength);
  }, "transferToImmutable(badExoticToPrimitive)");
  assert.compareArray(calls, ["Symbol.toPrimitive(number)"],
    "transferToImmutable(badExoticToPrimitive) calls");
}
