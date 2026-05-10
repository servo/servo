// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.add
description: The "add" property of Temporal.Duration.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.prototype.add,
  "function",
  "`typeof Duration.prototype.add` is `function`"
);

verifyProperty(Temporal.Duration.prototype, "add", {
  writable: true,
  enumerable: false,
  configurable: true,
});
