// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Calendar era code is canonicalized
features: [Temporal, Intl.Era-monthcode]
---*/

const date1 = Temporal.ZonedDateTime.from({
  calendar: "gregory",
  era: "ad",
  eraYear: 2024,
  year: 2024,
  month: 1,
  day: 1,
  timeZone: "UTC",
});
assert.sameValue(date1.era, "ce", "'ad' is accepted as alias for 'ce'");

const date2 = Temporal.ZonedDateTime.from({
  calendar: "gregory",
  era: "bc",
  eraYear: 44,
  year: -43,
  month: 3,
  day: 15,
  timeZone: "Europe/Rome",
});
assert.sameValue(date2.era, "bce", "'bc' is accepted as alias for 'bce'");
