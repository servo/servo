// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: >
  Throws if overflow is "reject" and any value is outside the valid bounds.
info: |
  Temporal.PlainTime.prototype.with ( temporalTimeLike [ , options ] )

  ...
  19. Let result be ? RegulateTime(hour, minute, second, millisecond, microsecond, nanosecond, overflow).
  ...

  RegulateTime ( hour, minute, second, millisecond, microsecond, nanosecond, overflow )

  ...
  2. Else,
    a. Assert: overflow is reject.
    b. If IsValidTime(hour, minute, second, millisecond, microsecond, nanosecond) is false, throw a RangeError exception.
  ...

features: [Temporal]
---*/

var instance = new Temporal.PlainTime();

var temporalTimeLikes = [
  {hour: -1},
  {hour: 24},
  {minute: -1},
  {minute: 60},
  {second: -1},
  {second: 60},
  {millisecond: -1},
  {millisecond: 1000},
  {microsecond: -1},
  {microsecond: 1000},
  {nanosecond: -1},
  {nanosecond: 1000},
];

var options = {overflow: "reject"};

for (var temporalTimeLike of temporalTimeLikes) {
  assert.throws(
    RangeError,
    () => instance.with(temporalTimeLike, options),
    `temporalTimeLike = ${JSON.stringify(temporalTimeLike)}`
  );
}
