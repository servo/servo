// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-125
description: >
    Object.create - 'configurable' property of one property in
    'Properties' is undefined (8.10.5 step 4.b)
includes: [propertyHelper.js]
---*/

var newObj = Object.create({}, {
  prop: {
    configurable: undefined
  }
});

verifyProperty(newObj, "prop", {
  configurable: false,
});
