// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every -'this' of 'callbackfn' is a Number object
    when T is not an object (T is a number primitive)
---*/

var accessed = false;

function callbackfn(val, idx, o) {
  accessed = true;
  return 5 === this.valueOf();
}

var obj = {
  0: 11,
  length: 2
};

assert(Array.prototype.every.call(obj, callbackfn, 5), 'Array.prototype.every.call(obj, callbackfn, 5) !== true');
assert(accessed, 'accessed !== true');
