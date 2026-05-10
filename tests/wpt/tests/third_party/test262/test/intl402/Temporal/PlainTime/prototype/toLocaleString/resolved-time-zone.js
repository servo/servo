// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: A time zone in resolvedOptions with a large offset still produces the correct string
locale: [en]
features: [Temporal]
---*/

// Using 24-hour clock to avoid format differences between Node 19 (which puts
// "\u{202f}" before AM/PM) and previous versions that use regular spaces.
const options = {
  timeZone: "Pacific/Apia",
  hour: "numeric",
  minute: "numeric",
  second: "numeric",
  hourCycle: "h23"
};

const time1 = new Temporal.PlainTime(0, 30, 45, 123, 456, 789);
const result1 = time1.toLocaleString("en", options);
assert.sameValue(result1, "00:30:45");

const time2 = new Temporal.PlainTime(23, 30, 45, 123, 456, 789);
const result2 = time2.toLocaleString("en", options);
assert.sameValue(result2, "23:30:45");
