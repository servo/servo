// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - when 'length' is own data property that
    overrides an inherited data property on an Array
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}
var arrProtoLen;

arrProtoLen = Array.prototype.length;
Array.prototype.length = 0;
var testResult = [12, 11].map(callbackfn);

assert.sameValue(testResult.length, 2, 'testResult.length');
