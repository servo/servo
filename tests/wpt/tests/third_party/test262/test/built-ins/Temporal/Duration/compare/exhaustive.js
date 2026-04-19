// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Tests for compare() with each possible outcome
features: [Temporal]
---*/

const plainDate = new Temporal.PlainDate(2000, 1, 1);

assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(6),
    new Temporal.Duration(5),
    { relativeTo: plainDate }
  ),
  1,
  "years >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(3),
    new Temporal.Duration(4),
    { relativeTo: plainDate }
  ),
  -1,
  "years <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 6),
    new Temporal.Duration(2, 5),
    { relativeTo: plainDate }
  ),
  1,
  "months >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 3),
    new Temporal.Duration(4, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "months <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 6),
    new Temporal.Duration(2, 1, 5),
    { relativeTo: plainDate }
  ),
  1,
  "weeks >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 3),
    new Temporal.Duration(4, 7, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "weeks <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 6),
    new Temporal.Duration(2, 1, 3, 5),
    { relativeTo: plainDate }
  ),
  1,
  "days >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 3),
    new Temporal.Duration(4, 7, 2, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "days <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6),
    new Temporal.Duration(2, 1, 3, 12, 5),
    { relativeTo: plainDate }
  ),
  1,
  "hours >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 3),
    new Temporal.Duration(4, 7, 2, 40, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "hours <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 5),
    { relativeTo: plainDate }
  ),
  1,
  "minutes >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "minutes <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 5),
    { relativeTo: plainDate }
  ),
  1,
  "seconds >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "seconds <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 5),
    { relativeTo: plainDate }
  ),
  1,
  "milliseconds >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "milliseconds <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 5),
    { relativeTo: plainDate }
  ),
  1,
  "microseconds >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "microseconds <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 444, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 444, 5),
    { relativeTo: plainDate }
  ),
  1,
  "nanoseconds >, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 4),
    { relativeTo: plainDate }
  ),
  -1,
  "nanoseconds <, relativeTo PlainDate"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 111),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 111),
    { relativeTo: plainDate }
  ),
  0,
  "equal, relativeTo PlainDate"
);

const zonedDateTime = new Temporal.ZonedDateTime(0n, "UTC");

assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(6),
    new Temporal.Duration(5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "years >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(3),
    new Temporal.Duration(4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "years <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 6),
    new Temporal.Duration(2, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "months >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 3),
    new Temporal.Duration(4, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "months <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 6),
    new Temporal.Duration(2, 1, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "weeks >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 3),
    new Temporal.Duration(4, 7, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "weeks <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 6),
    new Temporal.Duration(2, 1, 3, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "days >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 3),
    new Temporal.Duration(4, 7, 2, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "days <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6),
    new Temporal.Duration(2, 1, 3, 12, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "hours >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 3),
    new Temporal.Duration(4, 7, 2, 40, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "hours <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "minutes >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "minutes <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "seconds >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "seconds <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "milliseconds >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "milliseconds <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "microseconds >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "microseconds <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 444, 6),
    new Temporal.Duration(2, 1, 3, 12, 6, 30, 15, 222, 444, 5),
    { relativeTo: zonedDateTime }
  ),
  1,
  "nanoseconds >, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 3),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 4),
    { relativeTo: zonedDateTime }
  ),
  -1,
  "nanoseconds <, relativeTo ZonedDateTime"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 111),
    new Temporal.Duration(4, 7, 2, 40, 12, 15, 45, 333, 777, 111),
    { relativeTo: zonedDateTime }
  ),
  0,
  "equal, relativeTo ZonedDateTime"
);

assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 6),
    new Temporal.Duration(0, 0, 0, 5)
  ),
  1,
  "days >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 3),
    new Temporal.Duration(0, 0, 0, 4)
  ),
  -1,
  "days <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 12, 6),
    new Temporal.Duration(0, 0, 0, 12, 5)
  ),
  1,
  "hours >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 3),
    new Temporal.Duration(0, 0, 0, 40, 4)
  ),
  -1,
  "hours <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 12, 6, 6),
    new Temporal.Duration(0, 0, 0, 12, 6, 5)
  ),
  1,
  "minutes >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 12, 3),
    new Temporal.Duration(0, 0, 0, 40, 12, 4)
  ),
  -1,
  "minutes <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 6),
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 5)
  ),
  1,
  "seconds >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 3),
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 4)
  ),
  -1,
  "seconds <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 15, 6),
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 15, 5)
  ),
  1,
  "milliseconds >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 3),
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 4)
  ),
  -1,
  "milliseconds <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 15, 222, 6),
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 15, 222, 5)
  ),
  1,
  "microseconds >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 333, 3),
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 333, 4)
  ),
  -1,
  "microseconds <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 15, 222, 444, 6),
    new Temporal.Duration(0, 0, 0, 12, 6, 30, 15, 222, 444, 5)
  ),
  1,
  "nanoseconds >, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 333, 777, 3),
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 333, 777, 4)
  ),
  -1,
  "nanoseconds <, relativeTo nothing"
);
assert.sameValue(
  Temporal.Duration.compare(
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 333, 777, 111),
    new Temporal.Duration(0, 0, 0, 40, 12, 15, 45, 333, 777, 111)
  ),
  0,
  "equal, relativeTo nothing"
);
