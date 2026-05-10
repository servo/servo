// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some applied to the Arguments object
---*/

function callbackfn(val, idx, obj) {
  return '[object Arguments]' === Object.prototype.toString.call(obj);
}

var obj = (function() {
  return arguments;
}("a", "b"));

assert(Array.prototype.some.call(obj, callbackfn), 'Array.prototype.some.call(obj, callbackfn) !== true');
