// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.constructor
description: Test for Temporal.PlainDate.prototype.constructor.
info: The initial value of Temporal.PlainDate.prototype.constructor is %Temporal.PlainDate%.
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.PlainDate.prototype, "constructor", {
  value: Temporal.PlainDate,
  writable: true,
  enumerable: false,
  configurable: true,
});
