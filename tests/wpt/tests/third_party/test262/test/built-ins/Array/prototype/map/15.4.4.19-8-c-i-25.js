// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - This object is the Arguments object which
    implements its own property get method (number of arguments is
    less than number of parameters)
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 9;
  } else {
    return false;
  }
}

var func = function(a, b) {
  return Array.prototype.map.call(arguments, callbackfn);
};

var testResult = func(9);

assert.sameValue(testResult[0], true, 'testResult[0]');
