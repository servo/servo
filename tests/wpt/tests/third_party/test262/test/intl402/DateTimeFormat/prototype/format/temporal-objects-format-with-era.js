// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: >
  If era option and no other options are provided to DateTimeFormat constructor,
  objects should be formatted with default options
features: [Temporal]
locale: [en]
---*/

const formatter = new Intl.DateTimeFormat(["en"], { era: "narrow" });

assert(formatter.format(new Temporal.PlainDate(2025, 11, 4)).startsWith("11"), "formatting a PlainDate should work");
assert(formatter.format(new Temporal.PlainYearMonth(2025, 11, "gregory")).startsWith("11"), "formatting a PlainYearMonth should work");
assert(formatter.format(new Temporal.PlainMonthDay(11, 4, "gregory")).startsWith("11"), "formatting a PlainMonthDay should work");
assert(formatter.format(new Temporal.PlainTime(14, 46)).startsWith("2"), "formatting a PlainTime should work");
assert(formatter.format(new Temporal.PlainDateTime(2025, 11, 4, 14, 46)).startsWith("11"), "formatting a PlainDateTime should work");
assert.sameValue(formatter.format(new Temporal.Instant(0n)),
                 new Date(0).toLocaleString(["en"], { era: "narrow" }), "toLocaleString on an Instant with era option should return the same results as toLocaleString on the same Date with the same options");
