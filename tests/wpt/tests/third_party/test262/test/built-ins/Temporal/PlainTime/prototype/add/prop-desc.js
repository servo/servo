// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
description: The "add" property of Temporal.PlainTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainTime.prototype.add,
  "function",
  "`typeof PlainTime.prototype.add` is `function`"
);

verifyProperty(Temporal.PlainTime.prototype, "add", {
  writable: true,
  enumerable: false,
  configurable: true,
});
