// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is an object that has an
    own valueOf method that returns an object and toString method that
    returns a string
---*/

var toStringAccessed = false;
var valueOfAccessed = false;

var targetObj = this;
var obj = {
  1: targetObj,
  length: {
    toString: function() {
      toStringAccessed = true;
      return '3';
    },

    valueOf: function() {
      valueOfAccessed = true;
      return {};
    }
  }
};

assert.sameValue(Array.prototype.lastIndexOf.call(obj, targetObj), 1, 'Array.prototype.lastIndexOf.call(obj, targetObj)');
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
