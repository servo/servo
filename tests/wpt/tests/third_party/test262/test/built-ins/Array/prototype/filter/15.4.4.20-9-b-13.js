// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-13
description: >
    Array.prototype.filter - deleting own property with prototype
    property causes prototype index property to be visited on an Array
---*/

function callbackfn(val, idx, obj) {
  return val < 3 ? true : false;
}
var arr = [0, 111, 2];

Object.defineProperty(arr, "0", {
  get: function() {
    delete arr[1];
    return 0;
  },
  configurable: true
});

Array.prototype[1] = 1;
var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 3, 'newArr.length');
assert.sameValue(newArr[1], 1, 'newArr[1]');
