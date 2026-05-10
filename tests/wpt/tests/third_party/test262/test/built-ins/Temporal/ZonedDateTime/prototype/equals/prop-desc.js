// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: The "equals" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.equals,
  "function",
  "`typeof ZonedDateTime.prototype.equals` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "equals", {
  writable: true,
  enumerable: false,
  configurable: true,
});
