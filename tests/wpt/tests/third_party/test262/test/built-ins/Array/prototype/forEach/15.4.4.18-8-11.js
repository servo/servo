// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach doesn't mutate the array on which it is
    called on
---*/

function callbackfn(val, idx, obj)
{
  return true;
}
var arr = [1, 2, 3, 4, 5];
arr.forEach(callbackfn);

assert.sameValue(arr[0], 1, 'arr[0]');
assert.sameValue(arr[1], 2, 'arr[1]');
assert.sameValue(arr[2], 3, 'arr[2]');
assert.sameValue(arr[3], 4, 'arr[3]');
assert.sameValue(arr[4], 5, 'arr[4]');
