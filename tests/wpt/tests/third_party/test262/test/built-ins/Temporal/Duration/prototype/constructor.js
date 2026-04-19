// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.constructor
description: Test for Temporal.Duration.prototype.constructor.
info: The initial value of Temporal.Duration.prototype.constructor is %Temporal.Duration%.
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.Duration.prototype, "constructor", {
  value: Temporal.Duration,
  writable: true,
  enumerable: false,
  configurable: true,
});
