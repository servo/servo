// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.toreversed
description: >
  Array.prototype.toReversed gets the array elements from the last one to the first one.
info: |
  Array.prototype.toReversed ( )

  ...
  2. Let len be ? LengthOfArrayLike(O).
  ...
  5. Repeat, while k < len
    a. Let from be ! ToString(𝔽(len - k - 1)).
    ...
    c. Let fromValue be ? Get(O, from).
    ...
features: [change-array-by-copy]
includes: [compareArray.js]
---*/

var order = [];
var arrayLike = {
  length: 3,
  get 0() {
    order.push(0);
  },
  get 1() {
    order.push(1);
  },
  get 2() {
    order.push(2);
  },
};

Array.prototype.toReversed.call(arrayLike);

assert.compareArray(order, [2, 1, 0]);

order = [];
var arr = [0, 1, 2];
Object.defineProperty(arr, 0, { get: function() { order.push(0); } });
Object.defineProperty(arr, 1, { get: function() { order.push(1); } });
Object.defineProperty(arr, 2, { get: function() { order.push(2); } });

Array.prototype.toReversed.call(arr);

assert.compareArray(order, [2, 1, 0]);
