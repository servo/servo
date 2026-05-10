// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-2
description: >
    Array.prototype.filter - added properties in step 2 are visible
    here
---*/

function callbackfn(val, idx, obj) {
  return true;
}

var obj = {};

Object.defineProperty(obj, "length", {
  get: function() {
    obj[2] = "length";
    return 3;
  },
  configurable: true
});

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], "length", 'newArr[0]');
