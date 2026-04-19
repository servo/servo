// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: The "toLocaleString" property of Temporal.PlainTime.prototype
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainTime.prototype.toLocaleString,
  "function",
  "`typeof PlainTime.prototype.toLocaleString` is `function`"
);

verifyProperty(Temporal.PlainTime.prototype, "toLocaleString", {
  writable: true,
  enumerable: false,
  configurable: true,
});
