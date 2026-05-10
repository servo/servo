// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: Addition around end of month in the gregory calendar
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };

const months6 = new Temporal.Duration(0, 6);
const months6n = new Temporal.Duration(0, -6);
const durations = [
  months6,
  months6n,
];

const date20001201 = Temporal.PlainDateTime.from({ year: 2000, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar }, options);
const dates = [
  date20001201,
];

for (var duration of durations) {
  for (var start of dates) {
    const end = start.add(duration);

    // startYesterday = start - (1 day)
    const startYesterday = start.add({ days: -1 });
    // endYesterday = startYesterday + duration
    const endYesterday = startYesterday.add(duration);
    // When adding months, the result day should be the same
    // unless there are fewer days in the destination month than the source day
    assert.sameValue(endYesterday.day, Math.min(startYesterday.day, endYesterday.daysInMonth), "adding months should result in same day");

    // endYesterdayNextDay = endYesterday + (1 day)
    var endYesterdayNextDay = endYesterday.add({ days: 1 });
    // Move forward to next first-day-of-month
    while (endYesterdayNextDay.day !== 1) {
      endYesterdayNextDay = endYesterdayNextDay.add({ days: 1 });
    }

    TemporalHelpers.assertPlainDateTime(endYesterdayNextDay, end.year, end.month, end.monthCode, end.day, 12, 34, 0, 0, 0, 0, `endYesterdayNextDay`, end.era, end.eraYear);

    // endReverse should equal end
    const endReverse = endYesterdayNextDay.add({ days: -1 });
    const startReverse = endReverse.add(duration.negated());
    // subtracting months give the same day unless there are fewer days in the destination month
    assert.sameValue(startReverse.day, Math.min(endReverse.day, startReverse.daysInMonth));

    // Move forward to next first-day-of-month
    var startReverseNextDay = startReverse.add({ days: 1 });
    while(startReverseNextDay.day !== 1) {
      startReverseNextDay = startReverseNextDay.add({ days: 1 });
    }

    TemporalHelpers.assertPlainDateTime(startReverseNextDay, start.year, start.month, start.monthCode, start.day, 12, 34, 0, 0, 0, 0, `startReverseNextDay`, start.era, start.eraYear);
  }
}
