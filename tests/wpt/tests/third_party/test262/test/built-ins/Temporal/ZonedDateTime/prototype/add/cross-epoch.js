// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Cross-epoch add/subtract
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// "1969-12-25T12:23:45.678901234+00:00[UTC]"
const zdt = new Temporal.ZonedDateTime(-560174321098766n, "UTC")

// cross epoch in ms
  var one = zdt.subtract({
    hours: 240,
    nanoseconds: 800
  });
  var two = zdt.add({
    hours: 240,
    nanoseconds: 800
  });
  var three = two.subtract({
    hours: 480,
    nanoseconds: 1600
  });
  var four = one.add({
    hours: 480,
    nanoseconds: 1600
  });

TemporalHelpers.assertZonedDateTimesEqual(one,
                                          // "1969-12-15T12:23:45.678900434+00:00[UTC]"
                                          new Temporal.ZonedDateTime(-1424174321099566n, "UTC"));
TemporalHelpers.assertZonedDateTimesEqual(two,
                                          // "1970-01-04T12:23:45.678902034+00:00[UTC]")
                                          new Temporal.ZonedDateTime(303825678902034n, "UTC"));
TemporalHelpers.assertZonedDateTimesEqual(three, one);
TemporalHelpers.assertZonedDateTimesEqual(four, two);
