// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: >
  Throws RangeError when adding duration to ZonedDateTime relativeTo fails.
info: |
  Temporal.Duration.compare ( one, two [ , options ] )

  12. If zonedRelativeTo is not undefined, and either TemporalUnitCategory(largestUnit1)
      or TemporalUnitCategory(largestUnit2) is date, then
    ...
    c. Let after1 be ? AddZonedDateTime(zonedRelativeTo.[[EpochNanoseconds]], timeZone,
       calendar, duration1, constrain).
    d. Let after2 be ? AddZonedDateTime(zonedRelativeTo.[[EpochNanoseconds]], timeZone,
       calendar, duration2, constrain).
    ...
features: [Temporal]
---*/

var blank = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
var oneDay = new Temporal.Duration(0, 0, 0, 1);

var relativeTo = new Temporal.ZonedDateTime(864n * 10n**19n, "UTC");

var options = {
  relativeTo
};

assert.throws(RangeError, () => Temporal.Duration.compare(oneDay, blank, options));
assert.throws(RangeError, () => Temporal.Duration.compare(blank, oneDay, options));
