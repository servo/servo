// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.add
description: The "add" property of Temporal.Instant.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Instant.prototype.add,
  "function",
  "`typeof Instant.prototype.add` is `function`"
);

verifyProperty(Temporal.Instant.prototype, "add", {
  writable: true,
  enumerable: false,
  configurable: true,
});
