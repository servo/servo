// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - This object is the Arguments object which
    implements its own property get method (number of arguments equals
    number of parameters)
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 9;
  } else if (idx === 1) {
    return val === 11;
  } else {
    return false;
  }
}

var func = function(a, b) {
  return Array.prototype.map.call(arguments, callbackfn);
};

var testResult = func(9, 11);

assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(testResult[1], true, 'testResult[1]');
