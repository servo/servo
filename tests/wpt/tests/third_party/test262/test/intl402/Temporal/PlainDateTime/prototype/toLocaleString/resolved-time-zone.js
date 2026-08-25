// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tolocalestring
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
features: [Temporal]
---*/

// Using 24-hour clock to avoid format differences between Node 19 (which puts
// "\u{202f}" before AM/PM) and previous versions that use regular spaces.
const options = {
  timeZone: "Pacific/Apia",
  year: "numeric",
  month: "numeric",
  day: "numeric",
  hour: "numeric",
  minute: "numeric",
  second: "numeric",
  hourCycle: "h23"
};

const datetime1 = new Temporal.PlainDateTime(2021, 8, 4, 0, 30, 45, 123, 456, 789);
const result1 = datetime1.toLocaleString("en", options);
assert.sameValue(result1, "8/4/2021, 00:30:45");

const datetime2 = new Temporal.PlainDateTime(2021, 8, 4, 23, 30, 45, 123, 456, 789);
const result2 = datetime2.toLocaleString("en", options);
assert.sameValue(result2, "8/4/2021, 23:30:45");
