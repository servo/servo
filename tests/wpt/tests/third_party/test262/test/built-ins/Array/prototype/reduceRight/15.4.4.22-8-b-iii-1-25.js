// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - This object is the Arguments object
    which implements its own property get method (number of arguments
    is less than number of parameters)
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 0) {
    testResult = (prevVal === 1);
  }
}

var func = function(a, b, c) {
  Array.prototype.reduceRight.call(arguments, callbackfn);
};

func(0, 1);

assert(testResult, 'testResult !== true');
