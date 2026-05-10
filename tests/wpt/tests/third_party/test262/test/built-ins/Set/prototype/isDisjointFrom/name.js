// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.isdisjointfrom
description: Set.prototype.isDisjointFrom name property
info: |
    Set.prototype.isDisjointFrom ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.isDisjointFrom, "function");

verifyProperty(Set.prototype.isDisjointFrom, "name", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: "isDisjointFrom",
});
