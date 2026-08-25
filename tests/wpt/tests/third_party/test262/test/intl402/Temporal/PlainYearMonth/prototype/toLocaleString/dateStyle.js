// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: Basic tests that dateStyle option affects output
locale: [en-u-ca-gregory, en-u-ca-islamic-tbla]
features: [Temporal, Intl.DateTimeFormat-datetimestyle]
---*/

const dateGregorian = Temporal.PlainYearMonth.from({ year: 2024, monthCode: "M03", calendar: "gregory" });

assert(
  dateGregorian.toLocaleString("en-u-ca-gregory", { dateStyle: "long" }).includes("March"),
  "dateStyle: long writes month of March out in full"
);
assert(
  !dateGregorian.toLocaleString("en-u-ca-gregory", { dateStyle: "short" }).includes("March"),
  "dateStyle: short does not write month of March out in full"
);

const dateIslamic = Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M09", calendar: "islamic-tbla" });

assert(
  dateIslamic.toLocaleString("en-u-ca-islamic-tbla", { dateStyle: "long" }).includes("Ramadan"),
  "dateStyle: long writes month of Ramadan out in full"
);
assert(
  !dateIslamic.toLocaleString("en-u-ca-islamic-tbla", { dateStyle: "short" }).includes("Ramadan"),
  "dateStyle: short does not write month of Ramadan out in full"
);

const dateWithReferenceDay = new Temporal.PlainYearMonth(2024, 5, "gregory", 31);
assert(
  !dateWithReferenceDay.toLocaleString("en", { dateStyle: "full" }).includes("31"),
  "dateStyle: full should not format reference day at all"
);
