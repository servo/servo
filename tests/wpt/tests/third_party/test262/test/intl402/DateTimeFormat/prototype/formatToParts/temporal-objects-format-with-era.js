// Copyright (C) 2025 Igalia, S.L. All rights reserved.
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

assert(checkEra(formatter.formatToParts(new Temporal.PlainDate(2025, 11, 4))), "formatting a PlainDate should work");
assert(checkEra(formatter.formatToParts(new Temporal.PlainYearMonth(2025, 11, "gregory"))), "formatting a PlainYearMonth should work");
assert(!checkEra(formatter.formatToParts(new Temporal.PlainMonthDay(11, 4, "gregory"))), "formatting a PlainMonthDay should work");
assert(!checkEra(formatter.formatToParts(new Temporal.PlainTime(14, 46))), "formatting a PlainTime should work");
assert(checkEra(formatter.formatToParts(new Temporal.PlainDateTime(2025, 11, 4, 14, 46))), "formatting a PlainDateTime should work");
assert(checkEra(formatter.formatToParts(new Temporal.Instant(0n))), "formatting an Instant should work");
