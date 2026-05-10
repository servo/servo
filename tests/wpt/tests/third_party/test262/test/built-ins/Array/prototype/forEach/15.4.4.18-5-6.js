// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: Array.prototype.forEach - thisArg is function
---*/

var res = false;
var result;

function callbackfn(val, idx, obj)
{
  result = this.res;
}

function foo() {}
foo.res = true;

var arr = [1];
arr.forEach(callbackfn, foo)

assert.sameValue(result, true, 'result');
