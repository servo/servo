// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.from
description: >
  from works properly with eras (Japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";

const reiwa = Temporal.PlainYearMonth.from({ era: "reiwa", eraYear: 2, month: 1,  calendar });
const heisei = Temporal.PlainYearMonth.from({ era: "heisei", eraYear: 2, month: 1,  calendar });
const showa = Temporal.PlainYearMonth.from({ era: "showa", eraYear: 2, month: 1,  calendar });
const taisho = Temporal.PlainYearMonth.from({ era: "taisho", eraYear: 2, month: 1,  calendar });
const meiji = Temporal.PlainYearMonth.from({ era: "meiji", eraYear: 6, month: 1,  calendar });
const ce = Temporal.PlainYearMonth.from({ era: "ce", eraYear: 1000, month: 1,  calendar });
const bce = Temporal.PlainYearMonth.from({ era: "bce", eraYear: 1, month: 1,  calendar });

TemporalHelpers.assertPlainYearMonth(reiwa, 2020, 1, "M01",  `${reiwa}`, "reiwa", 2);

TemporalHelpers.assertPlainYearMonth(heisei, 1990, 1, "M01",  `${heisei}`, "heisei", 2);

TemporalHelpers.assertPlainYearMonth(showa, 1927, 1, "M01",  `${showa}`, "showa", 2);

TemporalHelpers.assertPlainYearMonth(taisho, 1913, 1, "M01",  `${taisho}`, "taisho", 2);

TemporalHelpers.assertPlainYearMonth(meiji, 1873, 1, "M01",  `${meiji}`, "meiji", 6);

TemporalHelpers.assertPlainYearMonth(ce, 1000, 1, "M01",  `${ce} (CE)`, "ce", 1000);

TemporalHelpers.assertPlainYearMonth(bce, 0, 1, "M01",  `${bce} (BCE)`, "bce", 1);
