// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-33
description: >
    Object.defineProperties - 'P' doesn't exist in 'O', test [[Get]]
    of 'P' is set as undefined value if absent in accessor descriptor
    'desc' (8.12.9 step 4.b)
includes: [propertyHelper.js]
---*/

var obj = {};
var setFun = function(value) {
  obj.setVerifyHelpProp = value;
};

Object.defineProperties(obj, {
  prop: {
    set: setFun,
    enumerable: true,
    configurable: true
  }
});
verifyWritable(obj, "prop", "setVerifyHelpProp");

verifyProperty(obj, "prop", {
  enumerable: true,
  configurable: true,
});
