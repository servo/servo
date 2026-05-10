// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Options must contain at least the required correctly-spelled properties.
features: [Temporal]
---*/

// object must contain at least the required correctly-spelled properties
assert.throws(TypeError, () => Temporal.ZonedDateTime.from({
  years: 1976,
  months: 11,
  days: 18,
  timeZone: "+01:00"
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.from({
  years: 1976,
  month: 11,
  day: 18,
  timeZone: "+01:00"
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.from({
  year: 1976,
  months: 11,
  day: 18,
  timeZone: "+01:00"
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.from({
  year: 1976,
  month: 11,
  days: 18,
  timeZone: "+01:00"
}));

