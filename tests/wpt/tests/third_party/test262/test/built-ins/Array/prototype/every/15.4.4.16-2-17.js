// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every applied to the Arguments object, which
    implements its own property get method
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var func = function(a, b) {
  arguments[2] = 9;
  return Array.prototype.every.call(arguments, callbackfn1) &&
    !Array.prototype.every.call(arguments, callbackfn2);
};

assert(func(12, 11), 'func(12, 11) !== true');
