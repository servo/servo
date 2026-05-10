// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-162
description: >
    Object.defineProperty - 'writable' property in 'Attributes' is own
    accessor property without a get function  (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var obj = {};

var attr = {};
Object.defineProperty(attr, "writable", {
  set: function() {}
});

Object.defineProperty(obj, "property", attr);

verifyProperty(obj, "property", {
  writable: false,
});
