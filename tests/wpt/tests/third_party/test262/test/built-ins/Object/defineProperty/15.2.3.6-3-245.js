// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-245
description: >
    Object.defineProperty - 'set' property in 'Attributes' is own
    accessor property without a get function (8.10.5 step 8.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var attributes = {};
Object.defineProperty(attributes, "set", {
  set: function() {}
});

Object.defineProperty(obj, "property", attributes);

verifyNotWritable(obj, "property");

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert(obj.hasOwnProperty("property"));
assert.sameValue(typeof obj.property, "undefined");
assert.sameValue(typeof desc.set, "undefined");
