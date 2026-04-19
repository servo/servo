// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: The "toPlainDateTime" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.toPlainDateTime,
  "function",
  "`typeof PlainDate.prototype.toPlainDateTime` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "toPlainDateTime", {
  writable: true,
  enumerable: false,
  configurable: true,
});
