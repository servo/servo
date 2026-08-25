// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: Basic tests that dateStyle option affects output
locale: [en-u-ca-gregory, en-u-ca-islamic-tbla]
features: [Temporal, Intl.DateTimeFormat-datetimestyle]
---*/

const dateGregorian = Temporal.PlainMonthDay.from({ monthCode: "M03", day: 26, calendar: "gregory" });

assert(
  dateGregorian.toLocaleString("en-u-ca-gregory", { dateStyle: "long" }).includes("March"),
  "dateStyle: long writes month of March out in full"
);
assert(
  !dateGregorian.toLocaleString("en-u-ca-gregory", { dateStyle: "short" }).includes("March"),
  "dateStyle: short does not write month of March out in full"
);

const dateIslamic = Temporal.PlainMonthDay.from({ monthCode: "M09", day: 16, calendar: "islamic-tbla" });

assert(
  dateIslamic.toLocaleString("en-u-ca-islamic-tbla", { dateStyle: "long" }).includes("Ramadan"),
  "dateStyle: long writes month of Ramadan out in full"
);
assert(
  !dateIslamic.toLocaleString("en-u-ca-islamic-tbla", { dateStyle: "short" }).includes("Ramadan"),
  "dateStyle: short does not write month of Ramadan out in full"
);

const dateWithReferenceYear = new Temporal.PlainMonthDay(5, 31, "gregory", 2222);
assert(
  !dateWithReferenceYear.toLocaleString("en", { dateStyle: "full" }).includes("2222"),
  "dateStyle: full should not format reference year at all"
);
