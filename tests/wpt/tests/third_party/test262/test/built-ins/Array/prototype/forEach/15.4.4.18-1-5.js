// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach applied to number primitive
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = obj instanceof Number;
}

Number.prototype[0] = 1;
Number.prototype.length = 1;

Array.prototype.forEach.call(2.5, callbackfn);

assert(result, 'result !== true');
