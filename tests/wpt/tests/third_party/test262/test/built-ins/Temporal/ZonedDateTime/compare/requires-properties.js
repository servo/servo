// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Object must contain at least the required properties.
features: [Temporal]
---*/

/*
const zdt1 = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");
const zdt2 = Temporal.ZonedDateTime.from("2019-10-29T10:46:38.271986102+01:00[+01:00]");
*/
const zdt1 = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");
const zdt2 = new Temporal.ZonedDateTime(1572342398271986102n, "+01:00");

// object must contain at least the required properties
assert.sameValue(Temporal.ZonedDateTime.compare({
  year: 1976,
  month: 11,
  day: 18,
  timeZone: "+01:00"
}, zdt2), -1);
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare({
  month: 11,
  day: 18,
  timeZone: "+01:00"
}, zdt2));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare({
  year: 1976,
  day: 18,
  timeZone: "+01:00"
}, zdt2));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare({
  year: 1976,
  month: 11,
  timeZone: "+01:00"
}, zdt2));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare({
  year: 1976,
  month: 11,
  day: 18
}, zdt2));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare({
  years: 1976,
  months: 11,
  days: 19,
  hours: 15,
  timeZone: "+01:00"
}, zdt2));
assert.sameValue(Temporal.ZonedDateTime.compare(zdt1, {
  year: 2019,
  month: 10,
  day: 29,
  timeZone: "+01:00"
}), -1);
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(zdt1, {
  month: 10,
  day: 29,
  timeZone: "+01:00"
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(zdt1, {
  year: 2019,
  day: 29,
  timeZone: "+01:00"
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(zdt1, {
  year: 2019,
  month: 10,
  timeZone: "+01:00"
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(zdt1, {
  year: 2019,
  month: 10,
  day: 29
}));
assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(zdt1, {
  years: 2019,
  months: 10,
  days: 29,
  hours: 10,
  timeZone: "+01:00"
}));
