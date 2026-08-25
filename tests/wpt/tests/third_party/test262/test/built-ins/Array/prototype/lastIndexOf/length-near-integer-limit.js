// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
  Elements are found in an array-like object
  whose "length" property is near the integer limit.
info: |
  Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  [...]
  7. Repeat, while k ≥ 0
    a. Let kPresent be ? HasProperty(O, ! ToString(k)).
    b. If kPresent is true, then
      i. Let elementK be ? Get(O, ! ToString(k)).
      ii. Let same be the result of performing Strict Equality Comparison searchElement === elementK.
      iii. If same is true, return k.
    [...]
---*/

var el = {};
var elIndex = Number.MAX_SAFE_INTEGER - 3;
var fromIndex = Number.MAX_SAFE_INTEGER - 1;
var arrayLike = {
  length: Number.MAX_SAFE_INTEGER,
};

arrayLike[elIndex] = el;

var res = Array.prototype.lastIndexOf.call(arrayLike, el, fromIndex);

assert.sameValue(res, elIndex);
