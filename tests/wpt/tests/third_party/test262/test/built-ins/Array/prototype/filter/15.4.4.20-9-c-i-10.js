// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-c-i-10
description: >
    Array.prototype.filter - element to be retrieved is own accessor
    property on an Array
---*/

function callbackfn(val, idx, obj) {
  return idx === 2 && val === 12;
}

var arr = [];

Object.defineProperty(arr, "2", {
  get: function() {
    return 12;
  },
  configurable: true
});
var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 12, 'newArr[0]');
