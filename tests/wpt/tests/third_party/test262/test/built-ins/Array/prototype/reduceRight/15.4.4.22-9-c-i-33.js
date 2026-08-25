// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - unnhandled exceptions happened in
    getter terminate iteration on an Array
---*/

var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx <= 1) {
    accessed = true;
  }
}

var arr = [0, , 2];

Object.defineProperty(arr, "1", {
  get: function() {
    throw new Test262Error("unhandle exception happened in getter");
  },
  configurable: true
});

assert.throws(Test262Error, function() {
  arr.reduceRight(callbackfn, "initialValue");
});

assert.sameValue(accessed, false, 'accessed');
