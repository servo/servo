// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.1.2.6
description: >
  Object.getOwnPropertyDescriptor.name is "getOwnPropertyDescriptor".
info: |
  Object.getOwnPropertyDescriptor ( O, P )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

verifyProperty(Object.getOwnPropertyDescriptor, "name", {
  value: "getOwnPropertyDescriptor",
  writable: false,
  enumerable: false,
  configurable: true,
});
