// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - boolean primitive can be used as thisArg
---*/

function callbackfn(val, idx, obj) {
  return this.valueOf() === false;
}

var testResult = [11].map(callbackfn, false);

assert.sameValue(testResult[0], true, 'testResult[0]');
