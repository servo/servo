// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaintime
description: The "toPlainTime" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.toPlainTime,
  "function",
  "`typeof ZonedDateTime.prototype.toPlainTime` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "toPlainTime", {
  writable: true,
  enumerable: false,
  configurable: true,
});
