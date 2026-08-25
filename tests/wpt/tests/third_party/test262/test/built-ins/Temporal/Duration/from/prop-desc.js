// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: The "from" property of Temporal.Duration
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Duration.from,
  "function",
  "`typeof Duration.from` is `function`"
);

verifyProperty(Temporal.Duration, "from", {
  writable: true,
  enumerable: false,
  configurable: true,
});
