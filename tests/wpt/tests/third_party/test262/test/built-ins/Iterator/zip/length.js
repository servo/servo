// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zip
description: >
  Iterator.zip has a "length" property whose value is 1.
info: |
  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the length property of a built-in
  Function object has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.
features: [joint-iteration]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator.zip, "length", {
  value: 1,
  writable: false,
  enumerable: false,
  configurable: true,
});
