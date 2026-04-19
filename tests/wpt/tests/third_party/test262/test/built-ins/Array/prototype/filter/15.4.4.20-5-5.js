// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-5-5
description: Array.prototype.filter - thisArg is object from object template
---*/

var res = false;

function callbackfn(val, idx, obj)
{
  return this.res;
}

function foo() {}
var f = new foo();
f.res = true;

var srcArr = [1];
var resArr = srcArr.filter(callbackfn, f);

assert.sameValue(resArr.length, 1, 'resArr.length');
