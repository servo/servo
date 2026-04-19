// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf throws TypeError exception when value
    of 'fromIndex' is an object that both toString and valueOf methods
    than don't return primitive value
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
  [0, null].lastIndexOf(null, fromIndex);
});

assert(toStringAccessed, 'toStringAccessed');
assert(valueOfAccessed, 'valueOfAccessed');
