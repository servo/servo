// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-11
description: >
    Array.prototype.filter - deleting property of prototype causes
    prototype index property not to be visited on an Array
---*/

function callbackfn(val, idx, obj) {
  return true;
}
var arr = [0, , 2];

Object.defineProperty(arr, "0", {
  get: function() {
    delete Array.prototype[1];
    return 0;
  },
  configurable: true
});

Array.prototype[1] = 1;
var newArr = arr.filter(callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
assert.notSameValue(newArr[1], 1, 'newArr[1]');
