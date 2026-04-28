// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-44
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is an object that
    has an own toString method that returns an object and toValue
    method that returns a primitive value
---*/

var obj = {
  "abc": 1
};
var valueOfAccessed = false;
var toStringAccessed = false;

var ownProp = {
  toString: function() {
    toStringAccessed = true;
    return {};
  },
  valueOf: function() {
    valueOfAccessed = true;
    return "abc";
  }
};

var desc = Object.getOwnPropertyDescriptor(obj, ownProp);

assert.sameValue(desc.value, 1, 'desc.value');
assert(valueOfAccessed, 'valueOfAccessed !== true');
assert(toStringAccessed, 'toStringAccessed !== true');
