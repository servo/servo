// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: The "from" property of Temporal.PlainTime
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.PlainTime.from,
  "function",
  "`typeof PlainTime.from` is `function`"
);

verifyProperty(Temporal.PlainTime, "from", {
  writable: true,
  enumerable: false,
  configurable: true,
});
