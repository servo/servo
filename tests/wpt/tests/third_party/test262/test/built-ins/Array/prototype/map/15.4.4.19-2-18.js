// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - applied to String object, which implements
    its own property get method
---*/

function callbackfn(val, idx, obj) {
  return parseInt(val, 10) > 1;
}

var str = new String("432");

String.prototype[3] = "1";
var testResult = Array.prototype.map.call(str, callbackfn);

assert.sameValue(testResult.length, 3, 'testResult.length');
