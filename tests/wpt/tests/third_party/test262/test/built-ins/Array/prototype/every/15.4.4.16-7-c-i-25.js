// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - This object is the Arguments object which
    implements its own property get method (number of arguments is
    less than number of parameters)
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val === 11;
}

var func = function(a, b) {
  return Array.prototype.every.call(arguments, callbackfn);
};

assert(func(11), 'func(11) !== true');
assert.sameValue(called, 1, 'called');
