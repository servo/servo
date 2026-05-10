// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
es5id: 15.4.4.16-5-12
description: Array.prototype.every - Boolean Object can be used as thisArg
---*/

var accessed = false;
var objBoolean = new Boolean();

function callbackfn(val, idx, obj) {
  accessed = true;
  return this === objBoolean;
}



assert([11].every(callbackfn, objBoolean), '[11].every(callbackfn, objBoolean) !== true');
assert(accessed, 'accessed !== true');
