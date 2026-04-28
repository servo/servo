// Copyright (C) 2023 Anthony Frehner. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.union
description: Set.prototype.union length property
info: |
    Set.prototype.union ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.union, "function");

verifyProperty(Set.prototype.union, "length", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: 1,
});
