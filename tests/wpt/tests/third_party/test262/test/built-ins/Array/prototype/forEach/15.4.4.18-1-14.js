// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach applied to Error object
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = obj instanceof Error;
}

var obj = new Error();
obj.length = 1;
obj[0] = 1;

Array.prototype.forEach.call(obj, callbackfn);

assert(result, 'result !== true');
