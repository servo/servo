// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - callbackfn with 0 formal parameter
---*/

function callbackfn() {
  return true;
}

var testResult = [11].map(callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
