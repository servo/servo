// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: The "since" property of Temporal.Instant.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Instant.prototype.since,
  "function",
  "`typeof Instant.prototype.since` is `function`"
);

verifyProperty(Temporal.Instant.prototype, "since", {
  writable: true,
  enumerable: false,
  configurable: true,
});
