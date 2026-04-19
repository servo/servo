// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element changed by callbackfn on
    previous iterations is observed
---*/

var result = false;
var obj = {
  0: 11,
  1: 12,
  length: 2
};

function callbackfn(val, idx, o) {
  if (idx === 0) {
    obj[idx + 1] = 8;
  }

  if (idx === 1) {
    result = (val === 8);
  }
}

Array.prototype.forEach.call(obj, callbackfn);

assert(result, 'result !== true');
