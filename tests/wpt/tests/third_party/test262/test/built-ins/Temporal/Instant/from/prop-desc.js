// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: The "from" property of Temporal.Instant
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Instant.from,
  "function",
  "`typeof Instant.from` is `function`"
);

verifyProperty(Temporal.Instant, "from", {
  writable: true,
  enumerable: false,
  configurable: true,
});
