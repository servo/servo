// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - thisArg is Object
---*/

var res = false;
var o = new Object();
o.res = true;

function callbackfn(val, idx, obj)
{
  return this.res;
}

var srcArr = [1];
var resArr = srcArr.map(callbackfn, o);

assert.sameValue(resArr[0], true, 'resArr[0]');
