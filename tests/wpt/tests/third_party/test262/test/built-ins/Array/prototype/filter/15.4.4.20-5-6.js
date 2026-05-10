// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-5-6
description: Array.prototype.filter - thisArg is function
---*/

var res = false;

function callbackfn(val, idx, obj)
{
  return this.res;
}

function foo() {}
foo.res = true;

var srcArr = [1];
var resArr = srcArr.filter(callbackfn, foo);

assert.sameValue(resArr.length, 1, 'resArr.length');
