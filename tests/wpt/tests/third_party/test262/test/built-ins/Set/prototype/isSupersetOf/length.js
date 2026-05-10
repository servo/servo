// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf length property
info: |
    Set.prototype.isSupersetOf ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.isSupersetOf, "function");

verifyProperty(Set.prototype.isSupersetOf, "length", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: 1,
});
