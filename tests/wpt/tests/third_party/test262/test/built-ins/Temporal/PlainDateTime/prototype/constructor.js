// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.constructor
description: Test for Temporal.PlainDateTime.prototype.constructor.
info: The initial value of Temporal.PlainDateTime.prototype.constructor is %Temporal.PlainDateTime%.
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.PlainDateTime.prototype, "constructor", {
  value: Temporal.PlainDateTime,
  writable: true,
  enumerable: false,
  configurable: true,
});
