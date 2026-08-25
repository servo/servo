// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: User code calls happen in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const actual = [];
const expected = [
  "get item.timeZone",
  "get item.plainTime",
  // ToTemporalTime
  "get item.plainTime.hour",
  "get item.plainTime.hour.valueOf",
  "call item.plainTime.hour.valueOf",
  "get item.plainTime.microsecond",
  "get item.plainTime.microsecond.valueOf",
  "call item.plainTime.microsecond.valueOf",
  "get item.plainTime.millisecond",
  "get item.plainTime.millisecond.valueOf",
  "call item.plainTime.millisecond.valueOf",
  "get item.plainTime.minute",
  "get item.plainTime.minute.valueOf",
  "call item.plainTime.minute.valueOf",
  "get item.plainTime.nanosecond",
  "get item.plainTime.nanosecond.valueOf",
  "call item.plainTime.nanosecond.valueOf",
  "get item.plainTime.second",
  "get item.plainTime.second.valueOf",
  "call item.plainTime.second.valueOf",
];

const instance = new Temporal.PlainDate(2000, 1, 1, "iso8601");

const plainTime = TemporalHelpers.propertyBagObserver(actual, {
  hour: 2,
  minute: 30,
  second: 0,
  millisecond: 0,
  microsecond: 0,
  nanosecond: 0,
}, "item.plainTime");
const item = TemporalHelpers.propertyBagObserver(actual, {
  plainTime,
  timeZone: "UTC"
}, "item", ["timeZone"]);

instance.toZonedDateTime(item);
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear
