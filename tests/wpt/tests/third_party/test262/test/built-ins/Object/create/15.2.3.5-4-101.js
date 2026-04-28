// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-101
description: >
    Object.create - 'configurable' property of one property in
    'Properties' is own data property (8.10.5 step 4.a)
includes: [propertyHelper.js]
---*/

var newObj = Object.create({}, {
  prop: {
    configurable: false
  }
});

verifyProperty(newObj, "prop", {
  configurable: false,
});
