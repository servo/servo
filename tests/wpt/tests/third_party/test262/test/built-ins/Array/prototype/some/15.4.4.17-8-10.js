// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - subclassed array when length is reduced
---*/

foo.prototype = new Array(1, 2, 3);

function foo() {}
var f = new foo();
f.length = 2;

function cb(val)
{
  if (val > 2)
    return true;
  else
    return false;
}
var i = f.some(cb);


assert.sameValue(i, false, 'i');
