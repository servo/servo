// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-179
description: >
    Object.defineProperty - 'writable' property in 'Attributes' is
    null  (8.10.5 step 6.b)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  writable: null
});

verifyProperty(obj, "property", {
  writable: false,
});
