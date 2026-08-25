// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - element to be retrieved is inherited data
    property on an Array
---*/

var kValue = {};

function callbackfn(val, idx, obj) {
  if (0 === idx) {
    return kValue === val;
  }
  return false;
}

Array.prototype[0] = kValue;

assert([, ].some(callbackfn), '[, ].some(callbackfn) !== true');
