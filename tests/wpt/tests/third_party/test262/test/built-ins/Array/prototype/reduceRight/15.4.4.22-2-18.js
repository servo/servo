// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight applied to String object, which
    implements its own property get method
---*/

var accessed = false;
var str = new String("432");

function callbackfn(preVal, curVal, idx, obj) {
  accessed = true;
  return obj.length === 3;
}

String.prototype[3] = "1";

assert(Array.prototype.reduceRight.call(str, callbackfn, 111), 'Array.prototype.reduceRight.call(str, callbackfn, 111) !== true');
assert(accessed, 'accessed !== true');
