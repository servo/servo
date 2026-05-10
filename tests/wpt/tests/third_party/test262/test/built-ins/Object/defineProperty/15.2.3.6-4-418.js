// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-418
description: >
    ES5 Attributes - Successfully add a property to an object when the
    object's prototype has a property with the same name and
    [[Writable]] set to true (Function.prototype.bind)
---*/

var foo = function() {};

Object.defineProperty(Function.prototype, "prop", {
  value: 1001,
  writable: true,
  enumerable: true,
  configurable: true
});

var obj = foo.bind({});
obj.prop = 1002;

assert(obj.hasOwnProperty("prop"), 'obj.hasOwnProperty("prop") !== true');
assert.sameValue(obj.prop, 1002, 'obj.prop');
