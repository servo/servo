// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - Error object can be used as thisArg
---*/

var objError = new RangeError();

function callbackfn(val, idx, obj) {
  return this === objError;
}

var testResult = [11].map(callbackfn, objError);

assert.sameValue(testResult[0], true, 'testResult[0]');
