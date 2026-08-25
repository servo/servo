// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-12
description: >
    Array.prototype.filter - deleting own property with prototype
    property causes prototype index property to be visited on an
    Array-like object
---*/

function callbackfn(val, idx, obj) {
  return true;
}
var obj = {
  0: 0,
  1: 111,
  2: 2,
  length: 10
};

Object.defineProperty(obj, "0", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

Object.prototype[1] = 1;
var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 3, 'newArr.length');
assert.sameValue(newArr[1], 1, 'newArr[1]');
