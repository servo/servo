// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every applied to Boolean object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return obj instanceof Boolean;
}

var obj = new Boolean(true);
obj.length = 2;
obj[0] = 11;
obj[1] = 12;

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert(accessed, 'accessed !== true');
