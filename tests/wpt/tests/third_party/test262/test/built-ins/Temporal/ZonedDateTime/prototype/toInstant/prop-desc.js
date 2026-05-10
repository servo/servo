// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toinstant
description: The "toInstant" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.toInstant,
  "function",
  "`typeof ZonedDateTime.prototype.toInstant` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "toInstant", {
  writable: true,
  enumerable: false,
  configurable: true,
});
