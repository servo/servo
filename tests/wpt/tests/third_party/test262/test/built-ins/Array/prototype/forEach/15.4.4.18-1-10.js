// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach applied to the Math object
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = ('[object Math]' === Object.prototype.toString.call(obj));
}

Math.length = 1;
Math[0] = 1;
Array.prototype.forEach.call(Math, callbackfn);

assert(result, 'result !== true');
