// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.reverse
description: Throws a TypeError exception when the backing buffer is immutable
info: |
  %TypedArray%.prototype.reverse ( )
  1. Let O be the this value.
  2. Let taRecord be ? ValidateTypedArray(O, ~seq-cst~, ~write~).

  ValidateTypedArray ( O, order [ , accessMode ] )
  1. If accessMode is not present, set accessMode to read.
  2. Perform ? RequireInternalSlot(O, [[TypedArrayName]]).
  3. Assert: O has a [[ViewedArrayBuffer]] internal slot.
  4. If accessMode is ~write~ and IsImmutableBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
features: [TypedArray, immutable-arraybuffer]
includes: [testTypedArray.js, compareArray.js]
---*/

testWithAllTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg(["1", "2", "3", "4"]));

  assert.throws(TypeError, function() {
    ta.reverse();
  });
  assert.compareArray(ta, new TA(["1", "2", "3", "4"]), "Must not mutate contents.");
}, null, ["immutable"]);
