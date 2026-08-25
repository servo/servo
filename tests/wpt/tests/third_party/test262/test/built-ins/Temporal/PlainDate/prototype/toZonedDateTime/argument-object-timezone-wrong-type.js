// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  Property "timeZone" can't be parsed as a time zone.
info: |
  Temporal.PlainDate.prototype.toZonedDateTime ( item )

  ...
  3. If item is an Object, then
    a. Let timeZoneLike be ? Get(item, "timeZone").
    b. If timeZoneLike is undefined, then
      ..
    c. Else,
      i. Let timeZone be ? ToTemporalTimeZoneIdentifier(timeZoneLike).
      ...

  ToTemporalTimeZoneIdentifier ( temporalTimeZoneLike )

  1. If temporalTimeZoneLike is an Object, then
    ...
  2. If temporalTimeZoneLike is not a String, throw a TypeError exception.
  ...
features: [Temporal]
---*/

var instance = new Temporal.PlainDate(1970, 1, 1);

for (var timeZone of [
  null,
  false,
  0,
  0n,
  Symbol(),
  {},
  [],
  function() {},
]) {
  var item = {timeZone};
  assert.throws(TypeError, () => instance.toZonedDateTime(item));
}
