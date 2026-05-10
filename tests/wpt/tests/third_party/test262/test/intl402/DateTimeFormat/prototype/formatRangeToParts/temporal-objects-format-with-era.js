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

function checkEra(parts) {
    for (let part of parts) {
        if (part.type === 'era' && part.value.startsWith('A'))
            return true;
    }
    return false;
}

assert(checkEra(formatter.formatRangeToParts(new Temporal.PlainDate(2025, 11, 4 , "gregory"), new Temporal.PlainDate(2025, 11, 5, "gregory"))), "formatting a PlainDate should work");
assert(checkEra(formatter.formatRangeToParts(new Temporal.PlainYearMonth(2025, 11, "gregory"), new Temporal.PlainYearMonth(2025, 12, "gregory"))), "formatting a PlainYearMonth should work");
assert(!checkEra(formatter.formatRangeToParts(new Temporal.PlainMonthDay(11, 4, "gregory"), new Temporal.PlainMonthDay(11, 14, "gregory"))), "formatting a PlainMonthDay should work");
assert(!checkEra(formatter.formatRangeToParts(new Temporal.PlainTime(14, 46), new Temporal.PlainTime(17, 46))), "formatting a PlainTime should work");
assert(checkEra(formatter.formatRangeToParts(new Temporal.PlainDateTime(2025, 11, 4, 14, 16, 0, 0, 0, 0, "gregory"), new Temporal.PlainDateTime(2025, 11, 15, 14, 47, 0, 0, 0, 0, "gregory"))), "formatting a PlainDateTime should work");
assert(checkEra(formatter.formatRangeToParts(new Temporal.Instant(0n), new Temporal.Instant(1000000000n))), "formatting an Instant should work");
