// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-9-8
description: Array.prototype.filter - no observable effects occur if len is 0
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var obj = {
  0: 11,
  1: 12,
  length: 0
};

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(accessed, false, 'accessed');
assert.sameValue(obj.length, 0, 'obj.length');
assert.sameValue(newArr.length, 0, 'newArr.length');
