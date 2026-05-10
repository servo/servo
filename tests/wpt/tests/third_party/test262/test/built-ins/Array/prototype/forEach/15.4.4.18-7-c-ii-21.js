// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - callbackfn called with correct
    parameters (kValue is correct)
---*/

var resultOne = false;
var resultTwo = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    resultOne = (val === 11);
  }

  if (idx === 1) {
    resultTwo = (val === 12);
  }

}

var obj = {
  0: 11,
  1: 12,
  length: 2
};

Array.prototype.forEach.call(obj, callbackfn);

assert(resultOne, 'resultOne !== true');
assert(resultTwo, 'resultTwo !== true');
