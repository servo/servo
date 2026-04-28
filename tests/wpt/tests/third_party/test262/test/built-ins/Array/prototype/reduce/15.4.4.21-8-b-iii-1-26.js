// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - This object is the Arguments object which
    implements its own property get method (number of arguments equals
    number of parameters)
---*/

var testResult = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 2) {
    testResult = (prevVal === 1);
  }
}

var func = function(a, b, c) {
  delete arguments[0];
  Array.prototype.reduce.call(arguments, callbackfn);
};

func(0, 1, 2);

assert(testResult, 'testResult !== true');
