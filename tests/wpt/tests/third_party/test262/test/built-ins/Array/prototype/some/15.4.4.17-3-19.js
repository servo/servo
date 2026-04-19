// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - value of 'length' is an Object which has an
    own toString method
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var toStringAccessed = false;
var obj = {
  0: 9,
  1: 11,
  2: 12,

  length: {
    toString: function() {
      toStringAccessed = true;
      return '2';
    }
  }
};

assert(Array.prototype.some.call(obj, callbackfn1), 'Array.prototype.some.call(obj, callbackfn1) !== true');
assert.sameValue(Array.prototype.some.call(obj, callbackfn2), false, 'Array.prototype.some.call(obj, callbackfn2)');
assert(toStringAccessed, 'toStringAccessed !== true');
