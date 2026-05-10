// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - element to be retrieved is inherited
    data property on an Array
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  if (idx === 1) {
    testResult = (val === 13);
  }
}

Array.prototype[1] = 13;

[, , , ].forEach(callbackfn);

assert(testResult, 'testResult !== true');
