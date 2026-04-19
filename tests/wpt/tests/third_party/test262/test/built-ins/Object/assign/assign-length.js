// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: The length property of the assign method should be 2
es6id:  19.1.2.1
info: |
    The length property of the assign method is 2.

    ES6 Section 17:

    Unless otherwise specified, the length property of a built-in Function
    object has the attributes { [[Writable]]: false, [[Enumerable]]: false,
    [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Object.assign, "length", {
  value: 2,
  writable: false,
  enumerable: false,
  configurable: true,
});
