// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to boolean primitive
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Boolean;
}

Boolean.prototype[0] = true;
Boolean.prototype.length = 1;

var newArr = Array.prototype.filter.call(false, callbackfn);

assert.sameValue(newArr[0], true, 'newArr[0]');
