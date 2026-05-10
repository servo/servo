// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-138
description: >
    Object.defineProperties - 'writable' property of 'descObj' is
    present (8.10.5 step 6)
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
