// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - callbackfn is called with 2 formal
    parameter
---*/

var result = false;

function callbackfn(prevVal, curVal) {
  result = (curVal > 10 && 1 === prevVal);
}

[11].reduce(callbackfn, 1);

assert(result, 'result !== true');
