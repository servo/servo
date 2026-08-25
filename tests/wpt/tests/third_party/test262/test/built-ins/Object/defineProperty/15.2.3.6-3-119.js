// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-119
description: >
    Object.defineProperty - 'configurable' property in 'Attributes' is
    the JSON object  (8.10.5 step 4.b)
---*/

var obj = {};

var attr = {
  configurable: JSON
};

Object.defineProperty(obj, "property", attr);

var beforeDeleted = obj.hasOwnProperty("property");

delete obj.property;

var afterDeleted = obj.hasOwnProperty("property");

assert.sameValue(beforeDeleted, true, 'beforeDeleted');
assert.sameValue(afterDeleted, false, 'afterDeleted');
