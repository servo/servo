// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: The "until" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.until,
  "function",
  "`typeof PlainDate.prototype.until` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "until", {
  writable: true,
  enumerable: false,
  configurable: true,
});
