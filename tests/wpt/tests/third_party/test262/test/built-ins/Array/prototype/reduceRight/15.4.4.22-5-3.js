// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight throws TypeError if 'length' is 0
    (subclassed Array, length overridden to false (type conversion)),
    no initVal
---*/

foo.prototype = new Array(1, 2, 3);

function foo() {}
var f = new foo();
f.length = false;

function cb() {}
assert.throws(TypeError, function() {
  f.reduceRight(cb);
});
