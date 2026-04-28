// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype
description: The "prototype" property of Temporal.Instant
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.Instant.prototype, "object");
assert.notSameValue(Temporal.Instant.prototype, null);

verifyProperty(Temporal.Instant, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
