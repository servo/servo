// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-5-b-60
description: >
    Object.defineProperties - 'configurable' property of 'descObj' is
    not present (8.10.5 step 4)
includes: [propertyHelper.js]
---*/


var obj = {};

Object.defineProperties(obj, {
  prop: {}
});

verifyProperty(obj, "prop", {
  configurable: false,
});
