// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: >
  Japanese calendar dates before the Meiji era use "ce" or "bce" eras.
info: |
  The Japanese calendar in Temporal only supports the five modern eras (meiji,
  taisho, showa, heisei, reiwa) plus "ce" and "bce" for dates outside those
  ranges. Dates before Meiji 6 (1873) are represented using the "ce" era.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// A date before the Meiji era (which starts at 1873).
// 1800-06-15 in ISO calendar should be in the "ce" era in the Japanese calendar.
var preMeiji = Temporal.PlainDate.from({ calendar: "japanese", era: "ce", eraYear: 1800, month: 6, day: 15 });
TemporalHelpers.assertPlainDate(preMeiji, 1800, 6, "M06", 15, "pre-Meiji date", "ce", 1800);

// Also verify round-tripping through ISO
var isoDate = preMeiji.withCalendar("iso8601");
TemporalHelpers.assertPlainDate(isoDate, 1800, 6, "M06", 15, "round-tripped ISO date");

// A date in early Meiji, but before the cutoff at Meiji 6 (1873)
// 1870-03-01 should still be in "ce" era
var earlyMeiji = Temporal.PlainDate.from({ calendar: "japanese", era: "ce", eraYear: 1870, month: 3, day: 1 });
TemporalHelpers.assertPlainDate(earlyMeiji, 1870, 3, "M03", 1, "early pre-Meiji date", "ce", 1870);

// A BCE date in the Japanese calendar
var bceDate = Temporal.PlainDate.from({ calendar: "japanese", era: "bce", eraYear: 100, month: 1, day: 1 });
TemporalHelpers.assertPlainDate(bceDate, -99, 1, "M01", 1, "BCE date", "bce", 100);

// Converting an ISO date to Japanese calendar should also trigger the era
// assignment for pre-Meiji dates.
var isoPreMeiji = new Temporal.PlainDate(1800, 6, 15);
var japanesePreMeiji = isoPreMeiji.withCalendar("japanese");
TemporalHelpers.assertPlainDate(japanesePreMeiji, 1800, 6, "M06", 15, "ISO date converted to Japanese", "ce", 1800);

// Also test a BCE date via ISO-to-Japanese conversion
var isoBce = new Temporal.PlainDate(-99, 1, 1);
var japaneseBce = isoBce.withCalendar("japanese");
TemporalHelpers.assertPlainDate(japaneseBce, -99, 1, "M01", 1, "ISO BCE date converted to Japanese", "bce", 100);
