// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every - thisArg is function
---*/

var res = false;

function callbackfn(val, idx, obj)
{
  return this.res;
}

function foo() {}
foo.res = true;
var arr = [1];

assert.sameValue(arr.every(callbackfn, foo), true, 'arr.every(callbackfn,foo)');
