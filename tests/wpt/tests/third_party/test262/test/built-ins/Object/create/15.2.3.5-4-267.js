// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-267
description: >
    Object.create - 'set' property of one property in 'Properties' is
    not present (8.10.5 step 8)
includes: [propertyHelper.js]
---*/

var newObj = Object.create({}, {
  prop: {
    get: function() {
      return "data";
    }
  }
});

assert(newObj.hasOwnProperty("prop"));
verifyNotWritable(newObj, "prop");
