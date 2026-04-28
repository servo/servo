// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: The "since" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.since,
  "function",
  "`typeof PlainDate.prototype.since` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "since", {
  writable: true,
  enumerable: false,
  configurable: true,
});
