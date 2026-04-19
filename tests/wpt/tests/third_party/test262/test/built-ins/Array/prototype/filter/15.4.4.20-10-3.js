// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - subclassed array when length is reduced
---*/

foo.prototype = new Array(1, 2, 3);

function foo() {}
var f = new foo();
f.length = 1;

function cb() {
  return true;
}
var a = f.filter(cb);


assert(Array.isArray(a), 'Array.isArray(a) !== true');
assert.sameValue(a.length, 1, 'a.length');
