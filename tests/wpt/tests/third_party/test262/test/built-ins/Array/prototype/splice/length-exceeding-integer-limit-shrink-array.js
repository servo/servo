// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.splice
description: >
  An element is removed from an array-like object whose length exceeds the integer limit.
info: |
  ...
  15. If itemCount < actualDeleteCount, then
    a. Let k be actualStart.
    b. Repeat, while k < (len - actualDeleteCount)
        i. Let from be ! ToString(k+actualDeleteCount).
       ii. Let to be ! ToString(k+itemCount).
      iii. Let fromPresent be ? HasProperty(O, from).
       iv. If fromPresent is true, then
          1. Let fromValue be ? Get(O, from).
          2. Perform ? Set(O, to, fromValue, true).
        v. Else fromPresent is false,
          1. Perform ? DeletePropertyOrThrow(O, to).
       vi. Increase k by 1.
    c. Let k be len.
    d. Repeat, while k > (len - actualDeleteCount + itemCount)
       i. Perform ? DeletePropertyOrThrow(O, ! ToString(k-1)).
      ii. Decrease k by 1.
  ...
includes: [compareArray.js]
features: [exponentiation]
---*/

var arrayLike = {
  "9007199254740986": "9007199254740986",
  "9007199254740987": "9007199254740987",
  "9007199254740988": "9007199254740988",
  /* "9007199254740989": hole */
  "9007199254740990": "9007199254740990",
  "9007199254740991": "9007199254740991",
  length: 2 ** 53 + 2,
};

var result = Array.prototype.splice.call(arrayLike, 9007199254740987, 1);

assert.compareArray(result, ["9007199254740987"],
  'The value of result is expected to be ["9007199254740987"]');

assert.sameValue(arrayLike.length, 2 ** 53 - 2,
  'The value of arrayLike.length is expected to be 2 ** 53 - 2');

assert.sameValue(arrayLike["9007199254740986"], "9007199254740986",
  'The value of arrayLike["9007199254740986"] is expected to be "9007199254740986"');

assert.sameValue(arrayLike["9007199254740987"], "9007199254740988",
  'The value of arrayLike["9007199254740987"] is expected to be "9007199254740988"');

assert.sameValue("9007199254740988" in arrayLike, false,
  'The result of evaluating ("9007199254740988" in arrayLike) is expected to be false');

assert.sameValue(arrayLike["9007199254740989"], "9007199254740990",
  'The value of arrayLike["9007199254740989"] is expected to be "9007199254740990"');

assert.sameValue("9007199254740990" in arrayLike, false,
  'The result of evaluating ("9007199254740990" in arrayLike) is expected to be false');

assert.sameValue(arrayLike["9007199254740991"], "9007199254740991",
  'The value of arrayLike["9007199254740991"] is expected to be "9007199254740991"');
