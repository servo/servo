// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tospliced
description: >
  Array.prototype.toSpliced.name is "toSpliced".
info: |
  Array.prototype.toSpliced ( start, deleteCount, ...items )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [change-array-by-copy]
---*/

verifyProperty(Array.prototype.toSpliced, "name", {
  value: "toSpliced",
  writable: false,
  enumerable: false,
  configurable: true,
});
