// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-292
description: >
    Object.defineProperty - 'O' is an Arguments object, 'name' is own
    data property of 'O', and 'desc' is data descriptor, test updating
    multiple attribute values of 'name' (10.6 [[DefineOwnProperty]]
    step 3)
includes: [propertyHelper.js]
---*/

(function() {
  Object.defineProperty(arguments, "0", {
    value: 20,
    writable: false,
    enumerable: false,
    configurable: false
  });

  verifyProperty(arguments, "0", {
    value: 20,
    writable: false,
    enumerable: false,
    configurable: false,
  });
}(0, 1, 2));
