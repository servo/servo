// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-typedarray-constructors
description: BigUint64Array.name property descriptor
info: |
  22.2.5 Properties of the TypedArray Constructors

  [...]

  Each TypedArray constructor has a name property whose value is the
  String value of the constructor name specified for it in Table 52.

  17 ECMAScript Standard Built-in Objects

  Every built-in function object, including constructors, that is not
  identified as an anonymous function has a name property whose value
  is a String. Unless otherwise specified, this value is the name that
  is given to the function in this specification. For functions that
  are specified as properties of objects, the name value is the
  property name string used to access the function. [...]

  Unless otherwise specified, the name property of a built-in function
  object, if it exists, has the attributes { [[Writable]]: false,
  [[Enumerable]]: false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [BigInt]
---*/

verifyProperty(BigUint64Array, "name", {
  value: "BigUint64Array",
  writable: false,
  enumerable: false,
  configurable: true
});
