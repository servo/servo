// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-293-1
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is own
    data property of 'O', test TypeError is not thrown when updating
    the [[Value]] attribute value of 'name' which is defined as
    non-writable and configurable (10.6 [[DefineOwnProperty]] step 3
    and 5b)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "0", {
    value: 10,
    writable: false
  });
  Object.defineProperty(arguments, "0", {
    value: 20
  });

  verifyProperty(arguments, "0", {
    value: 20,
    writable: false,
    enumerable: true,
    configurable: true,
  });
}(0, 1, 2));
