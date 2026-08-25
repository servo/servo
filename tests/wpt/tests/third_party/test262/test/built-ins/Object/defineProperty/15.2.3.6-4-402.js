// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-402
description: >
    ES5 Attributes - [[Value]] attribute of inherited property of
    [[Prototype]] internal property is correct (String instance)
---*/

Object.defineProperty(String.prototype, "prop", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});
var strObj = new String();

assert.sameValue(strObj.hasOwnProperty("prop"), false, 'strObj.hasOwnProperty("prop")');
assert.sameValue(strObj.prop, 1001, 'strObj.prop');
