// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight - callbackfn uses arguments
---*/

function callbackfn() {
  return arguments[0] === 100 && arguments[3][arguments[2]] === arguments[1];
}

assert.sameValue([11].reduceRight(callbackfn, 100), true, '[11].reduceRight(callbackfn, 100)');
