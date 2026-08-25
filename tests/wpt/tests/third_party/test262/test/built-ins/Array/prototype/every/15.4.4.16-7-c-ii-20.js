// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - callbackfn called with correct parameters
    (thisArg is correct)
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return 10 === this.threshold;
}

var thisArg = {
  threshold: 10
};

var obj = {
  0: 11,
  length: 1
};

assert(Array.prototype.every.call(obj, callbackfn, thisArg), 'Array.prototype.every.call(obj, callbackfn, thisArg) !== true');
