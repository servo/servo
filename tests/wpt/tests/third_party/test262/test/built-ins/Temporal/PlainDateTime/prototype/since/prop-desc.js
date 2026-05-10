// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: The "since" property of Temporal.PlainDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.prototype.since,
  "function",
  "`typeof PlainDateTime.prototype.since` is `function`"
);

verifyProperty(Temporal.PlainDateTime.prototype, "since", {
  writable: true,
  enumerable: false,
  configurable: true,
});
