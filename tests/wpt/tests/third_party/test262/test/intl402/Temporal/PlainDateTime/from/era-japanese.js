// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.from
description: >
  from works properly with eras (Japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";

const reiwa = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34,  calendar });
const heisei = Temporal.PlainDateTime.from({ era: "heisei", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34,  calendar });
const showa = Temporal.PlainDateTime.from({ era: "showa", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34,  calendar });
const taisho = Temporal.PlainDateTime.from({ era: "taisho", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34,  calendar });
const meiji = Temporal.PlainDateTime.from({ era: "meiji", eraYear: 6, month: 1, day: 1, hour: 12, minute: 34,  calendar });
const ce = Temporal.PlainDateTime.from({ era: "ce", eraYear: 1000, month: 1, day: 1, hour: 12, minute: 34,  calendar });
const bce = Temporal.PlainDateTime.from({ era: "bce", eraYear: 1, month: 1, day: 1, hour: 12, minute: 34,  calendar });

TemporalHelpers.assertPlainDateTime(reiwa, 2020, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${reiwa}`, "reiwa", 2);

TemporalHelpers.assertPlainDateTime(heisei, 1990, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${heisei}`, "heisei", 2);

TemporalHelpers.assertPlainDateTime(showa, 1927, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${showa}`, "showa", 2);

TemporalHelpers.assertPlainDateTime(taisho, 1913, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${taisho}`, "taisho", 2);

TemporalHelpers.assertPlainDateTime(meiji, 1873, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${meiji}`, "meiji", 6);

TemporalHelpers.assertPlainDateTime(ce, 1000, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${ce} (CE)`, "ce", 1000);

TemporalHelpers.assertPlainDateTime(bce, 0, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${bce} (BCE)`, "bce", 1);
