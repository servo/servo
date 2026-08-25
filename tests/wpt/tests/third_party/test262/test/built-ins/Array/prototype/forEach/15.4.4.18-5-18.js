// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - Error Object can be used as thisArg
---*/

var result = false;
var objError = new RangeError();

function callbackfn(val, idx, obj) {
  result = (this === objError);
}

[11].forEach(callbackfn, objError);

assert(result, 'result !== true');
