// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-159
description: >
    Object.defineProperties - 'descObj' is the JSON object which
    implements its own [[Get]] method to get 'writable' property
    (8.10.5 step 6.a)
includes: [propertyHelper.js]
---*/

var obj = {};

JSON.writable = false;

Object.defineProperties(obj, {
  property: JSON
});

verifyProperty(obj, "property", {
  writable: false,
});
