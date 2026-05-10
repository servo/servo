// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce throws TypeError exception - 'length' is an
    object with toString and valueOf methods that don�t return
    primitive values
---*/

var accessed = false;
var valueOfAccessed = false;
var toStringAccessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
  return true;
}

var obj = {
  1: 11,
  2: 12,

  length: {
    valueOf: function() {
      valueOfAccessed = true;
      return {};
    },
    toString: function() {
      toStringAccessed = true;
      return {};
    }
  }
};
assert.throws(TypeError, function() {
  Array.prototype.reduce.call(obj, callbackfn, 1);
});
assert.sameValue(accessed, false, 'accessed');
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
