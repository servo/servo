// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.add
description: >
  Throws duration days are too large.
info: |
  Temporal.PlainDateTime.prototype.add ( temporalDurationLike [ , options ] )

  ...
  3. Return ? AddDurationToDateTime(add, dateTime, temporalDurationLike, options).

  AddDurationToDateTime ( operation, dateTime, temporalDurationLike, options )

  ...
  6. Let timeResult be AddTime(dateTime.[[ISODateTime]].[[Time]], internalDuration.[[Time]]).
  7. Let dateDuration be ? AdjustDateDurationRecord(internalDuration.[[Date]], timeResult.[[Days]]).
  ...

features: [Temporal]
---*/

const secondsPerDay = 24 * 60 * 60;
const maxSeconds = 2 ** 53 - 1;
const maxDays = Math.trunc(maxSeconds / secondsPerDay);
const maxHours = Math.trunc(((maxSeconds / secondsPerDay) % 1) * 24);

let d = new Temporal.Duration(0, 0, 0, maxDays, maxHours);
let pdt = new Temporal.PlainDateTime(1970, 1, 1, 24 - maxHours);

assert.throws(RangeError, () => pdt.add(d));
