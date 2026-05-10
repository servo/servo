// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter throws TypeError exception when 'length' is
    an object with toString and valueOf methods that don�t return
    primitive values
---*/

var accessed = false;
var firstStepOccured = false;
var secondStepOccured = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return true;
}

var obj = {
  1: 11,
  2: 12,

  length: {
    valueOf: function() {
      firstStepOccured = true;
      return {};
    },
    toString: function() {
      secondStepOccured = true;
      return {};
    }
  }
};
assert.throws(TypeError, function() {
  Array.prototype.filter.call(obj, callbackfn);
});
assert.sameValue(accessed, false, 'accessed');
assert(firstStepOccured, 'firstStepOccured !== true');
assert(secondStepOccured, 'secondStepOccured !== true');
