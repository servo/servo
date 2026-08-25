// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - thisArg is object from object
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
var arr = [1];

assert.sameValue(arr.some(callbackfn, f), true, 'arr.some(callbackfn,f)');
