// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to number primitive
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Number;
}

Number.prototype[0] = 1;
Number.prototype.length = 1;

var newArr = Array.prototype.filter.call(2.5, callbackfn);

assert.sameValue(newArr[0], 1, 'newArr[0]');
