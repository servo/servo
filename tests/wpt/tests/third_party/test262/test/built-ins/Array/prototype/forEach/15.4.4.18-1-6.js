// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach applied to Number object
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = obj instanceof Number;
}

var obj = new Number(-128);
obj.length = 2;
obj[0] = 11;
obj[1] = 12;

Array.prototype.forEach.call(obj, callbackfn);

assert(result, 'result !== true');
