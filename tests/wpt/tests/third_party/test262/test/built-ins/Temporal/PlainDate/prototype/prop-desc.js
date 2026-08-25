// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype
description: The "prototype" property of Temporal.PlainDate
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.PlainDate.prototype, "object");
assert.notSameValue(Temporal.PlainDate.prototype, null);

verifyProperty(Temporal.PlainDate, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
