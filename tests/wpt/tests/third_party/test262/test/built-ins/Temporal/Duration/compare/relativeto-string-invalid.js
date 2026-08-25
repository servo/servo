// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: RangeError thrown if relativeTo is a string with the wrong format, before early return
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 31);

[
  ['bad string', 'bad string'],
  ['15:30:45.123456', 'time string'],
  ['iso8601', 'calendar name string'],
  ['UTC', 'timezone'],
  ['P1YT1H', 'duration'],
  ['2019-11-01T00:00Z', 'date-time + Z without an IANA annotation'],
  ['2025-01-01T00:00:00+00:0000', 'date-time with wrong offset format']
].forEach(([relativeTo, description]) => {
  assert.throws(RangeError, () => Temporal.Duration.compare(duration, duration, { relativeTo }));
});
