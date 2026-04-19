// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: The "subtract" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.subtract,
  "function",
  "`typeof Duration.prototype.subtract` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "subtract", {
  writable: true,
  enumerable: false,
  configurable: true,
});
