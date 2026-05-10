// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: RegExp.prototype[Symbol.matchAll] `name` property
info: |
  17 ECMAScript Standard Built-in Objects:

    [...]

    Every built-in function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    [...]

    Unless otherwise specified, the name property of a built-in function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Symbol.matchAll]
---*/

verifyProperty(RegExp.prototype[Symbol.matchAll], "name", {
  value: "[Symbol.matchAll]",
  writable: false,
  enumerable: false,
  configurable: true
});
