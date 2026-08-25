// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some throws TypeError exception when 'length' is
    an object with toString and valueOf methods that don�t return
    primitive values
---*/

var callbackfnAccessed = false;
var toStringAccessed = false;
var valueOfAccessed = false;

function callbackfn(val, idx, obj) {
  callbackfnAccessed = true;
  return val > 10;
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
  Array.prototype.some.call(obj, callbackfn);
});
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert.sameValue(callbackfnAccessed, false, 'callbackfnAccessed');
