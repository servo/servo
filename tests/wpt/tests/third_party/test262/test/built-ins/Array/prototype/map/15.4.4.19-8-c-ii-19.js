// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - non-indexed properties are not called.
---*/

var called = 0;
var result = false;

function callbackfn(val, idx, obj) {
  called++;
  if (val === 11) {
    result = true;
  }
  return true;
}

var obj = {
  0: 9,
  non_index_property: 11,
  length: 20
};

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(result, false, 'result');
assert.sameValue(testResult[0], true, 'testResult[0]');
assert.sameValue(called, 1, 'called');
