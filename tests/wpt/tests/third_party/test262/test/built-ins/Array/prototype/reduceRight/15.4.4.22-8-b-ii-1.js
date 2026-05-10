// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - added properties in step 2 are
    visible here
---*/

var obj = {};

function callbackfn(prevVal, curVal, idx, obj) {}

Object.defineProperty(obj, "length", {
  get: function() {
    obj[2] = "accumulator";
    return 3;
  },
  configurable: true
});

assert.sameValue(Array.prototype.reduceRight.call(obj, callbackfn), "accumulator", 'Array.prototype.reduceRight.call(obj, callbackfn)');
