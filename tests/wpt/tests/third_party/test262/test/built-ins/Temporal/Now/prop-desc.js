// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now
description: The "Now" property of Temporal
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(
  typeof Temporal.Now,
  "object",
  "`typeof Now` is `object`"
);

verifyProperty(Temporal, "Now", {
  writable: true,
  enumerable: false,
  configurable: true,
});
