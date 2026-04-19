// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - applied to String object, which
    implements its own property get method
---*/

var testResult = false;
var initialValue = 0;

function callbackfn(prevVal, curVal, idx, obj) {
  if (idx === 1) {
    testResult = (curVal === "1");
  }
}

var str = new String("012");

Array.prototype.reduce.call(str, callbackfn, initialValue);

assert(testResult, 'testResult !== true');
