// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter applied to the Arguments object, which
    implements its own property get method
---*/

function callbackfn(val, idx, obj) {
  return obj.length === 2;
}

var func = function(a, b) {
  var newArr = Array.prototype.filter.call(arguments, callbackfn);
  return newArr.length === 2;
};

assert(func(12, 11), 'func(12, 11) !== true');
