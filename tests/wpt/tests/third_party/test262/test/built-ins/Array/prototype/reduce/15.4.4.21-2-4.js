// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - 'length' is own data property that
    overrides an inherited data property on an Array
---*/

var storeProtoLength;

function callbackfn(prevVal, curVal, idx, obj) {
  return (obj.length === 2);
}

storeProtoLength = Array.prototype.length;
Array.prototype.length = 0;

assert.sameValue([12, 11].reduce(callbackfn, 1), true, '[12, 11].reduce(callbackfn, 1)');
