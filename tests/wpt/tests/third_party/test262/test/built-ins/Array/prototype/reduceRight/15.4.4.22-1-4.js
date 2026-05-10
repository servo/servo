// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: Array.prototype.reduceRight applied to Boolean object
---*/

var obj = new Boolean(true);
obj.length = 2;
obj[0] = 11;
obj[1] = 12;
var accessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return obj instanceof Boolean;
}

assert(Array.prototype.reduceRight.call(obj, callbackfn, 11), 'Array.prototype.reduceRight.call(obj, callbackfn, 11) !== true');
assert(accessed, 'accessed !== true');
