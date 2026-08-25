// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - Date Object can be used as thisArg
---*/

var accessed = false;

var objDate = new Date(0);

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === objDate;
}

var newArr = [11].filter(callbackfn, objDate);

assert.sameValue(newArr[0], 11, 'newArr[0]');
assert(accessed, 'accessed !== true');
