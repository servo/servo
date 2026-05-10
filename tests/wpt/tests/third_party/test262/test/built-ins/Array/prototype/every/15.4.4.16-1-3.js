// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every applied to boolean primitive
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return obj instanceof Boolean;
}

Boolean.prototype[0] = 1;
Boolean.prototype.length = 1;

assert(Array.prototype.every.call(false, callbackfn), 'Array.prototype.every.call(false, callbackfn) !== true');
assert(accessed, 'accessed !== true');
