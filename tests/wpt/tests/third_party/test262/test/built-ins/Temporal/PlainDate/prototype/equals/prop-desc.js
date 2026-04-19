// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.equals
description: The "equals" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.equals,
  "function",
  "`typeof PlainDate.prototype.equals` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "equals", {
  writable: true,
  enumerable: false,
  configurable: true,
});
