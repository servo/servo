// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - Date object can be used as thisArg
---*/

var objDate = new Date(0);

function callbackfn(val, idx, obj) {
  return this === objDate;
}

var testResult = [11].map(callbackfn, objDate);

assert.sameValue(testResult[0], true, 'testResult[0]');
