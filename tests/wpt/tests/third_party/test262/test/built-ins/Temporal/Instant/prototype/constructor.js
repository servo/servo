// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.constructor
description: Test for Temporal.Instant.prototype.constructor.
info: The initial value of Temporal.Instant.prototype.constructor is %Temporal.Instant%.
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.Instant.prototype, "constructor", {
  value: Temporal.Instant,
  writable: true,
  enumerable: false,
  configurable: true,
});
