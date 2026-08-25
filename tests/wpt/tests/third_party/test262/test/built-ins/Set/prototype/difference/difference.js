// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.difference
description: Set.prototype.difference properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.difference,
  "function",
  "`typeof Set.prototype.difference` is `'function'`"
);

verifyProperty(Set.prototype, "difference", {
  enumerable: false,
  writable: true,
  configurable: true,
});
