// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  %TypedArray%.prototype.with reads the TypedArray length ignoring the .length property
info: |
  %TypedArray%.prototype.with ( index, value )

  ...
  3. Let len be O.[[ArrayLength]].
  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg([3, 1, 2]));
  Object.defineProperty(ta, "length", { value: 2 })
  var res = ta.with(0, 0);
  assert.compareArray(res, [0, 1, 2]);
  assert.sameValue(res.length, 3);

  ta = new TA(makeCtorArg([3, 1, 2]));
  Object.defineProperty(ta, "length", { value: 5 });
  res = ta.with(0, 0);
  assert.compareArray(res, [0, 1, 2]);
  assert.sameValue(res.length, 3);
}, null, ["passthrough"]);

function setLength(length) {
    Object.defineProperty(TypedArray.prototype, "length", {
        get: () => length,
    });
}

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg([3, 1, 2]));

  setLength(2);
  var res = ta.with(0, 0);
  setLength(3);
  assert.compareArray(res, [0, 1, 2]);

  setLength(5);
  res = ta.with(0, 0);
  setLength(3);
  assert.compareArray(res, [0, 1, 2]);
}, null, ["passthrough"]);
