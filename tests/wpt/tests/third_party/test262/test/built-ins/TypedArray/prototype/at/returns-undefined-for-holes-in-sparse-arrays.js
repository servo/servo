// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.at
description: >
  Returns the item value at the specified index, holes are filled in sparse arrays.
info: |
  %TypedArray%.prototype.at( index )

  Let O be the this value.
  Perform ? ValidateTypedArray(O).
  Let len be O.[[ArrayLength]].
  Let relativeIndex be ? ToInteger(index).
  If relativeIndex ≥ 0, then
    Let k be relativeIndex.
  Else,
    Let k be len + relativeIndex.
  If k < 0 or k ≥ len, then return undefined.
  Return ? Get(O, ! ToString(k)).

includes: [testTypedArray.js]
features: [TypedArray, TypedArray.prototype.at]
---*/
assert.sameValue(
  typeof TypedArray.prototype.at,
  'function',
  'The value of `typeof TypedArray.prototype.at` is "function"'
);

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  let a = new TA(makeCtorArg([0, 1, , 3, 4, , 6]));
  let filler = 0;
  if (TA.name.startsWith('Float')) {
    filler = NaN;
  }
  assert.sameValue(a.at(0), 0, 'a.at(0) must return 0');
  assert.sameValue(a.at(1), 1, 'a.at(1) must return 1');
  assert.sameValue(a.at(2), filler, 'a.at(2) must return the value of filler');
  assert.sameValue(a.at(3), 3, 'a.at(3) must return 3');
  assert.sameValue(a.at(4), 4, 'a.at(4) must return 4');
  assert.sameValue(a.at(5), filler, 'a.at(5) must return the value of filler');
  assert.sameValue(a.at(6), 6, 'a.at(6) must return 6');
  assert.sameValue(a.at(-0), 0, 'a.at(-0) must return 0');
  assert.sameValue(a.at(-1), 6, 'a.at(-1) must return 6');
  assert.sameValue(a.at(-2), filler, 'a.at(-2) must return the value of filler');
  assert.sameValue(a.at(-3), 4, 'a.at(-3) must return 4');
  assert.sameValue(a.at(-4), 3, 'a.at(-4) must return 3');
  assert.sameValue(a.at(-5), filler, 'a.at(-5) must return the value of filler');
  assert.sameValue(a.at(-6), 1, 'a.at(-6) must return 1');
});
