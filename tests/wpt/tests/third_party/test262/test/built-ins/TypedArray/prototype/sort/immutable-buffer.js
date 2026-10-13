// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.sort
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  %TypedArray%.prototype.sort ( comparator )
  1. If comparator is not undefined and IsCallable(comparator) is false, throw a TypeError exception.
  2. Let obj be the this value.
  3. Let taRecord be ? ValidateTypedArray(obj, ~seq-cst~, ~write~).
  4. Let len be TypedArrayLength(taRecord).
  5. NOTE: The following closure performs a numeric comparison rather than the string comparison used in 23.1.3.30.
  6. Let SortCompare be a new Abstract Closure with parameters (x, y) that captures comparator and performs the following steps when called:
     a. Return ? CompareTypedArrayElements(x, y, comparator).
  7. Let sortedList be ? SortIndexedProperties(obj, len, SortCompare, ~read-through-holes~).

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
  function comparator() {
    calls.push("compare");
    return 0;
  }

  assert.throws(TypeError, function() {
    ta.sort(comparator);
  });
  assert.compareArray(calls, [], "Must verify mutability before comparing.");
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]), "Must not mutate contents.");

  calls = [];
  var empty = new TA(makeCtorArg(0));
  assert.throws(TypeError, function() {
    empty.sort(comparator);
  }, "Must verify mutability even when receiver is length 0");
}, null, ["immutable"]);
