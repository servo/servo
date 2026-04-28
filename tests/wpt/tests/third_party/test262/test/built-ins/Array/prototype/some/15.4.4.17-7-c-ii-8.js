// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element changed by callbackfn on previous
    iterations is observed
---*/

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    obj[idx + 1] = 11;
  }
  return val > 10;
}

var obj = {
  0: 9,
  1: 8,
  length: 2
};

assert(Array.prototype.some.call(obj, callbackfn), 'Array.prototype.some.call(obj, callbackfn) !== true');
