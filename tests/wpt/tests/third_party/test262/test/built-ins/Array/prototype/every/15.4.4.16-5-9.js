// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - Function Object can be used as thisArg
---*/

var accessed = false;
var objFunction = function() {};

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === objFunction;
}

assert([11].every(callbackfn, objFunction), '[11].every(callbackfn, objFunction) !== true');
assert(accessed, 'accessed !== true');
