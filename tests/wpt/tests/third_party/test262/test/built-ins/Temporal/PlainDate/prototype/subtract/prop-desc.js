// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: The "subtract" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.subtract,
  "function",
  "`typeof PlainDate.prototype.subtract` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "subtract", {
  writable: true,
  enumerable: false,
  configurable: true,
});
