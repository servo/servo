// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach applied to String object, which implements
    its own property get method
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 3);
}

var str = new String("012");

Array.prototype.forEach.call(str, callbackfn);

assert(result, 'result !== true');
