// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  relativeTo is a Temporal.PlainDateTime object.
info: |
  Temporal.Duration.prototype.total ( totalOf )

  ...
  7. Let relativeToRecord be ? GetTemporalRelativeToOption(totalOf).
  ...

  GetTemporalRelativeToOption ( options )

  1. Let value be ? Get(options, "relativeTo").
  ...
  5. If value is an Object, then
    ...
    c. If value has an [[InitializedTemporalDateTime]] internal slot, then
      i. Let plainDate be ! CreateTemporalDate(value.[[ISODateTime]].[[ISODate]], value.[[Calendar]]).
      ii. Return the Record { [[PlainRelativeTo]]: plainDate, [[ZonedRelativeTo]]: undefined }.
    ...
features: [Temporal]
---*/

var duration = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

var relativeToDate = new Temporal.PlainDate(1970, 1, 1);
var relativeToDateTime = new Temporal.PlainDateTime(1970, 1, 1);

for (var unit of [
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
]) {
  var expected = duration.total({unit, relativeTo: relativeToDate});
  var actual = duration.total({unit, relativeTo: relativeToDateTime});

  assert.sameValue(actual, expected, `unit = ${unit}`);
}
