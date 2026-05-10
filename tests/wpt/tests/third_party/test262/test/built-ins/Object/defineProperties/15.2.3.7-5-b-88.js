// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-88
description: >
    Object.defineProperties - value of 'configurable' property of
    'descObj' is false (8.10.5 step 4.b)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    configurable: false
  }
});

verifyProperty(obj, "property", {
  configurable: false,
});
