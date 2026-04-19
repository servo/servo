// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.negated
description: The "negated" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.negated,
  "function",
  "`typeof Duration.prototype.negated` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "negated", {
  writable: true,
  enumerable: false,
  configurable: true,
});
