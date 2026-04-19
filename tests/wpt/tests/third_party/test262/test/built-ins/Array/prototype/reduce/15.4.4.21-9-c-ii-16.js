// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce - non-indexed properties are not called
---*/

var accessed = false;
var result1 = true;
var result2 = true;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  if (curVal === 8) {
    result1 = false;
  }

  if (prevVal === 8) {
    result2 = false;
  }
}

var obj = {
  0: 11,
  10: 12,
  non_index_property: 8,
  length: 20
};

Array.prototype.reduce.call(obj, callbackfn, 1);

assert(result1, 'result1 !== true');
assert(result2, 'result2 !== true');
assert(accessed, 'accessed !== true');
