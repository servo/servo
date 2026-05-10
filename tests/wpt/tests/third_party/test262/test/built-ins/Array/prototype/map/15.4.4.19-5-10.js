// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - Array object can be used as thisArg
---*/

var objArray = new Array(2);

function callbackfn(val, idx, obj) {
  return this === objArray;
}

var testResult = [11].map(callbackfn, objArray);

assert.sameValue(testResult[0], true, 'testResult[0]');
