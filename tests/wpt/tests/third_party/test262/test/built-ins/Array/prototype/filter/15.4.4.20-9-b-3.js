// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-3
description: >
    Array.prototype.filter - deleted properties in step 2 are visible
    here
---*/

function callbackfn(val, idx, obj) {
  return true;
}
var obj = {
  2: 6.99,
  8: 19
};

Object.defineProperty(obj, "length", {
  get: function() {
    delete obj[2];
    return 10;
  },
  configurable: true
});

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.notSameValue(newArr[0], 6.99, 'newArr[0]');
