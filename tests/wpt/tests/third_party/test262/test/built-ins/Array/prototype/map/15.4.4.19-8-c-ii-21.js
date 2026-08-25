// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - callbackfn called with correct parameters
    (kValue is correct)
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    return val === 11;
  }

  if (idx === 1) {
    return val === 12;
  }

  return false;
}

var obj = {
  0: 11,
  1: 12,
  length: 2
};

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(testResult[1], true, 'testResult[1]');
