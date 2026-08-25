// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: The "with" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.with,
  "function",
  "`typeof ZonedDateTime.prototype.with` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "with", {
  writable: true,
  enumerable: false,
  configurable: true,
});
