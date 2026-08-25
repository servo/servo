// Copyright (C) 2023 Anthony Frehner and Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.intersection
description: Set.prototype.intersection name property
info: |
    Set.prototype.intersection ( other )
includes: [propertyHelper.js]
features: [set-methods]
---*/
assert.sameValue(typeof Set.prototype.intersection, "function");

verifyProperty(Set.prototype.intersection, "name", {
  enumerable: false,
  writable: false,
  configurable: true,
  value: "intersection",
});
