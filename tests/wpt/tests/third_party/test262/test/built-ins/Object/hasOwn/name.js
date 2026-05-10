// Copyright (C) 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.hasown
description: >
  Object.hasOwn.name is "hasOwn".
info: |
  Object.hasOwn ( _O_, _P_ )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
author: Jamie Kyle
features: [Object.hasOwn]
---*/

verifyProperty(Object.hasOwn, "name", {
  value: "hasOwn",
  writable: false,
  enumerable: false,
  configurable: true,
});
