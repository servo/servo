// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - thisArg is Object
---*/

var res = false;
var o = new Object();
o.res = true;
var result;

function callbackfn(val, idx, obj)
{
  result = this.res;
}

var arr = [1];
arr.forEach(callbackfn, o)

assert.sameValue(result, true, 'result');
