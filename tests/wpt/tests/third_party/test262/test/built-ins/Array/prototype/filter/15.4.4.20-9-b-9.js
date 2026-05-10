// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-9
description: >
    Array.prototype.filter - deleting own property causes index
    property not to be visited on an Array
---*/

function callbackfn(val, idx, obj) {
  return true;
}
var arr = [1, 2];

Object.defineProperty(arr, "1", {
  get: function() {
    return "6.99";
  },
  configurable: true
});

Object.defineProperty(arr, "0", {
  get: function() {
    delete arr[1];
    return 0;
  },
  configurable: true
});

var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 0, 'newArr[0]');
