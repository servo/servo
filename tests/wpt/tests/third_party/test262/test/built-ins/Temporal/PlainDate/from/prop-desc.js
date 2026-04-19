// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: The "from" property of Temporal.PlainDate
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainDate.from,
  "function",
  "`typeof PlainDate.from` is `function`"
);

verifyProperty(Temporal.PlainDate, "from", {
  writable: true,
  enumerable: false,
  configurable: true,
});
