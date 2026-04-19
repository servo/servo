// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-261
description: >
    Object.defineProperty - value of 'set' property in 'Attributes' is
    undefined (8.10.5 step 8.b)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  set: undefined
});

var desc = Object.getOwnPropertyDescriptor(obj, "property");

assert(obj.hasOwnProperty("property"));
verifyNotWritable(obj, "property");
