// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.copywithin
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  %TypedArray%.prototype.copyWithin ( target, start [ , end ] )
  1. Let O be the this value.
  2. Let taRecord be ? ValidateTypedArray(O, ~seq-cst~, ~write~).
  3. Let len be TypedArrayLength(taRecord).
  4. Let relativeTarget be ? ToIntegerOrInfinity(target).
  5. If relativeTarget = -∞, let targetIndex be 0.
  6. Else if relativeTarget < 0, let targetIndex be max(len + relativeTarget, 0).
  7. Else, let targetIndex be min(relativeTarget, len).
  8. Let relativeStart be ? ToIntegerOrInfinity(start).
  9. If relativeStart = -∞, let startIndex be 0.
  10. Else if relativeStart < 0, let startIndex be max(len + relativeStart, 0).
  11. Else, let startIndex be min(relativeStart, len).
  12. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).

  ValidateTypedArray ( O, order [ , accessMode ] )
  1. If accessMode is not present, set accessMode to read.
  2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
  3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
  4. If accessMode is ~write~ and IsImmutableBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
features: [TypedArray, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var calls = [];

  var ta = new TA(makeCtorArg(["1", "2", "3", "4"]));
  var target = {
    valueOf() {
      calls.push("target.valueOf");
      return 1;
    }
  };
  var start = {
    valueOf() {
      calls.push("start.valueOf");
      return 2;
    }
  };
  var end = {
    valueOf() {
      calls.push("end.valueOf");
      return 2;
    }
  };

  assert.throws(TypeError, function() {
    ta.copyWithin(target, start, end);
  });
  assert.compareArray(calls, [], "Must verify mutability before reading arguments.");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]), "Must not mutate contents.");
}, null, ["immutable"]);
