// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-171
description: >
    Object.defineProperties - value of 'writable' property of
    'descObj' is NaN (8.10.5 step 6.b)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperties(obj, {
  property: {
    writable: NaN
  }
});

verifyProperty(obj, "property", {
  writable: false,
});
