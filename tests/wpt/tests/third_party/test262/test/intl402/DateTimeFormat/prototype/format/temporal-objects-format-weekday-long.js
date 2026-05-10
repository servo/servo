// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: datetime.toLocaleString()
features: [Temporal]
locale: [en-US]
---*/

function maybeGetWeekdayOnlyFormat() {
  const fmt = new Intl.DateTimeFormat("en-US", { weekday: "long", timeZone: "+00:00" });
  const resolvedOptions = fmt.resolvedOptions();
  if (
    ["era", "year", "month", "day", "hour", "minute", "second", "timeZoneName"].some(
      (prop) => prop in resolvedOptions
    )
  ) {
    // no weekday-only format available
    return null;
  }
  return fmt;
}

const fmt = maybeGetWeekdayOnlyFormat();
if (fmt) {
  const expectedWeekday = fmt.format(new Date("1976-11-18T15:23:30"));

  const datetime = Temporal.PlainDateTime.from("1976-11-18T15:23:30");
  assert.sameValue(fmt.format(datetime), expectedWeekday);

  const date = Temporal.PlainDate.from("1976-11-18T15:23:30");
  assert.sameValue(fmt.format(date), expectedWeekday);

  const instant = Temporal.Instant.from("1976-11-18T14:23:30Z");
  assert.sameValue(fmt.format(instant), expectedWeekday);
}
