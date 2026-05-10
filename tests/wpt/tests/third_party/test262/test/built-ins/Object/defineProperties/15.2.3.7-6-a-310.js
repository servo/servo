// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-310
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is
    generic own accessor property of 'O', test TypeError is thrown
    when updating the [[Get]] attribute value of 'P' which is not
    configurable (10.6 [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

var arg = (function() {
  return arguments;
}(1, 2, 3));

function getFun() {
  return "genericPropertyString";
}

function setFun(value) {
  arg.verifySetFun = value;
}
Object.defineProperty(arg, "genericProperty", {
  get: getFun,
  set: setFun,
  configurable: false
});

try {
  Object.defineProperties(arg, {
    "genericProperty": {
      get: function() {
        return "overideGenericPropertyString";
      }
    }
  });

  throw new Test262Error("Expected an exception.");
} catch (e) {
  verifyEqualTo(arg, "genericProperty", getFun());

  verifyWritable(arg, "genericProperty", "verifySetFun");

  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arg, "genericProperty", {
  enumerable: false,
  configurable: false,
});
