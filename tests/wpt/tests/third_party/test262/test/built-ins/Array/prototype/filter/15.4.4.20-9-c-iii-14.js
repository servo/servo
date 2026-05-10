// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter return value of callbackfn is an empty
    string
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return "";
}

var newArr = [11].filter(callbackfn);

assert.sameValue(newArr.length, 0, 'newArr.length');
assert(accessed, 'accessed !== true');
