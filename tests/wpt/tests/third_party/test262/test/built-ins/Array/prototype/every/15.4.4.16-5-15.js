// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-15
description: Array.prototype.every - Date Object can be used as thisArg
---*/

var accessed = false;
var objDate = new Date(0);

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === objDate;
}

assert([11].every(callbackfn, objDate), '[11].every(callbackfn, objDate) !== true');
assert(accessed, 'accessed !== true');
