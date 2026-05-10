// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.at
description: >
  Returns the item value at the specified index, respecting holes in sparse arrays.
info: |
  Array.prototype.at ( )

  Let O be ? ToObject(this value).
  Let len be ? LengthOfArrayLike(O).
  Let relativeIndex be ? ToInteger(index).
  If relativeIndex ≥ 0, then
    Let k be relativeIndex.
  Else,
    Let k be len + relativeIndex.
  If k < 0 or k ≥ len, then return undefined.
  Return ? Get(O, ! ToString(k)).

features: [Array.prototype.at]
---*/
assert.sameValue(
  typeof Array.prototype.at,
  'function',
  'The value of `typeof Array.prototype.at` is expected to be "function"'
);

let a = [0, 1, , 3, 4, , 6];

assert.sameValue(a.at(0), 0, 'a.at(0) must return 0');
assert.sameValue(a.at(1), 1, 'a.at(1) must return 1');
assert.sameValue(a.at(2), undefined, 'a.at(2) returns undefined');
assert.sameValue(a.at(3), 3, 'a.at(3) must return 3');
assert.sameValue(a.at(4), 4, 'a.at(4) must return 4');
assert.sameValue(a.at(5), undefined, 'a.at(5) returns undefined');
assert.sameValue(a.at(6), 6, 'a.at(6) must return 6');
assert.sameValue(a.at(-0), 0, 'a.at(-0) must return 0');
assert.sameValue(a.at(-1), 6, 'a.at(-1) must return 6');
assert.sameValue(a.at(-2), undefined, 'a.at(-2) returns undefined');
assert.sameValue(a.at(-3), 4, 'a.at(-3) must return 4');
assert.sameValue(a.at(-4), 3, 'a.at(-4) must return 3');
assert.sameValue(a.at(-5), undefined, 'a.at(-5) returns undefined');
assert.sameValue(a.at(-6), 1, 'a.at(-6) must return 1');
