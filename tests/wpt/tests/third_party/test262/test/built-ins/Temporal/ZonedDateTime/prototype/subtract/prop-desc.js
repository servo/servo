// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: The "subtract" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.subtract,
  "function",
  "`typeof ZonedDateTime.prototype.subtract` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "subtract", {
  writable: true,
  enumerable: false,
  configurable: true,
});
