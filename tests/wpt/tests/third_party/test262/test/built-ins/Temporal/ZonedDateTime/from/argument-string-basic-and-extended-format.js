// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Mixture of basic and extended format.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

[
  "1976-11-18T152330.1-08:00[-08:00]",
  "19761118T15:23:30.1-08:00[-08:00]",
  "1976-11-18T15:23:30.1-0800[-08:00]",
  "1976-11-18T152330.1-0800[-08:00]",
  "19761118T15:23:30.1-0800[-08:00]",
  "19761118T152330.1-08:00[-08:00]",
  "19761118T152330.1-0800[-08:00]",
  "+001976-11-18T152330.1-08:00[-08:00]",
  "+0019761118T15:23:30.1-08:00[-08:00]",
  "+001976-11-18T15:23:30.1-0800[-08:00]",
  "+001976-11-18T152330.1-0800[-08:00]",
  "+0019761118T15:23:30.1-0800[-08:00]",
  "+0019761118T152330.1-08:00[-08:00]",
  "+0019761118T152330.1-0800[-08:00]"
].forEach(input => TemporalHelpers.assertZonedDateTimesEqual(
    Temporal.ZonedDateTime.from(input),
    new Temporal.ZonedDateTime(217207410100000000n, "-08:00")));
