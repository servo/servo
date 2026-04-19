// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - thisArg is Array
---*/

var res = false;
var a = new Array();
a.res = true;

function callbackfn(val, idx, obj)
{
  return this.res;
}

var srcArr = [1];
var resArr = srcArr.map(callbackfn, a);

assert.sameValue(resArr[0], true, 'resArr[0]');
