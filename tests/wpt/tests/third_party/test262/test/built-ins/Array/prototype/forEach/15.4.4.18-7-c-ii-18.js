// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
description: >
    Array.prototype.forEach - 'this' of 'callbackfn' is an String
    object when T is not an object (T is a string)
---*/

var result = false;

function callbackfn(val, idx, obj) {
  result = ('hello' === this.valueOf());
}

var obj = {
  0: 11,
  length: 2
};

Array.prototype.forEach.call(obj, callbackfn, "hello");

assert(result, 'result !== true');
