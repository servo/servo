// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference length property
info: |
    Set.prototype.symmetricDifference ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.symmetricDifference, "function");

verifyProperty(Set.prototype.symmetricDifference, "length", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: 1,
});
