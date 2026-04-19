// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tolocalestring
description: dateStyle or timeStyle present but undefined
features: [BigInt, Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(957270896_987_650_000n, "UTC");
const defaultFormatter = new Intl.DateTimeFormat("en", {
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "numeric",
  minute: "numeric",
  second: "numeric",
  timeZoneName: "short",
  timeZone: "UTC",
});
const expected = defaultFormatter.format(datetime.toInstant());

const actualDate = datetime.toLocaleString("en", { dateStyle: undefined });
assert.sameValue(actualDate, expected, "dateStyle undefined is the same as being absent");

const actualTime = datetime.toLocaleString("en", { timeStyle: undefined });
assert.sameValue(actualTime, expected, "timeStyle undefined is the same as being absent");
