// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: The "from" property of Temporal.PlainDateTime
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDateTime.from,
  "function",
  "`typeof PlainDateTime.from` is `function`"
);

verifyProperty(Temporal.PlainDateTime, "from", {
  writable: true,
  enumerable: false,
  configurable: true,
});
