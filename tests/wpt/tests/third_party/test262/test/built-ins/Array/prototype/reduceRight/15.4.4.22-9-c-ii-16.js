// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - non-indexed properties are not
    called on an Array-like object
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (prevVal === 8 || curVal === 8) {
    testResult = true;
  }
}

var obj = {
  0: 11,
  10: 12,
  non_index_property: 8,
  length: 20
};
Array.prototype.reduceRight.call(obj, callbackfn, "initialValue");

assert.sameValue(testResult, false, 'testResult');
