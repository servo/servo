// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.from
description: >
  from works properly with eras (Japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";

const reiwa = Temporal.ZonedDateTime.from({ era: "reiwa", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });
const heisei = Temporal.ZonedDateTime.from({ era: "heisei", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });
const showa = Temporal.ZonedDateTime.from({ era: "showa", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });
const taisho = Temporal.ZonedDateTime.from({ era: "taisho", eraYear: 2, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });
const meiji = Temporal.ZonedDateTime.from({ era: "meiji", eraYear: 6, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });
const ce = Temporal.ZonedDateTime.from({ era: "ce", eraYear: 1000, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });
const bce = Temporal.ZonedDateTime.from({ era: "bce", eraYear: 1, month: 1, day: 1, hour: 12, minute: 34, timeZone: "UTC",  calendar });

TemporalHelpers.assertPlainDateTime(reiwa.toPlainDateTime(), 2020, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${reiwa}`, "reiwa", 2);

TemporalHelpers.assertPlainDateTime(heisei.toPlainDateTime(), 1990, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${heisei}`, "heisei", 2);

TemporalHelpers.assertPlainDateTime(showa.toPlainDateTime(), 1927, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${showa}`, "showa", 2);

TemporalHelpers.assertPlainDateTime(taisho.toPlainDateTime(), 1913, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${taisho}`, "taisho", 2);

TemporalHelpers.assertPlainDateTime(meiji.toPlainDateTime(), 1873, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${meiji}`, "meiji", 6);

TemporalHelpers.assertPlainDateTime(ce.toPlainDateTime(), 1000, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${ce} (CE)`, "ce", 1000);

TemporalHelpers.assertPlainDateTime(bce.toPlainDateTime(), 0, 1, "M01", 1, 12, 34, 0, 0, 0, 0,  `${bce} (BCE)`, "bce", 1);
