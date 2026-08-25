// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - return value of callbackfn is null
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return null;
}

var obj = {
  0: 11,
  length: 1
};

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 0, 'newArr.length');
assert(accessed, 'accessed !== true');
