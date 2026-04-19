// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - 'length' is own accessor property without
    a get function that overrides an inherited accessor property
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return true;
}

Object.defineProperty(Object.prototype, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

var obj = {
  0: 12,
  1: 11
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 0, 'newArr.length');
assert.sameValue(accessed, false, 'accessed');
