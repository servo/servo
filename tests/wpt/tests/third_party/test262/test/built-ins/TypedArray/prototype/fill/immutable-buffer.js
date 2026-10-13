// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.fill
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  %TypedArray%.prototype.fill ( value [ , start [ , end ] ] )
  1. Let O be the this value.
  2. Let taRecord be ? ValidateTypedArray(O, ~seq-cst~, ~write~).
  3. Let len be TypedArrayLength(taRecord).
  4. If O.[[ContentType]] is bigint, set value to ? ToBigInt(value).
  5. Otherwise, set value to ? ToNumber(value).
  6. Let relativeStart be ? ToIntegerOrInfinity(start).
  7. If relativeStart = -∞, let startIndex be 0.
  8. Else if relativeStart < 0, let startIndex be max(len + relativeStart, 0).
  9. Else, let startIndex be min(relativeStart, len).
  10. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).

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
  var value = {
    valueOf() {
      calls.push("value.valueOf");
      return "8";
    }
  };
  var start = {
    valueOf() {
      calls.push("start.valueOf");
      return 1;
    }
  };
  var end = {
    valueOf() {
      calls.push("end.valueOf");
      return 1;
    }
  };

  assert.throws(TypeError, function() {
    ta.fill(value, start, end);
  });
  assert.compareArray(calls, [], "Must verify mutability before reading arguments.");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]), "Must not mutate contents.");
}, null, ["immutable"]);
