// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - This object is the Arguments object which
    implements its own property get method (number of arguments equals
    number of parameters)
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 11;
  } else if (idx === 1) {
    return val === 9;
  } else {
    return false;
  }
}

var func = function(a, b) {
  return Array.prototype.filter.call(arguments, callbackfn);
};
var newArr = func(11, 9);

assert.sameValue(newArr.length, 2, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
assert.sameValue(newArr[1], 9, 'newArr[1]');
