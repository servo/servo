// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: ZonedDateTime string accepts an inexact UTC offset rounded to hours and minutes
features: [Temporal]
includes: [compareArray.js]
---*/

let reference = new Temporal.ZonedDateTime(2670_000_000_000n, "Africa/Monrovia");

function action(string) {
  const result1 = Temporal.ZonedDateTime.compare(string, reference);
  const result2 = Temporal.ZonedDateTime.compare(reference, string);
  return [result1, result2];
};

assert.compareArray(
  action("1970-01-01T00:00-00:45[Africa/Monrovia]"), [0, 0],
  "rounded HH:MM is accepted in string offset"
);
assert.compareArray(
  action("1970-01-01T00:00:00-00:44:30[Africa/Monrovia]"), [0, 0],
  "unrounded HH:MM:SS is accepted in string offset"
);
assert.throws(
  RangeError, () => action("1970-01-01T00:00:00-00:44:40[Africa/Monrovia]"),
  "wrong :SS not accepted in string offset"
);
assert.throws(
  RangeError, () => action("1970-01-01T00:00:00-00:45:00[Africa/Monrovia]"),
  "rounded HH:MM:SS not accepted in string offset"
);
assert.throws(
  RangeError, () => action({ year: 1970, month: 1, day: 1, offset: "-00:45", timeZone: "Africa/Monrovia" }),
  "rounded HH:MM not accepted as offset in property bag"
);

// Pacific/Niue edge case

reference = new Temporal.ZonedDateTime(-543069621_000_000_000n, "Pacific/Niue");

assert.compareArray(
  action("1952-10-15T23:59:59-11:19:40[Pacific/Niue]"), [0, 0],
  "-11:19:40 is accepted as -11:19:40 in Pacific/Niue edge case"
);
assert.compareArray(
  action("1952-10-15T23:59:59-11:20[Pacific/Niue]"), [0, 0],
  "-11:20 matches the first candidate -11:19:40 in the Pacific/Niue edge case"
);
assert.compareArray(
  action("1952-10-15T23:59:59-11:20:00[Pacific/Niue]"), [1, -1],
  "-11:20:00 is accepted as -11:20:00 in the Pacific/Niue edge case"
);
assert.throws(
  RangeError, () => action("1952-10-15T23:59:59-11:19:50[Pacific/Niue]"),
  "wrong :SS not accepted in the Pacific/Niue edge case"
);
