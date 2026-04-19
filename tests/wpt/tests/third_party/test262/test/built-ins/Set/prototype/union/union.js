// Copyright (C) 2023 Anthony Frehner. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.union
description: Set.prototype.union properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.union,
  "function",
  "`typeof Set.prototype.union` is `'function'`"
);

verifyProperty(Set.prototype, "union", {
  enumerable: false,
  writable: true,
  configurable: true,
});
