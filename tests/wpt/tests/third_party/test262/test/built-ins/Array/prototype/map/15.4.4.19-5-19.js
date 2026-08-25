// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - the Arguments object can be used as thisArg
---*/

var arg;

function callbackfn(val, idx, obj) {
  return this === arg;
}

arg = (function() {
  return arguments;
}(1, 2, 3));

var testResult = [11].map(callbackfn, arg);

assert.sameValue(testResult[0], true, 'testResult[0]');
