// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - element changed by callbackfn on previous
    iterations is observed
---*/

var obj = {
  0: 9,
  1: 12,
  length: 2
};

function callbackfn(val, idx, o) {
  if (idx === 0) {
    obj[idx + 1] = 8;
  }
  return val > 10;
}

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult[1], false, 'testResult[1]');
