// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-b-8
description: >
    Array.prototype.filter - deleting own property causes index
    property not to be visited on an Array-like object
---*/

var accessed = false;
var obj = {
  length: 2
};

function callbackfn(val, idx, o) {
  accessed = true;
  return true;
}

Object.defineProperty(obj, "1", {
  get: function() {
    return 6.99;
  },
  configurable: true
});

Object.defineProperty(obj, "0", {
  get: function() {
    delete obj[1];
    return 0;
  },
  configurable: true
});

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 0, 'newArr[0]');
