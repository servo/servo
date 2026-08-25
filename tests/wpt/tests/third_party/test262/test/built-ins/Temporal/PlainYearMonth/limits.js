// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: Limits for the PlainYearMonth constructor.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

assert.throws(RangeError, () => new Temporal.PlainYearMonth(-271821, 3), "min");
assert.throws(RangeError, () => new Temporal.PlainYearMonth(275760, 10), "max");
TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(-271821, 4),
  -271821, 4, "M04", "min");
TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(-271821, 4, "iso8601", 18),
  -271821, 4, "M04", "min with referenceISODay",
  /* era = */ undefined, /* eraYear = */ undefined, /* referenceISODay = */ 18);
TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(275760, 9),
  275760, 9, "M09", "max");
TemporalHelpers.assertPlainYearMonth(new Temporal.PlainYearMonth(275760, 9, "iso8601", 14),
  275760, 9, "M09", "max with referenceISODay",
  /* era = */ undefined, /* eraYear = */ undefined, /* referenceISODay = */ 14);
