// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - 'this' of 'callbackfn' is an String object
    when T is not an object (T is a string primitive)
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return 'hello' === this.valueOf();
}

var obj = {
  0: 11,
  length: 2
};

assert(Array.prototype.every.call(obj, callbackfn, "hello"), 'Array.prototype.every.call(obj, callbackfn, "hello") !== true');
assert(accessed, 'accessed !== true');
