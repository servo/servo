// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach applied to Array-like object, 'length' is
    an own accessor property
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = (obj.length === 2);
}

var obj = {};

Object.defineProperty(obj, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

obj[0] = 12;
obj[1] = 11;
obj[2] = 9;

Array.prototype.forEach.call(obj, callbackfn);

assert(result, 'result !== true');
