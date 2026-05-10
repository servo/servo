// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  The "name" property of Iterator.concat
info: |
  17 ECMAScript Standard Built-in Objects

  Every built-in Function object, including constructors, that is not
  identified as an anonymous function has a name property whose value is a
  String. Unless otherwise specified, this value is the name that is given to
  the function in this specification.

  ...

  Unless otherwise specified, the name property of a built-in Function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
features: [iterator-sequencing]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator.concat, "name", {
  value: "concat",
  writable: false,
  enumerable: false,
  configurable: true,
});
