// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Property descriptor of Iterator.zipKeyed
info: |
  17 ECMAScript Standard Built-in Objects

  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
features: [joint-iteration]
includes: [propertyHelper.js]
---*/

verifyProperty(Iterator, "zipKeyed", {
  value: Iterator.zipKeyed,
  writable: true,
  enumerable: false,
  configurable: true,
});
