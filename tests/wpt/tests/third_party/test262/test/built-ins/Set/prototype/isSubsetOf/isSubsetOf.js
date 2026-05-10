// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issubsetof
description: Set.prototype.isSubsetOf properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.isSubsetOf,
  "function",
  "`typeof Set.prototype.isSubsetOf` is `'function'`"
);

verifyProperty(Set.prototype, "isSubsetOf", {
  enumerable: false,
  writable: true,
  configurable: true,
});
