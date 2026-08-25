// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight applied to String object
---*/

var obj = new String("hello\nworld\\!");
var accessed = false;

function callbackfn(prevVal, curVal, idx, o) {
  accessed = true;
  return o instanceof String;
}

assert(Array.prototype.reduceRight.call(obj, callbackfn, "h"), 'Array.prototype.reduceRight.call(obj, callbackfn, "h") !== true');
assert(accessed, 'accessed !== true');
