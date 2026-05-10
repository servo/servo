// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.issupersetof
description: Set.prototype.isSupersetOf properties
includes: [propertyHelper.js]
features: [set-methods]
---*/

assert.sameValue(
  typeof Set.prototype.isSupersetOf,
  "function",
  "`typeof Set.prototype.isSupersetOf` is `'function'`"
);

verifyProperty(Set.prototype, "isSupersetOf", {
  enumerable: false,
  writable: true,
  configurable: true,
});
