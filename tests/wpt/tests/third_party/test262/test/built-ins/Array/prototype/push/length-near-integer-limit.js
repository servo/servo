// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.push
description: >
  A value is inserted in an array-like object whose length property is near the integer limit.
info: |
  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  3. Let items be a List whose elements are, in left to right order, the
     arguments that were passed to this function invocation.
  ...
  5. Repeat, while items is not empty
    ...
  7. Perform ? Set(O, "length", len, true).
  ...
features: [exponentiation]
---*/

var arrayLike = {
  "9007199254740989": "9007199254740989",
  /* "9007199254740990": empty */
  "9007199254740991": "9007199254740991",
  length: 2 ** 53 - 2
};

Array.prototype.push.call(arrayLike, "new-value");

assert.sameValue(arrayLike.length, 2 ** 53 - 1,
  "New arrayLike.length is 2**53 - 1");

assert.sameValue(arrayLike["9007199254740989"], "9007199254740989",
  "arrayLike['9007199254740989'] is unchanged");

assert.sameValue(arrayLike["9007199254740990"], "new-value",
  "arrayLike['9007199254740990'] has new value");

assert.sameValue(arrayLike["9007199254740991"], "9007199254740991",
  "arrayLike['9007199254740991'] is unchanged");
