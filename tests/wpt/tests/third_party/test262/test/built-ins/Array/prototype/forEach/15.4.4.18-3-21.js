// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.foreach
es5id: 15.4.4.18-3-21
description: >
    Array.prototype.forEach - 'length' is an object that has an own
    valueOf method that returns an object and toString method that
    returns a string
---*/

var testResult = false;
var firstStepOccured = false;
var secondStepOccured = false;

function callbackfn(val, idx, obj) {
  testResult = (val > 10);
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

Array.prototype.forEach.call(obj, callbackfn);

assert(testResult, 'testResult !== true');
assert(firstStepOccured, 'firstStepOccured !== true');
assert(secondStepOccured, 'secondStepOccured !== true');
