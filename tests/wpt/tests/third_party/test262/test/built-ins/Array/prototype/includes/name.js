// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.includes
description: >
  Array.prototype.includes.name is "includes".
info: |
  Array.prototype.includes (searchElement [ , fromIndex ] )

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Array.prototype.includes]
---*/

verifyProperty(Array.prototype.includes, "name", {
  value: "includes",
  writable: false,
  enumerable: false,
  configurable: true
});
