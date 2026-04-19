// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-3-236
description: >
    Object.defineProperty - 'set' property in 'Attributes' is not
    present (8.10.5 step 8)
includes: [propertyHelper.js]
---*/

var obj = {};

Object.defineProperty(obj, "property", {
  get: function() {
    return 11;
  }
});

assert(obj.hasOwnProperty("property"));
verifyNotWritable(obj, "property");
