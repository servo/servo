// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.gettimezonetransition
description: Properties on an object passed to getTimeZoneTransition() are accessed in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const expected = [
  "get fields.direction",
  "get fields.direction.toString",
  "call fields.direction.toString"
];
const actual = [];

const fields = TemporalHelpers.propertyBagObserver(actual, {
  year: 1.7,
  month: 1.7,
  monthCode: "M01",
  day: 1.7,
  hour: 1,
  microsecond: 2,
  millisecond: 3,
  minute: 4,
  nanosecond: 5,
  second: 6,
  overflow: "constrain",
  largestUnit: "years",
  smallestUnit: "months",
  direction: "next",
  calendar: "iso8601",
}, "fields", ["calendar"]);

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

zdt.getTimeZoneTransition(fields);
assert.compareArray(actual, expected, "order of operations");
