// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: The "toZonedDateTimeISO" property of Temporal.Instant.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Instant.prototype.toZonedDateTimeISO,
  "function",
  "`typeof Instant.prototype.toZonedDateTimeISO` is `function`"
);

verifyProperty(Temporal.Instant.prototype, "toZonedDateTimeISO", {
  writable: true,
  enumerable: false,
  configurable: true,
});
