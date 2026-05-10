// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - 'length' is an object that has an own
    valueOf method that returns an object and toString method that
    returns a string
---*/

function callbackfn(val, idx, obj) {
  return val < 10;
}

var firstStepOccured = false;
var secondStepOccured = false;
var obj = {
  0: 11,
  1: 9,

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

var newArr = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
assert(firstStepOccured, 'firstStepOccured !== true');
assert(secondStepOccured, 'secondStepOccured !== true');
