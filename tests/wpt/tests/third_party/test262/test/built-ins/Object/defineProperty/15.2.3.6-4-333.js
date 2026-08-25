// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-333
description: >
    ES5 Attributes - property ([[Writable]] is true, [[Enumerable]] is
    true, [[Configurable]] is false) is writable
---*/

var obj = {};

Object.defineProperty(obj, "prop", {
  value: 2010,
  writable: true,
  enumerable: true,
  configurable: false
});
var propertyDefineCorrect = (obj.prop === 2010);
obj.prop = 1001;

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(obj.prop, 1001, 'obj.prop');
