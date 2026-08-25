// Copyright (C) 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tolocalestring
description: >
  BigInt.prototype.toLocaleString.name is toLocaleString.
info: |
  BigInt.prototype.toLocaleString ( [ locales [ , options ] ] )

  17 ECMAScript Standard Built-in Objects:

    Every built-in function object, including constructors, that is not
    identified as an anonymous function has a name property whose value
    is a String. For functions that are specified as properties of objects,
    the name value is the property name string used to access the function.

    Unless otherwise specified, the name property of a built-in function
    object, if it exists, has the attributes { [[Writable]]: false,
    [[Enumerable]]: false, [[Configurable]]: true }.

includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigInt.prototype.toLocaleString, "name", {
  value: "toLocaleString",
  writable: false,
  enumerable: false,
  configurable: true,
});
