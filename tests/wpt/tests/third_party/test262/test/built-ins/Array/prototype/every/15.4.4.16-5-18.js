// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-18
description: Array.prototype.every - Error Object can be used as thisArg
---*/

var accessed = false;
var objError = new RangeError();

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === objError;
}

assert([11].every(callbackfn, objError), '[11].every(callbackfn, objError) !== true');
assert(accessed, 'accessed !== true');
