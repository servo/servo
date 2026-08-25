// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Set.prototype.intersection properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.intersection,
  "function",
  "`typeof Set.prototype.intersection` is `'function'`"
);

verifyProperty(Set.prototype, "intersection", {
  enumerable: false,
  writable: true,
  configurable: true,
});
