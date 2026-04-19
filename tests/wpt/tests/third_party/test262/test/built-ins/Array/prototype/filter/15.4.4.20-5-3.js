// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - thisArg is Array
---*/

var res = false;
var a = new Array();
a.res = true;

function callbackfn(val, idx, obj)
{
  return this.res;
}

var srcArr = [1];
var resArr = srcArr.filter(callbackfn, a);

assert.sameValue(resArr.length, 1, 'resArr.length');
