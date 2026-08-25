// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.abs
description: The "abs" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.abs,
  "function",
  "`typeof Duration.prototype.abs` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "abs", {
  writable: true,
  enumerable: false,
  configurable: true,
});
