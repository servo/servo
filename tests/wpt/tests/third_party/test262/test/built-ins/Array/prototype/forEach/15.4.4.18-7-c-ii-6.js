// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - arguments to callbackfn are self
    consistent
---*/

var result = false;
var obj = {
  0: 11,
  length: 1
};
var thisArg = {};

function callbackfn() {
  result = (this === thisArg &&
    arguments[0] === 11 &&
    arguments[1] === 0 &&
    arguments[2] === obj);
}

Array.prototype.forEach.call(obj, callbackfn, thisArg);

assert(result, 'result !== true');
