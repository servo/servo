// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight - subclassed array when initialvalue
    provided
---*/

foo.prototype = new Array(0, 1, 2, 3);

function foo() {}
var f = new foo();

function cb(prevVal, curVal, idx, obj) {
  return prevVal + curVal;
}

assert.sameValue(f.reduceRight(cb, "4"), "43210", 'f.reduceRight(cb,"4")');
