// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  Length and deleteCount are both clamped to 2^53-1 when they exceed the integer limit.
info: |
  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
  7. Else,
    a. Let insertCount be the number of actual arguments minus 2.
    b. Let dc be ? ToInteger(deleteCount).
    c. Let actualDeleteCount be min(max(dc, 0), len - actualStart).
  ...
  11. Repeat, while k < actualDeleteCount
    a. Let from be ! ToString(actualStart+k).
    b. Let fromPresent be ? HasProperty(O, from).
    c. If fromPresent is true, then
       i. Let fromValue be ? Get(O, from).
      ii. Perform ? CreateDataPropertyOrThrow(A, ! ToString(k), fromValue).
    d. Increment k by 1.
  ...
includes: [compareArray.js]
features: [exponentiation]
---*/

var arrayLike = {
  "9007199254740988": "9007199254740988",
  "9007199254740989": "9007199254740989",
  "9007199254740990": "9007199254740990",
  "9007199254740991": "9007199254740991",
  length: 2 ** 53 + 2,
};

var result = Array.prototype.splice.call(arrayLike, 9007199254740989, 2 ** 53 + 4);

assert.compareArray(result, ["9007199254740989", "9007199254740990"],
  'The value of result is expected to be ["9007199254740989", "9007199254740990"]');

assert.sameValue(arrayLike.length, 2 ** 53 - 3,
  'The value of arrayLike.length is expected to be 2 ** 53 - 3');

assert.sameValue(arrayLike["9007199254740988"], "9007199254740988",
  'The value of arrayLike["9007199254740988"] is expected to be "9007199254740988"');

assert.sameValue("9007199254740989" in arrayLike, false,
  'The result of evaluating ("9007199254740989" in arrayLike) is expected to be false');

assert.sameValue("9007199254740990" in arrayLike, false,
  'The result of evaluating ("9007199254740990" in arrayLike) is expected to be false');

assert.sameValue(arrayLike["9007199254740991"], "9007199254740991",
  'The value of arrayLike["9007199254740991"] is expected to be "9007199254740991"');
