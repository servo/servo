// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - 'length' is own data property on an
    Array-like object
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 2);
}

var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: 2
};

Array.prototype.forEach.call(obj, callbackfn);

assert(result, 'result !== true');
