// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.symmetricdifference
description: Set.prototype.symmetricDifference properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.symmetricDifference,
  "function",
  "`typeof Set.prototype.symmetricDifference` is `'function'`"
);

verifyProperty(Set.prototype, "symmetricDifference", {
  enumerable: false,
  writable: true,
  configurable: true,
});
