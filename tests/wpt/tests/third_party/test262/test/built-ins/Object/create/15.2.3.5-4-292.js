// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-292
description: >
    Object.create - 'set' property of one property in 'Properties' is
    undefined (8.10.5 step 8.b)
includes: [propertyHelper.js]
---*/

var newObj = Object.create({}, {
  prop: {
    set: undefined
  }
});

assert(newObj.hasOwnProperty("prop"));
verifyNotWritable(newObj, "prop");
