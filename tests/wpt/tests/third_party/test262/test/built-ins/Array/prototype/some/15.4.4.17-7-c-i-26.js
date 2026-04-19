// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - This object is the Arguments object which
    implements its own property get method (number of arguments equals
    number of parameters)
---*/

var firstResult = false;
var secondResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    firstResult = (val === 11);
    return false;
  } else if (idx === 1) {
    secondResult = (val === 9);
    return false;
  } else {
    return true;
  }
}

var func = function(a, b) {
  return Array.prototype.some.call(arguments, callbackfn);
};

assert.sameValue(func(11, 9), false, 'func(11, 9)');
assert(firstResult, 'firstResult !== true');
assert(secondResult, 'secondResult !== true');
