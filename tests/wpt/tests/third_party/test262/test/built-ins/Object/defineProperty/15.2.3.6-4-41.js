// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-41
description: >
    Object.defineProperty - 'O' is the JSON object that uses Object's
    [[GetOwnProperty]] method to access the 'name' property (8.12.9
    step 1)
includes: [propertyHelper.js]
---*/


Object.defineProperty(JSON, "foo", {
  value: 12,
  configurable: true
});

verifyProperty(JSON, "foo", {
  value: 12,
  writable: false,
  enumerable: false,
  configurable: true,
});

delete JSON.foo;
