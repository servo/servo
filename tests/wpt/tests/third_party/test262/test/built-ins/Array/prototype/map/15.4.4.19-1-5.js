// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - applied to number primitive
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Number;
}

Number.prototype[0] = 1;
Number.prototype.length = 1;

var testResult = Array.prototype.map.call(2.5, callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
