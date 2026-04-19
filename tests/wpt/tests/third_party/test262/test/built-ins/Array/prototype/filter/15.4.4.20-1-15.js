// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to the Arguments object
---*/

function callbackfn(val, idx, obj) {
  return '[object Arguments]' === Object.prototype.toString.call(obj);
}

var obj = (function() {
  return arguments;
}("a", "b"));

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr[0], "a", 'newArr[0]');
assert.sameValue(newArr[1], "b", 'newArr[1]');
