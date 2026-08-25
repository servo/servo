// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - return value of callbackfn is null
---*/

var accessed = false;
var obj = {
  0: 11,
  length: 1
};

function callbackfn(val, idx, obj) {
  accessed = true;
  return null;
}



assert.sameValue(Array.prototype.every.call(obj, callbackfn), false, 'Array.prototype.every.call(obj, callbackfn)');
assert(accessed, 'accessed !== true');
