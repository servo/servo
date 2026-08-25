// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.valueof
description: The "valueOf" property of Temporal.PlainTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainTime.prototype.valueOf,
  "function",
  "`typeof PlainTime.prototype.valueOf` is `function`"
);

verifyProperty(Temporal.PlainTime.prototype, "valueOf", {
  writable: true,
  enumerable: false,
  configurable: true,
});
