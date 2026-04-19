// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: since() casts argument from object or string.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// var zdt = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");
const zdt = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");
// "2019-10-29T10:46:38.271986102+01:00[+01:00]"
const zdt2 = new Temporal.ZonedDateTime(1572342398271986102n, "+01:00");

TemporalHelpers.assertDuration(zdt.since({
  year: 2019,
  month: 10,
  day: 29,
  hour: 10,
  timeZone: "+01:00"
}), 0, 0, 0, 0, -376434, -36, -29, -876, -543, -211);
TemporalHelpers.assertDuration(zdt.since(zdt2),
  0, 0, 0, 0, -376435, -23, -8, -148, -529, -313);




