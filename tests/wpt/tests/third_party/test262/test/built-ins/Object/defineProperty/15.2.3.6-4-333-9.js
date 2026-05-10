// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-333-9
description: >
    Object.defineProperty - Named property 'P' with attributes
    [[Writable]]: true, [[Enumerable]]:true, [[Configurable]]:false is
    writable using simple assignment, 'A' is an Array Object
---*/

var obj = [];

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});
var verifyValue = (obj.prop === 2010);
obj.prop = 1001;

assert(verifyValue, 'verifyValue !== true');
assert.sameValue(obj.prop, 1001, 'obj.prop');
