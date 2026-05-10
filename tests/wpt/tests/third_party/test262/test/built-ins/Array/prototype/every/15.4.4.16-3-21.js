// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - 'length' is an object that has an own
    valueOf method that returns an object and toString method that
    returns a string
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var toStringAccessed = false;
var valueOfAccessed = false;

var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: {
    valueOf: function() {
      valueOfAccessed = true;
      return {};
    },
    toString: function() {
      toStringAccessed = true;
      return '2';
    }
  }
};

assert(Array.prototype.every.call(obj, callbackfn1), 'Array.prototype.every.call(obj, callbackfn1) !== true');
assert.sameValue(Array.prototype.every.call(obj, callbackfn2), false, 'Array.prototype.every.call(obj, callbackfn2)');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert(toStringAccessed, 'toStringAccessed !== true');
