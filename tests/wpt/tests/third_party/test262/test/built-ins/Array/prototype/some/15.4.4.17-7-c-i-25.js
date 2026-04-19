// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - This object is the Arguments object which
    implements its own property get method (number of arguments is
    less than number of parameters)
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 11;
  }
  return false;
}

var func = function(a, b) {
  return Array.prototype.some.call(arguments, callbackfn);
};

assert(func(11), 'func(11) !== true');
