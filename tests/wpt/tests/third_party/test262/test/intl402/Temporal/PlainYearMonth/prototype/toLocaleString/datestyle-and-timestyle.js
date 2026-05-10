// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: Using timeStyle, even if dateStyle is present, should throw
features: [Temporal]
---*/

const item = new Temporal.PlainYearMonth(2026, 1, "gregory", 1);

assert.throws(TypeError, function() {
  item.toLocaleString("en-u-ca-gregory", { dateStyle: "full", timeStyle: "full" });
});
