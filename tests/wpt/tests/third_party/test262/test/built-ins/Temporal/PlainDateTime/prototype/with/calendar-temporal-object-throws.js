// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if a Temporal object with a calendar is supplied
esid: sec-temporal.plaindatetime.prototype.with
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

const values = [
  Temporal.PlainDate.from("2022-04-12"),
  Temporal.PlainDateTime.from("2022-04-12T15:19:45"),
  Temporal.PlainMonthDay.from("04-12"),
  Temporal.PlainTime.from("15:19:45"),
  Temporal.PlainYearMonth.from("2022-04"),
  Temporal.ZonedDateTime.from("2022-04-12T15:19:45[UTC]"),
  Temporal.Now.plainDateTimeISO(),
  Temporal.Now.plainDateISO(),
  Temporal.Now.plainTimeISO(),
];

for (const value of values) {
  Object.defineProperty(value, "calendar", {
    get() { throw new Test262Error("should not get calendar property") }
  });
  Object.defineProperty(value, "timeZone", {
    get() { throw new Test262Error("should not get timeZone property") }
  });
  assert.throws(
    TypeError,
    () => datetime.with(value),
    "throws with temporal object"
  );
}
