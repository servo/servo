// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Collator.prototype.compare
description: >
  get Intl.Collator.prototype.compare.name is "get compare".
info: |
  10.3.3 get Intl.Collator.prototype.compare

  17 ECMAScript Standard Built-in Objects:
    Every built-in Function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String.

    Unless otherwise specified, the name property of a built-in Function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
---*/

var desc = Object.getOwnPropertyDescriptor(Intl.Collator.prototype, "compare");

verifyProperty(desc.get, "name", {
  value: "get compare",
  writable: false,
  enumerable: false,
  configurable: true,
});
