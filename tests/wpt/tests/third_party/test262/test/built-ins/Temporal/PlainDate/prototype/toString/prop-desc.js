// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tostring
description: The "toString" property of Temporal.PlainDate.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.prototype.toString,
  "function",
  "`typeof PlainDate.prototype.toString` is `function`"
);

verifyProperty(Temporal.PlainDate.prototype, "toString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
