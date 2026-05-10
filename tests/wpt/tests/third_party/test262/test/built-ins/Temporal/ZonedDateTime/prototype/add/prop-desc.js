// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: The "add" property of Temporal.ZonedDateTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.prototype.add,
  "function",
  "`typeof ZonedDateTime.prototype.add` is `function`"
);

verifyProperty(Temporal.ZonedDateTime.prototype, "add", {
  writable: true,
  enumerable: false,
  configurable: true,
});
