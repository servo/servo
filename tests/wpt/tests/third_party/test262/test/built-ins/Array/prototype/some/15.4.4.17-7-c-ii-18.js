// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - 'this' of 'callbackfn' is an String object
    when T is not an object (T is a string primitive)
---*/

function callbackfn(val, idx, obj) {
  return this.valueOf() === "hello!";
}

var obj = {
  0: 11,
  1: 9,
  length: 2
};

assert(Array.prototype.some.call(obj, callbackfn, "hello!"), 'Array.prototype.some.call(obj, callbackfn, "hello!") !== true');
