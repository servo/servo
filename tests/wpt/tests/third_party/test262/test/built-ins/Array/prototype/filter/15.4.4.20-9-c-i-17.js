// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - element to be retrieved is own accessor
    property without a get function on an Array-like object
---*/

function callbackfn(val, idx, obj) {
  return undefined === val && idx === 1;
}

var obj = {
  length: 2
};
Object.defineProperty(obj, "1", {
  set: function() {},
  configurable: true
});

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], undefined, 'newArr[0]');
