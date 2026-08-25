// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach applied to the Arguments object, which
    implements its own property get method
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 2);
}

var func = function(a, b) {
  arguments[2] = 9;
  Array.prototype.forEach.call(arguments, callbackfn);
  return result;
};

assert(func(12, 11), 'func(12, 11) !== true');
