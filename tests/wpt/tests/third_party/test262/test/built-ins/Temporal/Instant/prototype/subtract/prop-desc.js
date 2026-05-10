// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.subtract
description: The "subtract" property of Temporal.Instant.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Instant.prototype.subtract,
  "function",
  "`typeof Instant.prototype.subtract` is `function`"
);

verifyProperty(Temporal.Instant.prototype, "subtract", {
  writable: true,
  enumerable: false,
  configurable: true,
});
