// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
es5id: 15.4.4.20-5-4
description: >
    Array.prototype.filter - thisArg is object from object
    template(prototype)
---*/

var res = false;

function callbackfn(val, idx, obj)
{
  return this.res;
}

function foo() {}
foo.prototype.res = true;
var f = new foo();

var srcArr = [1];
var resArr = srcArr.filter(callbackfn, f);

assert.sameValue(resArr.length, 1, 'resArr.length');
