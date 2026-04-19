// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: The "round" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.round,
  "function",
  "`typeof Duration.prototype.round` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "round", {
  writable: true,
  enumerable: false,
  configurable: true,
});
