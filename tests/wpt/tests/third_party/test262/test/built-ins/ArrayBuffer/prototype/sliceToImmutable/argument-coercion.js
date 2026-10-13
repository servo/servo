// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slicetoimmutable
description: >
  Requested start and end are coerced to integers and checked for validity, then
  (if valid) an immutable ArrayBuffer with the correct length and contents is
  returned
info: |
  ArrayBuffer.prototype.sliceToImmutable ( start, end )
  ...
  5. Let len be O.[[ArrayBufferByteLength]].
  6. Let bounds be ? ResolveBounds(len, start, end).
  7. Let first be bounds.[[From]].
  8. Let final be bounds.[[To]].
  9. Let newLen be max(final - first, 0).
  ...
  12. Let fromBuf be O.[[ArrayBufferData]].
  13. Let currentLen be O.[[ArrayBufferByteLength]].
  14. If currentLen < final, throw a RangeError exception.
  15. Let newBuffer be ? AllocateImmutableArrayBuffer(%ArrayBuffer%, newLen, fromBuf, first, newLen).
  16. Return newBuffer.

  ResolveBounds ( len, start, end )
  1. Let relativeStart be ? ToIntegerOrInfinity(start).
  2. If relativeStart = -∞, let from be 0.
  3. Else if relativeStart < 0, let from be max(len + relativeStart, 0).
  4. Else, let from be min(relativeStart, len).
  5. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
  6. If relativeEnd = -∞, let to be 0.
  7. Else if relativeEnd < 0, let to be max(len + relativeEnd, 0).
  8. Else, let to be min(relativeEnd, len).
  9. Return the Record { [[From]]: from, [[To]]: to }.

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
assert.sameValue(ab.sliceToImmutable().byteLength, 8,
  "Must default to receiver byteLength.");
ab = new ArrayBuffer(8);
assert.sameValue(ab.sliceToImmutable(undefined, undefined).byteLength, 8,
  "Must default (undefined, undefined) to receiver byteLength.");

function make32ByteArrayBuffer() {
  var ab = new ArrayBuffer(32);
  var view = new Uint8Array(ab);
  for (var i = 0; i < 8; i++) view[i] = i + 1;
  return ab;
}

var goodInputs = [
  // Unmodified non-negative integral numbers
  [0, 0],
  [1, 1],
  [10, 10],
  // Truncated non-negative integral numbers
  [0.9, 0],
  [1.9, 1],
  [-0.9, 0],
  // Negative integral numbers
  [-1, -1],
  [-1.9, -1],
  [-2.9, -2],
  // Coerced to integral numbers
  [-0, 0],
  [null, 0],
  [false, 0],
  [true, 1],
  ["", 0],
  ["8", 8],
  ["+9", 9],
  ["-9", -9],
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
  ["0x00_ff", 0],
  // Clamped
  [-32, 0],
  ["-32", 0],
  [-Infinity, 0],
  ["-Infinity", 0],
  [33, 32],
  ["33", 32],
  [9007199254740992, 32], // Math.pow(2, 53) = 9007199254740992
  ["9007199254740992", 32], // Math.pow(2, 53) = 9007199254740992
  [Infinity, 32],
  ["Infinity", 32]
];

for (var i = 0; i < goodInputs.length; i++) {
  var rawStart = goodInputs[i][0];
  var intStart = goodInputs[i][1];
  for (var j = 0; j < goodInputs.length; j++) {
    var rawEnd = goodInputs[j][0];
    var intEnd = goodInputs[j][1];
    var intLength = Math.max(
      0,
      (intEnd >= 0 ? intEnd : 32 + intEnd) - (intStart >= 0 ? intStart : 32 + intStart)
    );
    var source = make32ByteArrayBuffer();
    var expectContents = Array.from(new Uint8Array(source)).slice(rawStart, rawEnd);
    var dest = source.sliceToImmutable(rawStart, rawEnd);
    var label = "sliceToImmutable(" + formatSimpleValue(rawStart) + ", " + formatSimpleValue(rawEnd) + ")";
    assert.sameValue(dest.byteLength, intLength, label + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents, label + " contents");
    assert.sameValue(dest.immutable, true, label + ".immutable");
  }
}

var whitespace = "\t\v\f\uFEFF\u3000\n\r\u2028\u2029";
function pad(rawInput) {
  if (typeof rawInput === "string") {
    return { skip: false, string: whitespace + rawInput + whitespace };
  } else if (typeof rawInput === "number") {
    return {
      skip: false,
      string: whitespace + (isNegativeZero(rawInput) ? "-0" : rawInput) + whitespace
    };
  }
  return { skip: true };
}
for (var i = 0; i < goodInputs.length; i++) {
  var rawStart = goodInputs[i][0];
  var intStart = goodInputs[i][1];
  var paddedStart = pad(rawStart);
  if (paddedStart.skip) continue;
  for (var j = 0; j < goodInputs.length; j++) {
    var rawEnd = goodInputs[j][0];
    var intEnd = goodInputs[j][1];
    var paddedEnd = pad(rawEnd);
    if (paddedEnd.skip) continue;
    var intLength = Math.max(
      0,
      (intEnd >= 0 ? intEnd : 32 + intEnd) - (intStart >= 0 ? intStart : 32 + intStart)
    );
    var source = make32ByteArrayBuffer();
    var expectContents = Array.from(new Uint8Array(source)).slice(rawStart, rawEnd);
    var dest = source.sliceToImmutable(paddedStart.string, paddedEnd.string);
    var label = "sliceToImmutable(" + formatSimpleValue(paddedStart.string) + ", " + formatSimpleValue(paddedEnd.string) + ")";
    assert.sameValue(dest.byteLength, intLength, label + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents, label + " contents");
    assert.sameValue(dest.immutable, true, label + ".immutable");
  }
}

for (var i = 0; i < goodInputs.length; i++) {
  var rawStart = goodInputs[i][0];
  var intStart = goodInputs[i][1];
  for (var j = 0; j < goodInputs.length; j++) {
    var rawEnd = goodInputs[j][0];
    var intEnd = goodInputs[j][1];
    var intLength = Math.max(
      0,
      (intEnd >= 0 ? intEnd : 32 + intEnd) - (intStart >= 0 ? intStart : 32 + intStart)
    );

    var calls = [];

    var badStartValueOf = false;
    var badStartToString = false;
    var objStart = {
      valueOf() {
        calls.push("start.valueOf");
        return badStartValueOf ? {} : rawStart;
      },
      toString() {
        calls.push("start.toString");
        return badStartToString ? {} : rawStart;
      }
    };
    var badEndValueOf = false;
    var badEndToString = false;
    var objEnd = {
      valueOf() {
        calls.push("end.valueOf");
        return badEndValueOf ? {} : rawEnd;
      },
      toString() {
        calls.push("end.toString");
        return badEndToString ? {} : rawEnd;
      }
    };
    function reprArgs(startMethodName, endMethodName) {
      var startRepr = "{ " + startMethodName + ": () => " + formatSimpleValue(rawStart) + " }";
      var endRepr = "{ " + endMethodName + ": () => " + formatSimpleValue(rawEnd) + " }";
      return "(" + startRepr + ", " + endRepr + ")";
    }

    var source = make32ByteArrayBuffer();
    var expectContents = Array.from(new Uint8Array(source)).slice(rawStart, rawEnd);
    var dest = source.sliceToImmutable(objStart, objEnd);
    assert.sameValue(dest.byteLength, intLength,
      "sliceToImmutable" + reprArgs("valueOf", "valueOf") + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents,
      "sliceToImmutable" + reprArgs("valueOf", "valueOf") + " contents");
    assert.sameValue(dest.immutable, true,
      "sliceToImmutable" + reprArgs("valueOf", "valueOf") + ".immutable");
    assert.compareArray(calls, ["start.valueOf", "end.valueOf"],
      "sliceToImmutable" + reprArgs("valueOf", "valueOf") + " calls");

    badStartValueOf = true;
    calls = [];
    source = make32ByteArrayBuffer();
    dest = source.sliceToImmutable(objStart, objEnd);
    assert.sameValue(dest.byteLength, intLength,
      "sliceToImmutable" + reprArgs("toString", "valueOf") + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents,
      "sliceToImmutable" + reprArgs("toString", "valueOf") + " contents");
    assert.sameValue(dest.immutable, true,
      "sliceToImmutable" + reprArgs("toString", "valueOf") + ".immutable");
    assert.compareArray(calls, ["start.valueOf", "start.toString", "end.valueOf"],
      "sliceToImmutable" + reprArgs("toString", "valueOf") + " calls");

    badEndValueOf = true;
    calls = [];
    source = make32ByteArrayBuffer();
    dest = source.sliceToImmutable(objStart, objEnd);
    assert.sameValue(dest.byteLength, intLength,
      "sliceToImmutable" + reprArgs("toString", "toString") + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents,
      "sliceToImmutable" + reprArgs("toString", "toString") + " contents");
    assert.sameValue(dest.immutable, true,
      "sliceToImmutable" + reprArgs("toString", "toString") + ".immutable");
    assert.compareArray(calls, ["start.valueOf", "start.toString", "end.valueOf", "end.toString"],
      "sliceToImmutable" + reprArgs("toString", "toString") + " calls");

    badEndToString = true;
    if (typeof Symbol === undefined || !Symbol.toPrimitive) continue;
    calls = [];
    objEnd[Symbol.toPrimitive] = function(hint) {
      calls.push("end[Symbol.toPrimitive](" + hint + ")");
      return rawEnd;
    };
    source = make32ByteArrayBuffer();
    dest = source.sliceToImmutable(objStart, objEnd);
    assert.sameValue(dest.byteLength, intLength,
      "sliceToImmutable" + reprArgs("toString", "[Symbol.toPrimitive]") + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents,
      "sliceToImmutable" + reprArgs("toString", "[Symbol.toPrimitive]") + " contents");
    assert.sameValue(dest.immutable, true,
      "sliceToImmutable" + reprArgs("toString", "[Symbol.toPrimitive]") + ".immutable");
    assert.compareArray(
      calls,
      ["start.valueOf", "start.toString", "end[Symbol.toPrimitive](number)"],
      "sliceToImmutable" + reprArgs("toString", "[Symbol.toPrimitive]") + " calls"
    );

    badStartToString = true;
    calls = [];
    objStart[Symbol.toPrimitive] = function(hint) {
      calls.push("start[Symbol.toPrimitive](" + hint + ")");
      return rawStart;
    };
    source = make32ByteArrayBuffer();
    dest = source.sliceToImmutable(objStart, objEnd);
    assert.sameValue(dest.byteLength, intLength,
      "sliceToImmutable" + reprArgs("[Symbol.toPrimitive]", "[Symbol.toPrimitive]") + ".byteLength");
    assert.compareArray(new Uint8Array(dest), expectContents,
      "sliceToImmutable" + reprArgs("[Symbol.toPrimitive]", "[Symbol.toPrimitive]") + " contents");
    assert.sameValue(dest.immutable, true,
      "sliceToImmutable" + reprArgs("[Symbol.toPrimitive]", "[Symbol.toPrimitive]") + ".immutable");
    assert.compareArray(
      calls,
      ["start[Symbol.toPrimitive](number)", "end[Symbol.toPrimitive](number)"],
      "sliceToImmutable" + reprArgs("[Symbol.toPrimitive]", "[Symbol.toPrimitive]") + " calls");
  }
}

function CustomError() {}

var badInputs = [
  // non-numbers
  typeof Symbol === undefined ? undefined : [Symbol("1"), TypeError],
  typeof Symbol === undefined || !Symbol.for ? undefined : [Symbol.for("1"), TypeError],
  typeof BigInt === undefined ? undefined : [BigInt(1), TypeError],
  // thrower
  [
    {
      valueOf() {
        throw new CustomError();
      },
      toString() {
        return "{ valueOf: throwError }";
      },
    },
    CustomError
  ]
];

for (var i = 0; i < badInputs.length; i++) {
  if (!badInputs[i]) continue;
  var rawBad = badInputs[i][0];
  var expectedErr = badInputs[i][1];

  var ab = make32ByteArrayBuffer();
  var rawGood = goodInputs[i % goodInputs.length][0];
  var calls = [];
  var objGood = {
    valueOf() {
      calls.push("good.valueOf");
      return rawGood;
    }
  };
  assert.throws(
    expectedErr,
    function() {
      ab.sliceToImmutable(rawBad, objGood);
    },
    "sliceToImmutable(" + formatSimpleValue(rawBad) + ", { valueOf: () => " + formatSimpleValue(rawGood) + " })"
  );
  assert.compareArray(calls, [],
    "sliceToImmutable(" + formatSimpleValue(rawBad) + ", { valueOf: () => " + formatSimpleValue(rawGood) + " }) calls");

  calls = [];
  assert.throws(
    expectedErr,
    function() {
      ab.sliceToImmutable(objGood, rawBad);
    },
    "sliceToImmutable({ valueOf: () => " + formatSimpleValue(rawGood) + " }, " + formatSimpleValue(rawBad) + ")"
  );
  assert.compareArray(calls, ["good.valueOf"],
    "sliceToImmutable({ valueOf: () => " + formatSimpleValue(rawGood) + " }, " + formatSimpleValue(rawBad) + ") calls");
}

var calls = [];
var objBad = {
  toString() {
    calls.push("toString");
    return {};
  },
  valueOf() {
    calls.push("valueOf");
    return {};
  }
};
ab = make32ByteArrayBuffer();
assert.throws(TypeError, function() {
  ab.sliceToImmutable(objBad);
}, "sliceToImmutable(badOrdinaryToPrimitive)");
assert.compareArray(calls, ["valueOf", "toString"],
  "sliceToImmutable(badOrdinaryToPrimitive) calls");
if (typeof Symbol !== undefined && Symbol.toPrimitive) {
  calls = [];
  objBad[Symbol.toPrimitive] = function(hint) {
    calls.push("Symbol.toPrimitive(" + hint + ")");
    return {};
  };
  ab = make32ByteArrayBuffer();
  assert.throws(TypeError, function() {
    ab.sliceToImmutable(objBad);
  }, "sliceToImmutable(badExoticToPrimitive)");
  assert.compareArray(calls, ["Symbol.toPrimitive(number)"],
    "sliceToImmutable(badExoticToPrimitive) calls");
}
