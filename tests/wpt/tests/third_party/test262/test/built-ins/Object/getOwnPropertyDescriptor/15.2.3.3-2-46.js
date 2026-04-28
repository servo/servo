// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.3-2-46
description: >
    Object.getOwnPropertyDescriptor - TypeError exception was thrown
    when 'P' is an object that both toString and valueOf wouldn't
    return primitive value
---*/

var obj = {
  "1": 1
};
var toStringAccessed = false;
var valueOfAccessed = false;

var ownProp = {
  toString: function() {
    toStringAccessed = true;
    return [1];
  },
  valueOf: function() {
    valueOfAccessed = true;
    return [1];
  }
};
assert.throws(TypeError, function() {
  Object.getOwnPropertyDescriptor(obj, ownProp);
});
assert(toStringAccessed, 'toStringAccessed !== true');
assert(valueOfAccessed, 'valueOfAccessed !== true');
