// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-98
description: >
    Object.defineProperty - 'Attributes' is the global object that
    uses Object's [[Get]] method to access the 'configurable' property
    (8.10.5 step 4.a)
---*/

var obj = {};

this.configurable = true;

Object.defineProperty(obj, "property", this);

var beforeDeleted = obj.hasOwnProperty("property");

delete obj.property;

var afterDeleted = obj.hasOwnProperty("property");

assert.sameValue(beforeDeleted, true, 'beforeDeleted');
assert.sameValue(afterDeleted, false, 'afterDeleted');
