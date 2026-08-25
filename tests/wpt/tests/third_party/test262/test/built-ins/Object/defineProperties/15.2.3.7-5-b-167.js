// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-167
description: >
    Object.defineProperties - value of 'writable' property of
    'descObj' is false (8.10.5 step 6.b)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    writable: false
  }
});

verifyProperty(obj, "property", {
  writable: false,
});
