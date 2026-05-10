// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.from
description: >
  from works properly with eras (Japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";

const reiwa = Temporal.PlainDate.from({ era: "reiwa", eraYear: 2, month: 1, day: 1,  calendar });
const heisei = Temporal.PlainDate.from({ era: "heisei", eraYear: 2, month: 1, day: 1,  calendar });
const showa = Temporal.PlainDate.from({ era: "showa", eraYear: 2, month: 1, day: 1,  calendar });
const taisho = Temporal.PlainDate.from({ era: "taisho", eraYear: 2, month: 1, day: 1,  calendar });
const meiji = Temporal.PlainDate.from({ era: "meiji", eraYear: 6, month: 1, day: 1,  calendar });
const ce = Temporal.PlainDate.from({ era: "ce", eraYear: 1000, month: 1, day: 1,  calendar });
const bce = Temporal.PlainDate.from({ era: "bce", eraYear: 1, month: 1, day: 1,  calendar });

TemporalHelpers.assertPlainDate(reiwa, 2020, 1, "M01", 1,  `${reiwa}`, "reiwa", 2);

TemporalHelpers.assertPlainDate(heisei, 1990, 1, "M01", 1,  `${heisei}`, "heisei", 2);

TemporalHelpers.assertPlainDate(showa, 1927, 1, "M01", 1,  `${showa}`, "showa", 2);

TemporalHelpers.assertPlainDate(taisho, 1913, 1, "M01", 1,  `${taisho}`, "taisho", 2);

TemporalHelpers.assertPlainDate(meiji, 1873, 1, "M01", 1,  `${meiji}`, "meiji", 6);

TemporalHelpers.assertPlainDate(ce, 1000, 1, "M01", 1,  `${ce} (CE)`, "ce", 1000);

TemporalHelpers.assertPlainDate(bce, 0, 1, "M01", 1,  `${bce} (BCE)`, "bce", 1);
