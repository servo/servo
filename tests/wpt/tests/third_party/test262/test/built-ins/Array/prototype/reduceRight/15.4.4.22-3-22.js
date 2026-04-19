// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
    Array.prototype.reduceRight throws TypeError exception when
    'length' is an object with toString and valueOf methods that don�t
    return primitive values
---*/

var accessed = false;
var toStringAccessed = false;
var valueOfAccessed = false;

function callbackfn(prevVal, curVal, idx, obj) {
  accessed = true;
}

var obj = {
  0: 11,
  1: 12,

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
  Array.prototype.reduceRight.call(obj, callbackfn, 1);
});
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(accessed, false, 'accessed');
