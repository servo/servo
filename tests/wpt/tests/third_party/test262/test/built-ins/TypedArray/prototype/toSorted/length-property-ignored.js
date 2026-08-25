// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.tosorted
description: >
  %TypedArray%.prototype.toSorted does not read a "length" property
info: |
  %TypedArray%.prototype.toSorted ( comparefn )

  ...
  4. Let len be O.[[ArrayLength]].
  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, change-array-by-copy]
---*/

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg([3, 1, 2]));
  Object.defineProperty(ta, "length", { value: 2 })
  var res = ta.toSorted()
  assert.compareArray(res, [1, 2, 3]);
  assert.sameValue(res.length, 3);

  ta = new TA(makeCtorArg([3, 1, 2]));
  Object.defineProperty(ta, "length", { value: 5 });
  res = ta.toSorted();
  assert.compareArray(res, [1, 2, 3]);
  assert.sameValue(res.length, 3);
});

function setLengthOnPrototype(length) {
    Object.defineProperty(TypedArray.prototype, "length", {
        get: () => length,
    });
}

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  var ta = new TA(makeCtorArg([3, 1, 2]));

  setLengthOnPrototype(2);
  var res = ta.toSorted();
  setLengthOnPrototype(3);
  assert.compareArray(res, [1, 2, 3]);

  setLengthOnPrototype(5);
  res = ta.toSorted();
  setLengthOnPrototype(3);

  assert.compareArray(res, [1, 2, 3]);
}, null, ["passthrough"]);
