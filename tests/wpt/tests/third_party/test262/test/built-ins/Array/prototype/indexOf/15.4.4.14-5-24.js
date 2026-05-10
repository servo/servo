// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf throws TypeError exception when value of
    'fromIndex' is an object with toString and valueOf methods that
    don�t return primitive values
---*/

var toStringAccessed = false;
var valueOfAccessed = false;
var fromIndex = {
  toString: function() {
    toStringAccessed = true;
    return {};
  },

  valueOf: function() {
    valueOfAccessed = true;
    return {};
  }
};

assert.throws(TypeError, function() {
  [0, true].indexOf(true, fromIndex);
});

assert(toStringAccessed, 'toStringAccessed');
assert(valueOfAccessed, 'valueOfAccessed');
