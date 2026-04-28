// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - 'length' is an object that has an own
    valueOf method that returns an object and toString method that
    returns a string
---*/

var toStringAccessed = false;
var valueOfAccessed = false;

var obj = {
  1: true,
  length: {
    toString: function() {
      toStringAccessed = true;
      return '2';
    },

    valueOf: function() {
      valueOfAccessed = true;
      return {};
    }
  }
};

assert.sameValue(Array.prototype.indexOf.call(obj, true), 1, 'Array.prototype.indexOf.call(obj, true)');
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
