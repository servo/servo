// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: The "equals" property of Temporal.PlainTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainTime.prototype.equals,
  "function",
  "`typeof PlainTime.prototype.equals` is `function`"
);

verifyProperty(Temporal.PlainTime.prototype, "equals", {
  writable: true,
  enumerable: false,
  configurable: true,
});
