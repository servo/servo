// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: Array.prototype.some - thisArg is Array
---*/

var res = false;
var a = new Array();
a.res = true;

function callbackfn(val, idx, obj)
{
  return this.res;
}

var arr = [1];

assert.sameValue(arr.some(callbackfn, a), true, 'arr.some(callbackfn, a)');
