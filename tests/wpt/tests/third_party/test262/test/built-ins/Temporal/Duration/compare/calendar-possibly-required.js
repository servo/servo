// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo argument needed if days = 0 but years/months/weeks non-zero
features: [Temporal]
---*/
const duration1a = new Temporal.Duration(1);
const duration1b = new Temporal.Duration(1, 0, 0, 0, 0, 0, 0, 0, 0, 1);
const duration2a = new Temporal.Duration(0, 12);
const duration2b = new Temporal.Duration(0, 12, 0, 0, 0, 0, 0, 0, 0, 1);
const duration3a = new Temporal.Duration(0, 0, 5);
const duration3b = new Temporal.Duration(0, 0, 5, 0, 0, 0, 0, 0, 0, 1);
const duration4a = new Temporal.Duration(0, 0, 0, 42);
const duration4b = new Temporal.Duration(0, 0, 0, 42, 0, 0, 0, 0, 0, 1);
const relativeTo = new Temporal.PlainDate(2021, 12, 15);
assert.throws(
  RangeError,
  () => { Temporal.Duration.compare(duration1a, duration1b); },
  "cannot compare Duration values without relativeTo if year is non-zero"
);
assert.sameValue(-1,
  Temporal.Duration.compare(duration1a, duration1b, { relativeTo }),
  "compare succeeds for year-only Duration provided relativeTo is supplied");
assert.throws(
  RangeError,
  () => { Temporal.Duration.compare(duration2a, duration2b); },
  "cannot compare Duration values without relativeTo if month is non-zero"
);
assert.sameValue(-1,
  Temporal.Duration.compare(duration2a, duration2b, { relativeTo }),
  "compare succeeds for year-and-month Duration provided relativeTo is supplied");
assert.throws(
  RangeError,
  () => { Temporal.Duration.compare(duration3a, duration3b); },
  "cannot compare Duration values without relativeTo if week is non-zero"
);
assert.sameValue(-1,
  Temporal.Duration.compare(duration3a, duration3b, { relativeTo }),
  "compare succeeds for year-and-month-and-week Duration provided relativeTo is supplied"
);

assert.sameValue(-1,
  Temporal.Duration.compare(duration4a, duration4b),
  "compare succeeds for zero year-month-week non-zero day Duration even without relativeTo");

// Double-check that the comparison also works with a relative-to argument
assert.sameValue(-1,
  Temporal.Duration.compare(duration4a, duration4b, { relativeTo }),
  "compare succeeds for zero year-month-week non-zero day Duration with relativeTo"
);
