// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to the Arguments object
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return ('[object Arguments]' === Object.prototype.toString.call(obj));
}

var obj = (function() {
  return arguments;
}("a", "b"));

assert(Array.prototype.reduce.call(obj, callbackfn, 1), 'Array.prototype.reduce.call(obj, callbackfn, 1) !== true');
