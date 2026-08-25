// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - the JSON object can be used as thisArg
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (this === JSON);
}

[11].forEach(callbackfn, JSON);

assert(result, 'result !== true');
