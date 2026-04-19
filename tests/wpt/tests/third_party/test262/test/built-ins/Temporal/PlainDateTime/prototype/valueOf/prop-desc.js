// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.valueof
description: The "valueOf" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.valueOf,
  "function",
  "`typeof PlainDateTime.prototype.valueOf` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "valueOf", {
  writable: true,
  enumerable: false,
  configurable: true,
});
