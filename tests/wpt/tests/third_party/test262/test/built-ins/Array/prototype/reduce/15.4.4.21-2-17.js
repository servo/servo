// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce applied to the Arguments object, which
    implements its own property get method
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return (obj.length === 2);
}

var func = function(a, b) {
  arguments[2] = 9;
  return Array.prototype.reduce.call(arguments, callbackfn, 1);
};

assert.sameValue(func(12, 11), true, 'func(12, 11)');
