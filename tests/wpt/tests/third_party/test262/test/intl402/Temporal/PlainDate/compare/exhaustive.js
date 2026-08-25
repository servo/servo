// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: Tests for compare() with each possible outcome
features: [Temporal]
---*/

const cal1 = "iso8601";
const cal2 = "gregory";

assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(2000, 5, 31, cal1),
    new Temporal.PlainDate(1987, 5, 31, cal2)
  ),
  1,
  "year >"
);
assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(1981, 12, 15, cal1),
    new Temporal.PlainDate(2048, 12, 15, cal2)
  ),
  -1,
  "year <"
);
assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(2000, 5, 31, cal1),
    new Temporal.PlainDate(2000, 3, 31, cal2)
  ),
  1,
  "month >"
);
assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(1981, 4, 15, cal1),
    new Temporal.PlainDate(1981, 12, 15, cal2)
  ),
  -1,
  "month <"
);
assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(2000, 5, 31, cal1),
    new Temporal.PlainDate(2000, 5, 14, cal2)
  ),
  1,
  "day >"
);
assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(1981, 4, 15, cal1),
    new Temporal.PlainDate(1981, 4, 21, cal2)
  ),
  -1,
  "day <"
);
assert.sameValue(
  Temporal.PlainDate.compare(
    new Temporal.PlainDate(2000, 5, 31, cal1),
    new Temporal.PlainDate(2000, 5, 31, cal2)
  ),
  0,
  "="
);
