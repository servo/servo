// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.5-4-109
description: >
    Object.create - 'configurable' property of one property in
    'Properties' is own accessor property without a get function
    (8.10.5 step 4.a)
includes: [propertyHelper.js]
---*/

var descObj = {};
Object.defineProperty(descObj, "configurable", {
  set: function() {}
});

var newObj = Object.create({}, {
  prop: descObj
});

verifyProperty(newObj, "prop", {
  configurable: false,
});
