// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf name property
info: |
    Set.prototype.isSupersetOf ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.isSupersetOf, "function");

verifyProperty(Set.prototype.isSupersetOf, "name", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: "isSupersetOf",
});
