// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - element changed by callbackfn on previous
    iterations is observed
---*/

var result = false;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 0) {
    obj[idx + 1] = 8;
  }

  if (idx === 1) {
    result = (curVal === 8);
  }
}

var obj = {
  0: 11,
  1: 12,
  length: 2
};

Array.prototype.reduce.call(obj, callbackfn, 1);

assert(result, 'result !== true');
