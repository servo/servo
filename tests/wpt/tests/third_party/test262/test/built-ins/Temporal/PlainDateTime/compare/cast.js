// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: Arguments may be casted (string, plain object)
features: [Temporal]
---*/

const dt1 = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
const dt2 = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);

// outcome smaller

assert.sameValue(
  Temporal.PlainDateTime.compare({ year: 1976, month: 11, day: 18, hour: 15 }, dt2),
  -1,
  "casts first argument (plain object)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare("1976-11-18T15:23:30.123456789", dt2),
  -1,
  "casts first argument (string)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare(dt1, { year: 2019, month: 10, day: 29, hour: 10 }),
  -1,
  "casts second argument (plain object)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare(dt1, "2019-10-29T10:46:38.271986102"),
  -1,
  "casts second argument (string)"
);

// outcome equal

assert.sameValue(
  Temporal.PlainDateTime.compare({
    year: 2019,
    month: 10,
    day: 29,
    hour: 10,
    minute: 46,
    second: 38,
    millisecond: 271,
    microsecond: 986,
    nanosecond: 102
  }, dt2),
  0,
  "casts first argument (plain object)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare("2019-10-29T10:46:38,271986102", dt2),
  0,
  "casts first argument (string)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare(dt2,
    {
      year: 2019,
      month: 10,
      day: 29,
      hour: 10,
      minute: 46,
      second: 38,
      millisecond: 271,
      microsecond: 986,
      nanosecond: 102
    }),
  0,
  "casts second argument (plain object)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare(dt2, "2019-10-29T10:46:38,271986102"),
  0,
  "casts second argument (string)"
);

// larger

assert.sameValue(
  Temporal.PlainDateTime.compare({ year: 2025, month: 11, day: 18, hour: 15 }, dt2),
  1,
  "casts first argument (plain object)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare("2025-11-18T15:23:30.123456789", dt2),
  1,
  "casts first argument (string)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare(dt1, { year: 1800, month: 10, day: 29, hour: 10 }),
  1,
  "casts second argument (plain object)"
);

assert.sameValue(
  Temporal.PlainDateTime.compare(dt2, "2019-10-29T10:46:38,271986101"),
  1,
  "casts second argument (string)"
);
