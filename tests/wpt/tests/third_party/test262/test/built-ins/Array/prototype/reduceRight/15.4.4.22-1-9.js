// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight applied to Function object
---*/

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;
var accessed = false;

function callbackfn(prevVal, curVal, idx, o) {
  accessed = true;
  return o instanceof Function;
}

assert(Array.prototype.reduceRight.call(obj, callbackfn, 11), 'Array.prototype.reduceRight.call(obj, callbackfn, 11) !== true');
assert(accessed, 'accessed !== true');
