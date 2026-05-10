// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype-@@tostringtag
description: The @@toStringTag property of Temporal.PlainYearMonth
includes: [propertyHelper.js]
features: [Temporal]
---*/

verifyProperty(Temporal.PlainYearMonth.prototype, Symbol.toStringTag, {
  value: "Temporal.PlainYearMonth",
  writable: false,
  enumerable: false,
  configurable: true,
});
