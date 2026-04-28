// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - Boolean object can be used as thisArg
---*/

var objBoolean = new Boolean();

function callbackfn(val, idx, obj) {
  return this === objBoolean;
}

var testResult = [11].map(callbackfn, objBoolean);

assert.sameValue(testResult[0], true, 'testResult[0]');
