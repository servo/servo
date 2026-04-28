// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - 'length' is a string containing -Infinity
---*/

var accessed2 = false;

function callbackfn2(prevVal, curVal, idx, obj) {
  accessed2 = true;
  return 2;
}

var obj2 = {
  0: 9,
  length: "-Infinity"
};

assert.sameValue(Array.prototype.reduce.call(obj2, callbackfn2, 1), 1, 'Array.prototype.reduce.call(obj2, callbackfn2, 1)');
assert.sameValue(accessed2, false, 'accessed2');
