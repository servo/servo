// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: The "toZonedDateTime" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.toZonedDateTime,
  "function",
  "`typeof PlainDate.prototype.toZonedDateTime` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "toZonedDateTime", {
  writable: true,
  enumerable: false,
  configurable: true,
});
