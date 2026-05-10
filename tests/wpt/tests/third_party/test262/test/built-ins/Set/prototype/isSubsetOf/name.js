// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf name property
info: |
    Set.prototype.isSubsetOf ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.isSubsetOf, "function");

verifyProperty(Set.prototype.isSubsetOf, "name", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: "isSubsetOf",
});
