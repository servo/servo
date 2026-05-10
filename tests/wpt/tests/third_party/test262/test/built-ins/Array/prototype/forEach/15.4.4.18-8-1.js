// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach doesn't call callbackfn if 'length' is 0
    (empty array)
---*/

var callCnt = 0;

function cb() {
  callCnt++
}
var i = [].forEach(cb);

assert.sameValue(callCnt, 0, 'callCnt');
