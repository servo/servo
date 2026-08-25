// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-86-1
description: >
    Object.defineProperty - 'Attributes' is a Function object which
    implements its own [[Get]] method to access the 'configurable'
    property of prototype object (8.10.5 step 4.a)
---*/

var obj = {};

Function.prototype.configurable = true;
var funObj = function(a, b) {
  return a + b;
};

Object.defineProperty(obj, "property", funObj);

var beforeDeleted = obj.hasOwnProperty("property");

delete obj.property;

var afterDeleted = obj.hasOwnProperty("property");

assert.sameValue(beforeDeleted, true, 'beforeDeleted');
assert.sameValue(afterDeleted, false, 'afterDeleted');
