// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - subclassed array when length is reduced
---*/

foo.prototype = new Array(1, 2, 3);

function foo() {}
var f = new foo();
f.length = 1;

var callCnt = 0;

function cb() {
  callCnt++
}
var i = f.forEach(cb);

assert.sameValue(callCnt, 1, 'callCnt');
