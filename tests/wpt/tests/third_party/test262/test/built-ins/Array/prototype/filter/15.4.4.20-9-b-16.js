// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-16
description: >
    Array.prototype.filter - decreasing length of array does not
    delete non-configurable properties
flags: [noStrict]
---*/

function callbackfn(val, idx, obj) {
  return true;
}

var arr = [0, 1, 2];

Object.defineProperty(arr, "2", {
  get: function() {
    return "unconfigurable";
  },
  configurable: false
});

Object.defineProperty(arr, "1", {
  get: function() {
    arr.length = 2;
    return 1;
  },
  configurable: true
});

var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 3, 'newArr.length');
assert.sameValue(newArr[2], "unconfigurable", 'newArr[2]');
