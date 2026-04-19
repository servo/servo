// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - Deleting the array itself within the
    callbackfn of Array.prototype.every is successful once
    Array.prototype.every is called for all elements
---*/

var o = new Object();
o.arr = [1, 2, 3, 4, 5];

function callbackfn(val, Idx, obj) {
  delete o.arr;
  if (val === Idx + 1)
    return true;
  else
    return false;
}

assert(o.arr.every(callbackfn), 'o.arr.every(callbackfn) !== true');
assert.sameValue(o.hasOwnProperty("arr"), false, 'o.hasOwnProperty("arr")');
