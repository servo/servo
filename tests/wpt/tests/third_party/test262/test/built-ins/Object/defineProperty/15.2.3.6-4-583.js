// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-583
description: >
    ES5 Attributes - [[Get]] field of inherited property of
    [[Prototype]] internal property is correct (Error Instance)
---*/

var data = "data";

Object.defineProperty(Error.prototype, "prop", {
  get: function() {
    return data;
  },
  set: function(value) {
    data = value;
  },
  enumerable: true,
  configurable: true
});
var errObj = new Error();

assert.sameValue(errObj.hasOwnProperty("prop"), false, 'errObj.hasOwnProperty("prop")');
assert.sameValue(errObj.prop, "data", 'errObj.prop');
