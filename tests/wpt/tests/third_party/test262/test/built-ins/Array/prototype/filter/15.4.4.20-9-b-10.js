// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-10
description: >
    Array.prototype.filter - deleting property of prototype causes
    prototype index property not to be visited on an Array-like Object
---*/

function callbackfn(val, idx, obj) {
  return true;
}
var obj = {
  2: 2,
  length: 20
};

Object.defineProperty(obj, "0", {
  get: function() {
    delete Object.prototype[1];
    return 0;
  },
  configurable: true
});

Object.prototype[1] = 1;
var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
assert.notSameValue(newArr[1], 1, 'newArr[1]');
