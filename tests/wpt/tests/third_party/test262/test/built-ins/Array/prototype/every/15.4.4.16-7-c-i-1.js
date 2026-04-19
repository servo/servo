// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own data
    property on an Array-like object
---*/

var kValue = {};

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    return val !== kValue;
  } else {
    return true;
  }
}

var obj = {
  5: kValue,
  length: 100
};

assert.sameValue(Array.prototype.every.call(obj, callbackfn), false, 'Array.prototype.every.call(obj, callbackfn)');
