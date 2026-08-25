// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es6id: 20.2.2.13
description: Length property of Math.cosh is 1
info: |
  Math.cosh ( x )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Math.cosh, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true
});
