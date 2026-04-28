// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to Function object which
    implements its own property get method
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (prevVal === 2);
  }
}

var obj = function(a, b, c) {
  return a + b + c;
};

obj[0] = 0;
obj[1] = 1;
obj[2] = 2;

Array.prototype.reduceRight.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
