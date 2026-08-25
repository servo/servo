// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - subclassed array when length is reduced
---*/

foo.prototype = new Array(1, 2, 3);

function foo() {}
var f = new foo();
f.length = 2;

function cb(val)
{
  if (val > 2)
    return false;
  else
    return true;
}
var i = f.every(cb);


assert.sameValue(i, true, 'i');
