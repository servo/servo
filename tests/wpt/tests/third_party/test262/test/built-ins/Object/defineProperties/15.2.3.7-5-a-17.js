// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-a-17
description: >
    Object.defineProperties - 'Properties' is the Arguments object
    which implements its own [[Get]] method to get enumerable own
    property
---*/

var obj = {};
var arg;

(function fun() {
  arg = arguments;
}());

Object.defineProperty(arg, "prop", {
  value: {
    value: 17
  },
  enumerable: true
});

Object.defineProperties(obj, arg);

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 17, 'obj.prop');
