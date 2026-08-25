// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: The "toString" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.toString,
  "function",
  "`typeof Duration.prototype.toString` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "toString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
