// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - getOwnPropertyDescriptor(all true) of
    returned array element
---*/

function callbackfn(val, idx, obj) {
  if (val % 2)
    return true;
  else
    return false;
}
var srcArr = [0, 1, 2, 3, 4];
var resArr = srcArr.filter(callbackfn);

assert(resArr.length > 0, 'resArr.length > 0');

var desc = Object.getOwnPropertyDescriptor(resArr, 1);

assert.sameValue(desc.value, 3, 'desc.value'); //srcArr[1] = true
assert.sameValue(desc.writable, true, 'desc.writable');
assert.sameValue(desc.enumerable, true, 'desc.enumerable');
assert.sameValue(desc.configurable, true, 'desc.configurable');
