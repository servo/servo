// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - applied to the Arguments object
---*/

function callbackfn(val, idx, obj) {
  return ('[object Arguments]' === Object.prototype.toString.call(obj));
}

var obj = (function() {
  return arguments;
}("a", "b"));

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult[1], true, 'testResult[1]');
