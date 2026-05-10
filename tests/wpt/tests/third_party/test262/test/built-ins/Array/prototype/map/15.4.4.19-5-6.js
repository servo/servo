// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: Array.prototype.map - thisArg is function
---*/

var res = false;

function callbackfn(val, idx, obj)
{
  return this.res;
}

function foo() {}
foo.res = true;

var srcArr = [1];
var resArr = srcArr.map(callbackfn, foo);

assert.sameValue(resArr[0], true, 'resArr[0]');
