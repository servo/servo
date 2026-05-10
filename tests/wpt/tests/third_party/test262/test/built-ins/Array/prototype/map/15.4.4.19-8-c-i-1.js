// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - element to be retrieved is own data property
    on an Array-like object
---*/

var kValue = {};

function callbackfn(val, idx, obj) {
  if (idx === 5) {
    return val === kValue;
  }
  return false;
}

var obj = {
  5: kValue,
  length: 100
};

var newArr = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(newArr[5], true, 'newArr[5]');
