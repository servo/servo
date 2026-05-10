// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaindatetime
description: The "toPlainDateTime" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.toPlainDateTime,
  "function",
  "`typeof ZonedDateTime.prototype.toPlainDateTime` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "toPlainDateTime", {
  writable: true,
  enumerable: false,
  configurable: true,
});
