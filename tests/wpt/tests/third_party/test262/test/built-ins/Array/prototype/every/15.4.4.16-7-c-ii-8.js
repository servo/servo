// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element changed by callbackfn on previous
    iterations is observed
---*/

var obj = {
  0: 11,
  1: 12,
  length: 2
};

function callbackfn(val, idx, o) {
  if (idx === 0) {
    obj[idx + 1] = 8;
  }
  return val > 10;
}



assert.sameValue(Array.prototype.every.call(obj, callbackfn), false, 'Array.prototype.every.call(obj, callbackfn)');
