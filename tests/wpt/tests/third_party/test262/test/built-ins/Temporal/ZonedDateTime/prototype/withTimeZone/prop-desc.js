// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: The "withTimeZone" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.withTimeZone,
  "function",
  "`typeof ZonedDateTime.prototype.withTimeZone` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "withTimeZone", {
  writable: true,
  enumerable: false,
  configurable: true,
});
