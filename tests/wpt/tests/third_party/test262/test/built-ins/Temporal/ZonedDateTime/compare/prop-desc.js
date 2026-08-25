// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: The "compare" property of Temporal.ZonedDateTime
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.ZonedDateTime.compare,
  "function",
  "`typeof ZonedDateTime.compare` is `function`"
);

verifyProperty(Temporal.ZonedDateTime, "compare", {
  writable: true,
  enumerable: false,
  configurable: true,
});
