// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to string primitive
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof String;
}

var newArr = Array.prototype.filter.call("abc", callbackfn);

assert.sameValue(newArr[0], "a", 'newArr[0]');
