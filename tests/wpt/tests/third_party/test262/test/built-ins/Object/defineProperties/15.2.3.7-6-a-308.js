// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.7-6-a-308
description: >
    Object.defineProperties - 'O' is an Arguments object, 'P' is
    generic own data property of 'O', test TypeError is thrown when
    updating the [[Enumerable]] attribute value of 'P' which is not
    configurable (10.6 [[DefineOwnProperty]] step 4)
includes: [propertyHelper.js]
---*/

var arg = (function() {
  return arguments;
}(1, 2, 3));

Object.defineProperty(arg, "genericProperty", {
  enumerable: true,
  configurable: false
});

try {
  Object.defineProperties(arg, {
    "genericProperty": {
      enumerable: false
    }
  });

  throw new Test262Error("Expected an exception.");
} catch (e) {
  if (!(e instanceof TypeError)) {
    throw new Test262Error("Expected TypeError, got " + e);
  }
}

verifyProperty(arg, "genericProperty", {
  value: undefined,
  writable: false,
  enumerable: true,
  configurable: false,
});
