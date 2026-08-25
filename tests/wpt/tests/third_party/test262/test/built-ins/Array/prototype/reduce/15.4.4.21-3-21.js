// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - 'length' is an object that has an own
    valueOf method that returns an object and toString method that
    returns a string
---*/

var valueOfOccured = false;
var toStringOccured = false;

function callbackfn(prevVal, curVal, idx, obj) {
  return (curVal === 11 && idx === 1);
}

var obj = {
  1: 11,
  2: 9,
  length: {
    valueOf: function() {
      valueOfOccured = true;
      return {};
    },
    toString: function() {
      toStringOccured = true;
      return '2';
    }
  }
};

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, 1), true, 'Array.prototype.reduce.call(obj, callbackfn, 1)');
assert(valueOfOccured, 'valueOfOccured !== true');
assert(toStringOccured, 'toStringOccured !== true');
