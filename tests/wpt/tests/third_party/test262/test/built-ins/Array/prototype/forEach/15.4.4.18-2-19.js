// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach applied to Function object, which
    implements its own property get method
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 2);
}

var fun = function(a, b) {
  return a + b;
};
fun[0] = 12;
fun[1] = 11;
fun[2] = 9;

Array.prototype.forEach.call(fun, callbackfn);

assert(result, 'result !== true');
