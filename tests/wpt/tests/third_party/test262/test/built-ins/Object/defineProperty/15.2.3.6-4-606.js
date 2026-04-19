// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-606
description: >
    ES5 Attributes - all attributes in Object.preventExtensions are
    correct
includes: [propertyHelper.js]
---*/

verifyProperty(Object, "preventExtensions", {
  writable: true,
  enumerable: false,
  configurable: true,
});
