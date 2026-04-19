// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.unshift
description: >
  Test properties are correctly accessed when length property is near 2^53-1.
info: |
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let argCount be the number of actual arguments.
  4. If argCount > 0, then
    ...
    b. Let k be len.
    c. Repeat, while k > 0,
        i. Let from be ! ToString(k-1).
       ii. Let to be ! ToString(k+argCount-1).
      iii. Let fromPresent be ? HasProperty(O, from).
       iv. If fromPresent is true, then
          1. Let fromValue be ? Get(O, from).
          2. Perform ? Set(O, to, fromValue, true).
        v. Else fromPresent is false,
          1. Perform ? DeletePropertyOrThrow(O, to).
       vi. Decrease k by 1.
features: [exponentiation]
---*/

function StopUnshift() {}

var arrayLike = {
  get "9007199254740986" () {
    throw new StopUnshift();
  },
  "9007199254740987": "9007199254740987",
  /* "9007199254740988": hole */
  "9007199254740989": "9007199254740989",
  /* "9007199254740990": empty */
  "9007199254740991": "9007199254740991",
  length: 2 ** 53 - 2
};

assert.throws(StopUnshift, function() {
  Array.prototype.unshift.call(arrayLike, null);
});

assert.sameValue(arrayLike.length, 2 ** 53 - 2,
  "arrayLike.length is unchanged");

assert.sameValue(arrayLike["9007199254740987"], "9007199254740987",
  "arrayLike['9007199254740987'] is unchanged");

assert.sameValue(arrayLike["9007199254740988"], "9007199254740987",
  "arrayLike['9007199254740988'] is replaced with arrayLike['9007199254740987']");

assert.sameValue("9007199254740989" in arrayLike, false,
  "arrayLike['9007199254740989'] is removed");

assert.sameValue(arrayLike["9007199254740990"], "9007199254740989",
  "arrayLike['9007199254740990'] is replaced with arrayLike['9007199254740989']");

assert.sameValue(arrayLike["9007199254740991"], "9007199254740991",
  "arrayLike['9007199254740991'] is unchanged");
