// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  A value is inserted in an array-like object whose length property is near the integer limit.
info: |
  ...
  16. Else if itemCount > actualDeleteCount, then
    a. Let k be (len - actualDeleteCount).
    b. Repeat, while k > actualStart
        i. Let from be ! ToString(k + actualDeleteCount - 1).
       ii. Let to be ! ToString(k + itemCount - 1).
      iii. Let fromPresent be ? HasProperty(O, from).
       iv. If fromPresent is true, then
          1. Let fromValue be ? Get(O, from).
          2. Perform ? Set(O, to, fromValue, true).
        v. Else fromPresent is false,
          1. Perform ? DeletePropertyOrThrow(O, to).
       vi. Decrease k by 1.
  ...
includes: [compareArray.js]
features: [exponentiation]
---*/

var arrayLike = {
  "9007199254740985": "9007199254740985",
  "9007199254740986": "9007199254740986",
  "9007199254740987": "9007199254740987",
  /* "9007199254740988": hole */
  "9007199254740989": "9007199254740989",
  /* "9007199254740990": empty */
  "9007199254740991": "9007199254740991",
  length: 2 ** 53 - 2,
};

var result = Array.prototype.splice.call(arrayLike, 9007199254740986, 0, "new-value");

assert.compareArray(result, [], 'The value of result is expected to be []');

assert.sameValue(arrayLike.length, 2 ** 53 - 1, 'The value of arrayLike.length is expected to be 2 ** 53 - 1');

assert.sameValue(arrayLike["9007199254740985"], "9007199254740985",
  'The value of arrayLike["9007199254740985"] is expected to be "9007199254740985"');

assert.sameValue(arrayLike["9007199254740986"], "new-value",
  'The value of arrayLike["9007199254740986"] is expected to be "new-value"');

assert.sameValue(arrayLike["9007199254740987"], "9007199254740986",
  'The value of arrayLike["9007199254740987"] is expected to be "9007199254740986"');

assert.sameValue(arrayLike["9007199254740988"], "9007199254740987",
  'The value of arrayLike["9007199254740988"] is expected to be "9007199254740987"');

assert.sameValue("9007199254740989" in arrayLike, false,
  'The result of evaluating ("9007199254740989" in arrayLike) is expected to be false');

assert.sameValue(arrayLike["9007199254740990"], "9007199254740989",
  'The value of arrayLike["9007199254740990"] is expected to be "9007199254740989"');

assert.sameValue(arrayLike["9007199254740991"], "9007199254740991",
  'The value of arrayLike["9007199254740991"] is expected to be "9007199254740991"');
