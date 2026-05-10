// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-13
description: Array.prototype.every - Number Object can be used as thisArg
---*/

var accessed = false;
var objNumber = new Number();

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === objNumber;
}

assert([11].every(callbackfn, objNumber), '[11].every(callbackfn, objNumber) !== true');
assert(accessed, 'accessed !== true');
