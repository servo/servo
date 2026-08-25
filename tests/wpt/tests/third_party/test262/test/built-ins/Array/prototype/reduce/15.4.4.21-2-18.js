// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce applied to String object, which implements
    its own property get method
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return (obj.length === 3);
}

var str = new String("012");

assert.sameValue(Array.prototype.reduce.call(str, callbackfn, 1), true, 'Array.prototype.reduce.call(str, callbackfn, 1)');
