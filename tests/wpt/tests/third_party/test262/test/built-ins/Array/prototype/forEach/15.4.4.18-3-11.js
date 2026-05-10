// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
es5id: 15.4.4.18-3-11
description: >
    Array.prototype.forEach - 'length' is a string containing a
    positive number
---*/

var testResult = false;

function callbackfn(val, idx, obj) {
  testResult = (val > 10);
}

var obj = {
  1: 11,
  2: 9,
  length: "2"
};

Array.prototype.forEach.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
