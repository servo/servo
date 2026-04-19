// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - callbackfn is called with 0 formal
    parameter
---*/

var called = 0;

function callbackfn() {
  called++;
  return true;
}

assert.sameValue([11, 12].reduceRight(callbackfn, 11), true, '[11, 12].reduceRight(callbackfn, 11)');
assert.sameValue(called, 2, 'called');
