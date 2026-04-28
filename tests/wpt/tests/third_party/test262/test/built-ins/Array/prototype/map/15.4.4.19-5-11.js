// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - String object can be used as thisArg
---*/

var objString = new String();

function callbackfn(val, idx, obj) {
  return this === objString;
}

var testResult = [11].map(callbackfn, objString);

assert.sameValue(testResult[0], true, 'testResult[0]');
