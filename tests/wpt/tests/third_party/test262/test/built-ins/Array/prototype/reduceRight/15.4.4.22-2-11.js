// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to Array-like object, 'length'
    is an own accessor property without a get function
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return typeof obj.length === "undefined";
}

var obj = {
  0: 11,
  1: 12
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn, 111), 111, 'Array.prototype.reduceRight.call(obj, callbackfn, 111)');
assert.sameValue(accessed, false, 'accessed');
