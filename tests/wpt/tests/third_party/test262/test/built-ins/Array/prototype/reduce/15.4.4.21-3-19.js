// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - value of 'length' is an Object which has
    an own toString method
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return (curVal === 11 && idx === 1);
}

var obj = {
  1: 11,
  2: 9,
  length: {
    toString: function() {
      return '2';
    }
  }
};

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, 1), true, 'Array.prototype.reduce.call(obj, callbackfn, 1)');
