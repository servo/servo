// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - 'length' is an object that has an own
    valueOf method that returns an object and toString method that
    returns a string
---*/

var firstStepOccured = false;
var secondStepOccured = false;

function callbackfn(val, idx, obj) {
  return true;
}

var obj = {
  1: 11,
  2: 9,
  length: {
    valueOf: function() {
      firstStepOccured = true;
      return {};
    },
    toString: function() {
      secondStepOccured = true;
      return '2';
    }
  }
};

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
assert(firstStepOccured, 'firstStepOccured !== true');
assert(secondStepOccured, 'secondStepOccured !== true');
