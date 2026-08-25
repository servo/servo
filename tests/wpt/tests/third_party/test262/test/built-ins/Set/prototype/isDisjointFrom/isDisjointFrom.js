// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.isDisjointFrom,
  "function",
  "`typeof Set.prototype.isDisjointFrom` is `'function'`"
);

verifyProperty(Set.prototype, "isDisjointFrom", {
  enumerable: false,
  writable: true,
  configurable: true,
});
