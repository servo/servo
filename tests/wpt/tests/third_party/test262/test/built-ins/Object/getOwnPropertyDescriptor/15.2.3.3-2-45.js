// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-45
description: >
    Object.getOwnPropertyDescriptor - argument 'P' is an object which
    has an own toString and valueOf method
---*/

var obj = {
  "bbq": 1,
  "abc": 2
};
var valueOfAccessed = false;

var ownProp = {
  toString: function() {
    return "bbq";
  },
  valueOf: function() {
    valueOfAccessed = true;
    return "abc";
  }
};

var desc = Object.getOwnPropertyDescriptor(obj, ownProp);

assert.sameValue(desc.value, 1, 'desc.value');
assert.sameValue(valueOfAccessed, false, 'valueOfAccessed');
