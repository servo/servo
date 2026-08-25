// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Argument may be cast
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
const dt2 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);

assert.sameValue(
  dt1.equals({
    year: 1976,
    month: 11,
    day: 18,
    hour: 15,
    minute: 23,
    second: 30,
    millisecond: 123,
    microsecond: 456,
    nanosecond: 789
  }),
  true,
  "casts argument (plain object, positive)"
);


assert.sameValue(
  dt2.equals({ year: 1976, month: 11, day: 18, hour: 15 }),
  false,
  "casts argument (plain object, negative)"
);

assert.sameValue(
  dt1.equals("1976-11-18T15:23:30.123456789"),
  true,
  "casts argument (string, positive)"
);

assert.sameValue(
  dt2.equals("1976-11-18T15:23:30.123456789"),
  false,
  "casts argument (string, negative)"
);
