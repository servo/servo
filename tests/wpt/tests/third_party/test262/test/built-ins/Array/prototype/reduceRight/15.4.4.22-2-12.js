// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - 'length' is own accessor property
    without a get function that overrides an inherited accessor
    property
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return typeof obj.length === "undefined";
}

Object.defineProperty(Object.prototype, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

var obj = {
  0: 12,
  1: 13
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn, 11), 11, 'Array.prototype.reduceRight.call(obj, callbackfn, 11)');
assert.sameValue(accessed, false, 'accessed');
