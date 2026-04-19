// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - RegExp Object can be used as thisArg
---*/

var result = false;
var objRegExp = new RegExp();

function callbackfn(val, idx, obj) {
  result = (this === objRegExp);
}

[11].forEach(callbackfn, objRegExp);

assert(result, 'result !== true');
