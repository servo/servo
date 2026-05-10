// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype
description: The "prototype" property of Temporal.PlainTime
includes: [propertyHelper.js]
features: [Temporal]
---*/

assert.sameValue(typeof Temporal.PlainTime.prototype, "object");
assert.notSameValue(Temporal.PlainTime.prototype, null);

verifyProperty(Temporal.PlainTime, "prototype", {
  writable: false,
  enumerable: false,
  configurable: false,
});
