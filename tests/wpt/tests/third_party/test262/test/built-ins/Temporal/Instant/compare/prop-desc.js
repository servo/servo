// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: The "compare" property of Temporal.Instant
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Instant.compare,
  "function",
  "`typeof Instant.compare` is `function`"
);

verifyProperty(Temporal.Instant, "compare", {
  writable: true,
  enumerable: false,
  configurable: true,
});
