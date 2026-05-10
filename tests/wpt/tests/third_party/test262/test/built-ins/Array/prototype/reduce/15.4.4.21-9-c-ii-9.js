// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - callbackfn is called with 0 formal
    parameter
---*/

var called = 0;

function callbackfn() {
  called++;
}

[11, 12].reduce(callbackfn, 1);

assert.sameValue(called, 2, 'called');
