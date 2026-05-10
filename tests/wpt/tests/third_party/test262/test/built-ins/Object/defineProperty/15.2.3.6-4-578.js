// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-578
description: >
    ES5 Attributes - [[Get]] field of inherited property of
    [[Prototype]] internal property is correct (String instance)
---*/

var data = "data";

Object.defineProperty(String.prototype, "prop", {
  get: function() {
    return data;
  },
  set: function(value) {
    data = value;
  },
  enumerable: true,
  configurable: true
});
var strObj = new String();

assert.sameValue(strObj.hasOwnProperty("prop"), false, 'strObj.hasOwnProperty("prop")');
assert.sameValue(strObj.prop, "data", 'strObj.prop');
