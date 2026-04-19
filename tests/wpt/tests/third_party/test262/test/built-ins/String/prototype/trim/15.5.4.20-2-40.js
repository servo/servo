// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.5.4.20-2-40
description: >
    String.prototype.trim - 'this' is an object that has an own
    toString method that returns an object and valueOf method that
    returns a primitive value
---*/

var toStringAccessed = false;
var valueOfAccessed = false;
var obj = {
  toString: function() {
    toStringAccessed = true;
    return {};
  },
  valueOf: function() {
    valueOfAccessed = true;
    return "abc";
  }
};

assert.sameValue(String.prototype.trim.call(obj), "abc", 'String.prototype.trim.call(obj)');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert(toStringAccessed, 'toStringAccessed !== true');
