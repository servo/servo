// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - This object is the Arguments object
    which implements its own property get method (number of arguments
    equals number of parameters)
---*/

var called = 0;
var testResult = false;

function callbackfn(val, idx, obj) {
  called++;
  if (called !== 1 && !testResult) {
    return;
  }
  if (idx === 0) {
    testResult = (val === 11);
  } else if (idx === 1) {
    testResult = (val === 9);
  } else {
    testResult = false;
  }
}

var func = function(a, b) {
  Array.prototype.forEach.call(arguments, callbackfn);
};

func(11, 9);

assert(testResult, 'testResult !== true');
