// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter applied to Array-like object, 'length' is
    own data property
---*/

function callbackfn(val, idx, obj) {
  return obj.length === 2;
}

var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: 2
};

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
