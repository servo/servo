// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - value of 'fromIndex' is an object that
    has an own valueOf method that returns an object and toString
    method that returns a string
---*/

var toStringAccessed = false;
var valueOfAccessed = false;

var fromIndex = {
  toString: function() {
    toStringAccessed = true;
    return '1';
  },

  valueOf: function() {
    valueOfAccessed = true;
    return {};
  }
};

assert.sameValue([0, true].indexOf(true, fromIndex), 1, '[0, true].indexOf(true, fromIndex)');
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
