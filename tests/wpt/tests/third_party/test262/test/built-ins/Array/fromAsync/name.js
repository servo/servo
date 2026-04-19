// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Value and property descriptor of Array.fromAsync.name
info: |
  Every built-in function object, including constructors, has a *"name"*
  property whose value is a String. Unless otherwise specified, this value is
  the name that is given to the function in this specification. [...]
  For functions that are specified as properties of objects, the name value is
  the property name string used to access the function.

  Unless otherwise specified, the *"name"* property of a built-in function
  object has the attributes { [[Writable]]: *false*, [[Enumerable]]: *false*,
  [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Array.fromAsync]
---*/

verifyProperty(Array.fromAsync, "name", {
  value: "fromAsync",
  writable: false,
  enumerable: false,
  configurable: true,
});
