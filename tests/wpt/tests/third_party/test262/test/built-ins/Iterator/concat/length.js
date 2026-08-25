// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Iterator.concat has a "length" property whose value is 0.
info: |
  ECMAScript Standard Built-in Objects

  Unless otherwise specified, the length property of a built-in
  Function object has the attributes { [[Writable]]: false, [[Enumerable]]:
  false, [[Configurable]]: true }.
features: [iterator-sequencing]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator.concat, "length", {
  value: 0,
  writable: false,
  enumerable: false,
  configurable: true,
});
