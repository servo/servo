// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo string accepts an inexact UTC offset rounded to hours and minutes
features: [Temporal]
---*/

let duration1 = new Temporal.Duration(0, 0, 0, 31);
let duration2 = new Temporal.Duration(0, 1);

let result;
let relativeTo;

const action = (relativeTo) => Temporal.Duration.compare(duration1, duration2, { relativeTo });

relativeTo = "1970-01-01T00:00:00-00:45[Africa/Monrovia]";
result = action(relativeTo);
assert.sameValue(result, 0, "rounded HH:MM is accepted in string offset");

relativeTo = "1970-01-01T00:00:00-00:44:30[Africa/Monrovia]";
result = action(relativeTo);
assert.sameValue(result, 0, "unrounded HH:MM:SS is accepted in string offset");

relativeTo = "1970-01-01T00:00:00-00:44:40[Africa/Monrovia]";
assert.throws(RangeError, () => action(relativeTo), "wrong :SS not accepted in string offset");

relativeTo = "1970-01-01T00:00:00-00:45:00[Africa/Monrovia]";
assert.throws(RangeError, () => action(relativeTo), "rounded HH:MM:SS not accepted in string offset");

relativeTo = { year: 1970, month: 1, day: 1, offset: "-00:45", timeZone: "Africa/Monrovia" };
assert.throws(RangeError, () => action(relativeTo), "rounded HH:MM not accepted as offset in property bag");

// Pacific/Niue edge case

duration1 = new Temporal.Duration(0, 0, 0, 0, /* hours = */ 24);
duration2 = new Temporal.Duration(0, 0, 0, /* days = */ 1);

assert.sameValue(
  action("1952-10-15T23:59:59-11:19:40[Pacific/Niue]"), -1,
  "-11:19:40 is accepted as -11:19:40 in Pacific/Niue edge case"
);
assert.sameValue(
  action("1952-10-15T23:59:59-11:20[Pacific/Niue]"), -1,
  "-11:20 matches the first candidate -11:19:40 in the Pacific/Niue edge case"
);
assert.sameValue(
  action("1952-10-15T23:59:59-11:20:00[Pacific/Niue]"), 0,
  "-11:20:00 is accepted as -11:20:00 in the Pacific/Niue edge case"
);
assert.throws(
  RangeError, () => action("1952-10-15T23:59:59-11:19:50[Pacific/Niue]"),
  "wrong :SS not accepted in the Pacific/Niue edge case"
);
