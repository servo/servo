// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Non-positive era years are remapped in ethiopic calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethiopic";
const options = { overflow: "reject" };

const am0 = Temporal.PlainDate.from({ era: "am", eraYear: 0, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  am0,
  0, 1, "M01", 1, "AM 0 resolves to AA 5500",
  "aa", 5500);

const am1n = Temporal.PlainDate.from({ era: "am", eraYear: -1, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  am1n,
  -1, 1, "M01", 1, "AM -1 resolves to AA 5499",
  "aa", 5499);

const aa0 = Temporal.PlainDate.from({ era: "aa", eraYear: 0, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  aa0,
  -5500, 1, "M01", 1, "AA 0 is not remapped",
  "aa", 0);

const aa1n = Temporal.PlainDate.from({ era: "aa", eraYear: -1, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  aa1n,
  -5501, 1, "M01", 1, "AA -1 is not remapped",
  "aa", -1);
