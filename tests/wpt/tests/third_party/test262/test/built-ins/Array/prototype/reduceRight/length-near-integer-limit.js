// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
  Elements are processed in an array-like object
  whose "length" property is near the integer limit.
info: |
  Array.prototype.reduceRight ( callbackfn [ , initialValue ] )

  1. Let O be ? ToObject(this value).
  2. Let len be ? LengthOfArrayLike(O).
  [...]
  9. Repeat, while k ≥ 0
    a. Let Pk be ! ToString(k).
    b. Let kPresent be ? HasProperty(O, Pk).
    c. If kPresent is true, then
      i. Let kValue be ? Get(O, Pk).
      ii. Set accumulator to ? Call(callbackfn, undefined, « accumulator, kValue, k, O »).
    [...]
includes: [compareArray.js]
---*/

var arrayLike = {
  length: Number.MAX_SAFE_INTEGER,
};

arrayLike[Number.MAX_SAFE_INTEGER - 1] = 1;
arrayLike[Number.MAX_SAFE_INTEGER - 3] = 3;

var accumulator = function(acc, el, index) {
  acc.push([el, index]);

  if (el === 3) {
    throw acc;
  }

  return acc;
};

try {
  Array.prototype.reduceRight.call(arrayLike, accumulator, []);
  throw new Test262Error("should not be called");
} catch (acc) {
  assert.sameValue(acc.length, 2, 'The value of acc.length is expected to be 2');
  assert.compareArray(
    acc[0],
    [1, Number.MAX_SAFE_INTEGER - 1],
    'The value of acc[0] is expected to be [1, Number.MAX_SAFE_INTEGER - 1]'
  );
  assert.compareArray(
    acc[1],
    [3, Number.MAX_SAFE_INTEGER - 3],
    'The value of acc[1] is expected to be [3, Number.MAX_SAFE_INTEGER - 3]'
  );
}
