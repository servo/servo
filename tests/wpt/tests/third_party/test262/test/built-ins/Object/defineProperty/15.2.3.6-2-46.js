// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-2-46
description: >
    Object.defineProperty - argument 'P' is an object that has an own
    toString and valueOf method
---*/

var obj = {};
var toStringAccessed = false;
var valueOfAccessed = false;

var ownProp = {
  toString: function() {
    toStringAccessed = true;
    return "abc";
  },
  valueOf: function() {
    valueOfAccessed = true;
    return "prop";
  }
};
Object.defineProperty(obj, ownProp, {});

assert(obj.hasOwnProperty("abc"), 'obj.hasOwnProperty("abc") !== true');
assert.sameValue(valueOfAccessed, false, 'valueOfAccessed');
assert(toStringAccessed, 'toStringAccessed !== true');
