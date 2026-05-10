// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: The "with" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.with,
  "function",
  "`typeof PlainDate.prototype.with` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "with", {
  writable: true,
  enumerable: false,
  configurable: true,
});
