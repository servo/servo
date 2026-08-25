// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - the global object can be used as thisArg
---*/

var global = this;

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === global;
}

var newArr = [11].filter(callbackfn, global);

assert.sameValue(newArr[0], 11, 'newArr[0]');
assert(accessed, 'accessed !== true');
