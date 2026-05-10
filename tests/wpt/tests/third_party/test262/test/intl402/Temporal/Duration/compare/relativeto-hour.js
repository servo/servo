// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo with hours.
features: [Temporal]
---*/

const oneDay = new Temporal.Duration(0, 0, 0, 1);
const hours24 = new Temporal.Duration(0, 0, 0, 0, 24);

assert.sameValue(
  Temporal.Duration.compare(oneDay, hours24, { relativeTo: Temporal.ZonedDateTime.from('2017-01-01T00:00[America/Montevideo]') }),
  0,
  'relativeTo does not affect days if ZonedDateTime, and duration encompasses no DST change');
assert.sameValue(
  Temporal.Duration.compare(oneDay, hours24, { relativeTo: Temporal.ZonedDateTime.from('2019-11-03T00:00[America/Vancouver]') }),
  1,
  'relativeTo does affect days if ZonedDateTime, and duration encompasses DST change');
assert.sameValue(
  Temporal.Duration.compare(oneDay, hours24, { relativeTo: '2019-11-03T00:00[America/Vancouver]' }),
  1,
  'casts relativeTo to ZonedDateTime from string');
assert.sameValue(
  Temporal.Duration.compare(oneDay, hours24, {
    relativeTo: { year: 2019, month: 11, day: 3, timeZone: 'America/Vancouver' }
  }),
  1,
  'casts relativeTo to ZonedDateTime from object');

