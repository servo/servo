// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: The "since" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.since,
  "function",
  "`typeof ZonedDateTime.prototype.since` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "since", {
  writable: true,
  enumerable: false,
  configurable: true,
});
